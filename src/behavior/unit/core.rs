extern crate rand;
extern crate byteorder;

use data::move_groups::MoveGroup;
use data::build_groups::BuildTarget;
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io::Cursor;
use std::f64;
use std::f64::consts::PI;
use std::rc::Rc;
use std::collections::HashSet;
use data::kdt_point as kdtp;
use behavior::weapon::core as weapon;
use behavior::unit::building;
use libs::movement as mv;
use data::game::Game;
use data::kdt_point::{KDTUnit, KDTMissile};
use data::aliases::*;

/*
header = 1
type = 1
id = 2
x = 2
y = 2
anim = 1
team = 1
face = 1
health = 1
progress = 1
weapons = 1a (face,anim)
construction = id
num_psngrs = 1
psngr_ids = 2b

TOTAL = 13 + 1 * wpns + 2 * psngrs
*/

pub fn encode(game: &Game, id: UnitID, vec: &mut Cursor<Vec<u8>>) {
    let units = &game.units;
    let health = units.health(id);
    let max_health = units.max_health(id);
    let progress = units.progress(id);
    let build_cost = units.build_cost(id);
    let encoded_progress = if progress >= build_cost {
        255
    } else {
        (progress / build_cost * 255.0) as u8
    };
    let encoded_health = if health >= max_health {
        255
    } else {
        (health / max_health * 255.0) as u8
    };
    let facing = mv::denormalize(units.facing(id));

    if let Some(unit_type_id) = units.unit_type(id).clone() {
        unsafe {
            let _ = vec.write_u8(ClientMessage::UnitMove as u8);
            let _ = vec.write_u8(unit_type_id.usize_unwrap() as u8);
            let _ = vec.write_u16::<BigEndian>(id.usize_unwrap() as u16);
            let (x, y) = units.xy(id);
            let _ = vec.write_u16::<BigEndian>((x * 64.0) as u16);
            let _ = vec.write_u16::<BigEndian>((y * 64.0) as u16);
            let _ = vec.write_u8(units.anim(id) as u8);
            let _ = vec.write_u8(units.team(id).usize_unwrap() as u8);
            let _ = vec.write_u8((facing * 255.0 / (2.0 * PI)) as u8);
            let _ = vec.write_u8(encoded_health);
            let _ = vec.write_u8(encoded_progress);

            for wpn in units.weapons(id) {
                let f = mv::denormalize(wpn.facing());
                let _ = vec.write_u8((f * 255.0 / (2.0 * PI)) as u8);
            }

            let capacity = units.capacity(id);
            if capacity > (0 as usize) {
                let passengers = units.passengers(id);
                let _ = vec.write_u8(passengers.len() as u8);

                for psngr in passengers.iter() {
                    let _ = vec.write_u16::<BigEndian>((*psngr).usize_unwrap() as u16);
                }
            }

            let train_rate = game.units.train_rate(id);

            if train_rate > 0.0 {
                let train_order_front = game.units.train_queue(id).front().cloned();
                if let Some(train_order) = train_order_front {
                    let proto = game.units.proto(train_order.unit_type);
                    let train_cost = proto.build_cost();
                    let train_progress = game.units.train_progress(id);
                    let _ = vec.write_u8((train_progress / train_cost * 255.0) as u8);
                }
                else {
                    let _ = vec.write_u8(0);
                }
            }
        }
    }
    else {
        panic!("You probably have a bad unit name reference.");
    }
}

pub fn event_handler(game: &mut Game, event: UnitEvent) {
    if let UnitEvent::UnitSteps(id) = event {
		if game.units.progress(id) >= game.units.build_cost(id) {
			follow_top_order(game, id);
			let team = game.units.team(id);
            move_and_collide_and_correct(game, id);

            game.teams.prime_output[team] += game.units.prime_output(id);
            game.teams.energy_output[team] += game.units.energy_output(id);
            game.teams.prime[team] += game.units.prime_output(id);
            game.teams.energy[team] += game.units.energy_output(id);
            let health = game.units.health(id);
            let health_regen = game.units.health_regen(id);
            let max_health = game.units.max_health(id);
            game.units.set_health(
                id,
                f64::min(max_health, health + health_regen),
            );

            for i in 0..game.units.weapons(id).len() {
                let mut wpn = game.units.weapons(id)[i].clone();
                weapon::attack_orders(game, &mut wpn, id);
                game.units.mut_weapons(id)[i] = wpn;
            }
        }
    }
}

fn follow_top_order(game: &mut Game, id: UnitID) {
    let current_order = game.units.orders(id).front().cloned();

    match current_order {
        None => {
            slow_down(game, id);
        }
        Some(ord) => {
            follow_order(game, id, &*ord);
        }
    }
}

fn follow_order(game: &mut Game, id: UnitID, ord: &Order) {
    match ord.order_type {
        OrderType::Move(ref mg) => {
            proceed_on_path(game, id, mg);
        }
        OrderType::AttackMove(ref mg) => {
            let nearest_enemy = kdtp::nearest_visible_enemy_in_active_range(game, id);

            match nearest_enemy {
                Some(t_id) => {
                    let no_weapons = game.units.weapons(id).is_empty();

                    if no_weapons {
                        slow_down(game, id);
                    } else {
                        let wpn_range = game.units.weapons(id)[0].range();
                        let target_in_range = weapon::target_in_range(game, id, t_id, wpn_range);
                        let is_bomber = match game.units.weapons(id)[0].attack() {
                            &Attack::Bomb(_) |
                            &Attack::LaserBomb(_) => true,
                            _ => false,
                        };
                        if target_in_range && !is_bomber {
                            let (tx, ty) = game.units.xy(t_id);
                            turn_towards_point(game, id, tx, ty);
                            slow_down(game, id);
                        } else {
                            move_towards_target(game, id, t_id, mg);
                        }
                    }
                }
                None => {
                    proceed_on_path(game, id, mg);
                }
            }
        }
        OrderType::AttackTarget(ref mg, target) => {
            match game.units.target_id(target) {
                Some(t_id) => {
                    let team = game.units.team(id);
                    let is_visible = game.teams.visible[team][t_id].is_visible();
                    let (tx, ty) = game.units.xy(t_id);

                    if is_visible {
                        let wpn_range = game.units.weapons(id)[0].range();
                        let target_in_range = weapon::target_in_range(game, id, t_id, wpn_range);
                        let is_bomber = match game.units.weapons(id)[0].attack() {
                            &Attack::Bomb(_) |
                            &Attack::LaserBomb(_) => true,
                            _ => false,
                        };

                        if target_in_range && !is_bomber {
                            turn_towards_point(game, id, tx, ty);
                            slow_down(game, id);
                        } else {
                            let end_goal = game.units.xy(t_id);
                            mg.set_goal(end_goal);
                            move_towards_target(game, id, t_id, mg);
                        }
                    } else {
                        complete_order(game, id);
                    }
                }
                None => {
                    complete_order(game, id);
                }
            }
        }
        OrderType::Build(ref bg) => {
            match bg.build_target() {
                BuildTarget::Point(xy) => {
                    building::build_at_point(game, bg, id, xy);
                }
                BuildTarget::Unit(target) => {
                    match game.units.target_id(target) {
                        Some(t_id) => {
                            building::build_unit(game, id, t_id);
                        }
                        None => {
                            complete_order(game, id);
                        }
                    }
                }
            }
        }
        OrderType::Assist(mut target) => {
            let build_rate = game.units.build_rate(id);
            let mut assisters = HashSet::new();
            assisters.insert(target);

            if let Some(target_id) = game.units.target_id(target) {
                if let Some(target_order) = game.units.orders(target_id).front().cloned() {
                    if let OrderType::Assist(target_b) = target_order.order_type {
                        target = target_b;

                        // Prevent infinite loops of assisting
                        if assisters.contains(&target) {
                            complete_assist_order(game, id);
                            return;
                        }

                        assisters.insert(target);
                    }
                    else {
                        follow_order(game, id, &target_order);
                        return;
                    }
                }
                else {
                    let target_health = game.units.health(target_id);
                    let target_max_health = game.units.max_health(target_id);

                    if build_rate > 0.0 && (target_health < target_max_health) {
                        building::build_unit(game, id, target_id);
                        return;
                    }
                    else {
                        let xy = game.units.xy(target_id);
                        let radius = game.units.radius(id);
                        let target_radius = game.units.radius(target_id);
                        let speed = game.units.speed(id);
                        let deceleration = game.units.deceleration(id);
                        let dist_to_stop = mv::dist_to_stop(speed, deceleration);
                        move_towards_point(game, id, xy, dist_to_stop + speed + radius + target_radius + 5.0);
                        return;
                    }
                }
            }
            complete_assist_order(game, id);
        }
		OrderType::Stop => (),
    }
}

pub fn complete_assist_order(game: &mut Game, id: UnitID) {
    let opt_top_order = game.units.mut_orders(id).pop_front();
    if let Some(ref order) = opt_top_order {
        let order_completee = game.units.new_unit_target(id);
        game.logger.log_order_completed(
            order_completee,
            order.order_id,
        );
    }
}

pub fn complete_order(game: &mut Game, id: UnitID) {
    let opt_top_order: Option<Rc<Order>> = game.units.orders(id).front().cloned();
    if let Some(ref order) = opt_top_order {
        if let OrderType::Assist(_) = order.order_type {
            return;
        }
        let _ = game.units.mut_orders(id).pop_front();
        let order_completee = game.units.new_unit_target(id);
        game.logger.log_order_completed(
            order_completee,
            order.order_id,
        );
    }
}

pub fn complete_training(game: &mut Game, id: UnitID) {
    let opt_top_order: Option<TrainOrder> = game.units.train_queue(id).front().cloned();
    if let Some(ref order) = opt_top_order {
        let _ = game.units.mut_orders(id).pop_front();
        let order_completee = game.units.new_unit_target(id);
        game.logger.log_order_completed(
            order_completee,
            order.order_id,
        );
    }
}

fn move_towards_target(game: &mut Game, id: UnitID, t_id: UnitID, mg: &MoveGroup) {
    let team = game.units.team(id);
    let (ux, uy) = game.units.xy(id);
    let (tx, ty) = game.units.xy(t_id);
    let (ax, ay) = (ux as isize, uy as isize);
    let (bx, by) = (tx as isize, ty as isize);
    let a = (ax, ay);
    let b = (bx, by);

    let a_to_b_open = game.teams.jps_grid[team].is_line_open(a, b);
    let b_to_a_open = game.teams.jps_grid[team].is_line_open(b, a);

    if a_to_b_open && b_to_a_open {
        turn_towards_point(game, id, tx, ty);
        speed_up(game, id);
    } else {
        proceed_on_path(game, id, mg);
    }
}

fn move_towards_point(game: &mut Game, id: UnitID, (x,y): (f64,f64), dist: f64) {
    if game.units.move_type(id) == MoveType::Ground {
        let path_exists = calculate_path(game, id, (x as isize, y as isize));
        if !path_exists {
            complete_order(game, id);
            return;
        }
        prune_path(game, id);
        turn_towards_path(game, id);
        let should_brake = should_brake_now(game, id, dist);

        if should_brake || game.units.path(id).is_empty() {
            complete_order(game, id);
            slow_down(game, id);
        } else {
            speed_up(game, id);
        }
    } else if game.units.move_type(id) == MoveType::Air {
        turn_towards_point(game, id, x, y);
        let should_brake = should_brake_now(game, id, dist);

        if should_brake || game.units.path(id).is_empty() {
            complete_order(game, id);
            slow_down(game, id);
        } else {
            speed_up(game, id);
        }
    }
}

fn proceed_on_path(game: &mut Game, id: UnitID, mg: &MoveGroup) {
    let (x, y) = mg.goal();

    match game.units.move_type(id) {
        MoveType::Ground => {
            let path_exists = calculate_path(game, id, (x as isize, y as isize));
            if !path_exists {
                complete_order(game, id);
                return;
            }
            prune_path(game, id);
            turn_towards_path(game, id);
            let the_end_is_near = approaching_end_of_move_group_path(game, id, mg);
            let the_end_has_come = arrived_at_end_of_move_group_path(game, id, mg);

            if the_end_has_come || game.units.path(id).is_empty() {
                let radius = game.units.radius(id);
                let unit_target = game.units.new_unit_target(id);

                mg.done_moving(unit_target, radius);
                complete_order(game, id);
                slow_down(game, id);
            } else if the_end_is_near {
                slow_down(game, id);
            } else {
                speed_up(game, id);
            }
        }
        MoveType::Air => {
            turn_towards_point(game, id, x, y);
            let the_end_is_near = approaching_end_of_move_group_path(game, id, mg);
            let the_end_has_come = arrived_at_end_of_move_group_path(game, id, mg);

            if the_end_has_come || game.units.path(id).is_empty() {
                let radius = game.units.radius(id);
                let unit_target = game.units.new_unit_target(id);

                mg.done_moving(unit_target, radius);
                complete_order(game, id);
                slow_down(game, id);
            } else if the_end_is_near {
                slow_down(game, id);
            } else {
                speed_up(game, id);
            }
        }
        MoveType:: Water | MoveType::Underwater => {
            unimplemented!("Water / Underwater Movement");
        }
        MoveType::Hover | MoveType::Amphibious => {
            unimplemented!("Hover / Amphibious Movement");
        }
        MoveType::None => {
            complete_order(game, id);
        }
    }
}

pub fn calculate_path(game: &mut Game, id: UnitID, (x, y): (isize, isize)) -> bool {
    let team = game.units.team(id);
    let (sx, sy) = {
        let (zx, zy) = game.units.xy(id);
        (zx as isize, zy as isize)
    };

    if !game.units.path(id).is_empty() {
        let a = (sx, sy);
        let b = game.units.path(id)[game.units.path(id).len() - 1];
        let a_to_b_open = game.teams.jps_grid[team].is_line_open(a, b);
        let b_to_a_open = game.teams.jps_grid[team].is_line_open(b, a);
        let destination_changed = (x, y) != game.units.path(id)[0];

        if destination_changed || !a_to_b_open || !b_to_a_open {
            match game.teams.jps_grid[team].find_path((sx, sy), (x, y)) {
                None => {
                    // BAD WRONG FALSE STOP FREEZE
                    *game.units.mut_path(id) = Vec::new();
                    return false;
                }
                Some(new_path) => {
                    *game.units.mut_path(id) = new_path;
                    return true;
                }
            }
        }
        true
    } else {
        match game.teams.jps_grid[game.units.team(id)].find_path((sx, sy), (x, y)) {
            None => {
                *game.units.mut_path(id) = Vec::new();
                false
            }
            Some(new_path) => {
                *game.units.mut_path(id) = new_path;
                true
            }
        }
    }
}

pub fn prune_path(game: &mut Game, id: UnitID) {
    let team = game.units.team(id);
    let (sx, sy) = {
        let (zx, zy) = game.units.xy(id);
        (zx as isize, zy as isize)
    };

    let path = &mut game.units.mut_path(id);

    if path.len() >= 2 {
        let a = (sx, sy);
        let b = path[path.len() - 2];
        let a_to_b_open = game.teams.jps_grid[team].is_line_open(a, b);
        let b_to_a_open = game.teams.jps_grid[team].is_line_open(b, a);

        if a_to_b_open && b_to_a_open {
            path.pop();
        }
    }
}

fn move_forward(game: &Game, id: UnitID) -> (f64, f64) {
    let (x, y) = game.units.xy(id);
    let speed = game.units.speed(id);
    let facing = game.units.facing(id);
    mv::move_in_direction(x, y, speed, facing)
}

fn collide(game: &Game, id: UnitID) -> (f64, f64) {
    let (x, y) = game.units.xy(id);
    let r = game.units.collision_radius(id);
    let w = game.units.weight(id);
    let ratio = game.units.collision_ratio(id);
    let resist = game.units.collision_resist(id);

    let colliders = {
        let is_collider = |b: &KDTUnit| {
            if let Some(b_id) = game.units.target_id(b.target) {
                game.units.collision_type(id).has_a_match(
                    game.units.collision_type(b_id),
                ) && b_id != id && !((b.x - x).abs() < 0.000001 && (b.y - y).abs() < 0.000001) &&
                    {
                        let dx = b.x - x;
                        let dy = b.y - y;
                        let dr = b.collision_radius + r;
                        (dx * dx) + (dy * dy) <= dr * dr
                    }
            }
            else {
                false
            }
        };
        game.unit_kdt.in_range(&is_collider, &[(x, r), (y, r)])
    };

    let kdtp = KDTUnit {
        target: game.units.new_unit_target(id),
        x: x,
        y: y,
        radius: 0.0,
        collision_radius: r,
        weight: w,
    };

    let (x_off, y_off) = mv::collide(&kdtp, &colliders);
    let (x_repel, y_repel) = if colliders.is_empty() {
        (0.0, 0.0)
    }
    else {
        game.units.xy_repulsion(id)
    };

    ((x_repel + x_off * ratio) * resist, (y_repel + y_off * ratio) * resist)
}


// Moves unit forward (using its speed)
// Collides with other nearby units (using radii)
// Corrects the unit to not be on any unpathable terrain
pub fn move_and_collide_and_correct(game: &mut Game, id: UnitID) {
    let (x, y) = game.units.xy(id);
    let (mx, my) = move_forward(game, id);
    let (xo, yo) = collide(game, id);
    let rx = game.get_random_collision_offset();
    let ry = game.get_random_collision_offset();
    let (new_x, new_y, x_corrected, y_corrected) = game.bytegrid.correct_move(
        (x, y),
        (mx + xo + rx, my + yo + ry),
    );

    let x_dif = new_x - x;
    let y_dif = new_y - y;

    let dist_traveled = f64::sqrt(x_dif * x_dif + y_dif * y_dif);
    let reduct = f64::max(1.0, dist_traveled / game.units.top_speed(id));

    let x_repel;
    let y_repel;

    if x_corrected {
        x_repel = 0.0;
    } else {
        x_repel = xo / reduct;
    }

    if y_corrected {
        y_repel = 0.0;
    } else {
        y_repel = yo / reduct;
    }

    game.units.set_xy_repulsion(id, (x_repel, y_repel));

    if game.bytegrid.is_open((new_x as isize, new_y as isize)) {
        game.units.set_xy(id, (new_x, new_y));
    }
}

pub fn missiles_in_vision(game: &Game, id: UnitID) -> Vec<KDTMissile> {
    let (x, y) = game.units.xy(id);
    let r = game.units.sight_range(id);

    let is_visible = |b: &KDTMissile| {
        let dx = b.x - x;
        let dy = b.y - y;
        (dx * dx) + (dy * dy) <= r * r
    };

    game.missile_kdt.in_range(&is_visible, &[(x, r), (y, r)])
}

pub fn slow_down(game: &mut Game, id: UnitID) {
    let new_speed = game.units.speed(id) - game.units.deceleration(id);
    if new_speed <= 0.0 {
        game.units.set_speed(id, 0.0);
    } else {
        game.units.set_speed(id, new_speed);
    }
}

pub fn speed_up(game: &mut Game, id: UnitID) {
    let new_speed = game.units.speed(id) + game.units.acceleration(id);
    let top_speed = game.units.top_speed(id);
    game.units.set_speed(
        id,
        if new_speed > top_speed {
            top_speed
        } else {
            new_speed
        },
    );
}

pub fn turn_towards_path(game: &mut Game, id: UnitID) {

    if !game.units.path(id).is_empty() {
        let (nx, ny) = {
            let path = &game.units.path(id);
            path[path.len() - 1]
        };
        let gx = nx as f64 + 0.5;
        let gy = ny as f64 + 0.5;
        turn_towards_point(game, id, gx, gy);
    }
}

fn turn_towards_point(game: &mut Game, id: UnitID, gx: f64, gy: f64) {
    let (sx, sy) = game.units.xy(id);
    let ang = mv::new(gx - sx, gy - sy);
    let turn_rate = game.units.turn_rate(id);
    let facing = game.units.facing(id);

    game.units.set_facing(
        id,
        mv::turn_towards(facing, ang, turn_rate),
    );
}

/*
Returns true if the unit should brake now so it comes to
a complete stop [distance] from the end of the path.
*/
fn should_brake_now(game: &Game, id: UnitID, distance: f64) -> bool {
    let path = &game.units.path(id);

    if path.len() == 1 {
        let (nx, ny) = path[0];
        let gx = nx as f64 + 0.5;
        let gy = ny as f64 + 0.5;
        let (sx, sy) = game.units.xy(id);
        let dx = gx - sx;
        let dy = gy - sy;

        (distance * distance) > (dx * dx + dy * dy)
    } else {
        path.is_empty()
    }
}

fn approaching_end_of_move_group_path(game: &Game, id: UnitID, mg: &MoveGroup) -> bool {
    let speed = game.units.speed(id);
    let deceleration = game.units.deceleration(id);
    let dist_to_group = mg.dist_to_group();
    let dist_to_stop = mv::dist_to_stop(speed, deceleration);

    should_brake_now(game, id, dist_to_group + dist_to_stop)
}

fn arrived_at_end_of_move_group_path(game: &Game, id: UnitID, mg: &MoveGroup) -> bool {
    let speed = game.units.speed(id);
    let radius = game.units.radius(id);
    let dist_to_group = mg.dist_to_group();
    let dist_to_end = speed + radius;

    should_brake_now(game, id, dist_to_group + dist_to_end)
}

pub fn damage_unit(game: &mut Game, id: UnitID, amount: f64) {
    let health = game.units.health(id);

    if health >= 0.0 && health - amount <= 0.0 {
        game.logger.log_unit_death(id);
        game.units.kill_unit(id);
    }
    else {
        game.units.set_health(id, health - amount);
    }
}