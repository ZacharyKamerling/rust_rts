extern crate rand;
extern crate byteorder;

use data::move_groups::{MoveGroup};
use data::build_groups::{BuildGroup,BuildTarget};
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io::Cursor;
use std::f32;
use std::f32::consts::{PI};
use data::kdt_point as kdtp;
use behavior::weapon::core as weapon;
use behavior::unit::building as building;
use libs::movement as mv;
use data::game::{Game};
use data::kdt_point::{KDTUnit,KDTMissile};
use data::logger::{UnitDeath};
use data::units::{UnitTarget};
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
weapons = 2a (face,anim)
num_psngrs = 1
psngr_ids = 2b

TOTAL = 13 + 2wpns + 2psngrs
*/

pub fn encode(game: &Game, id: UnitID, vec: &mut Cursor<Vec<u8>>) {
    let ref units = game.units;
    let health = units.health(id);
    let max_health = units.max_health(id);
    let progress = units.progress(id);
    let progress_required = units.progress_required(id);
    let encoded_progress =
        if progress == progress_required {
            255
        } else {
            (progress / progress_required * 254.0) as u8
        };
    let encoded_health =
        if health == max_health {
            255
        } else {
            (health / max_health * 254.0) as u8
        };
    let facing = mv::denormalize(units.facing(id));

    let _ = vec.write_u8(0);
    let _ = vec.write_u8(units.unit_type(id) as u8);
    unsafe {
        let _ = vec.write_u16::<BigEndian>(id.usize_unwrap() as u16);
    }
    let (x,y) = units.xy(id);
    let _ = vec.write_u16::<BigEndian>((x * 64.0) as u16);
    let _ = vec.write_u16::<BigEndian>((y * 64.0) as u16);
    let _ = vec.write_u8(units.anim(id) as u8);
    unsafe {
        let _ = vec.write_u8(units.team(id).usize_unwrap() as u8);
    }
    let _ = vec.write_u8((facing * 255.0 / (2.0 * PI)) as u8);
    let _ = vec.write_u8(encoded_health);
    let _ = vec.write_u8(encoded_progress);

    for w_id in units.weapons(id).iter() {
        let ref wpns = game.weapons;
        let f = mv::denormalize(wpns.facing[*w_id]);
        let _ = vec.write_u8((f * 255.0 / (2.0 * PI)) as u8);
        let _ = vec.write_u8(wpns.anim[*w_id] as u8);
    }

    let capacity = units.capacity(id);
    if capacity > (0 as usize) {
        let ref passengers = units.passengers(id);
        let _ = vec.write_u8((passengers.len() as u8));

        for psngr in passengers.iter() {
            unsafe {
                let _ = vec.write_u16::<BigEndian>((*psngr).usize_unwrap() as u16);
            }
        }
    }
}

pub fn event_handler(game: &mut Game, event: UnitEvent) {
    match event {
        UnitEvent::UnitSteps(id) => {
            follow_order(game, id);
        }
        _ => {

        }
    }
}

pub fn follow_order(game: &mut Game, id: UnitID) {
    let current_order = game.units.orders(id).front().map(|a| a.clone());

    match current_order {
        None => {
            slow_down(game, id);
        }
        Some(ord) => {
            match *ord {
                Order::Move(ref mg) => {
                    proceed_on_path(game, id, mg);
                }
                Order::AttackMove(ref mg) => {
                    let nearest_enemy = kdtp::nearest_visible_enemy_in_active_range(game, id);

                    match nearest_enemy {
                        Some(t_id) => {
                            let no_weapons = game.units.weapons(id).is_empty();

                            if no_weapons {
                                slow_down(game, id);
                            }
                            else {
                                let wpn_id = game.units.weapons(id)[0];
                                let wpn_range = game.weapons.range[wpn_id];
                                let target_in_range = weapon::target_in_range(game, id, t_id, wpn_range);
                                let is_bomber =
                                        match game.weapons.attack_type[wpn_id] {
                                            AttackType::BombAttack(_) | AttackType::LaserBombAttack(_) => true,
                                            _ => false
                                        };
                                if target_in_range && !is_bomber {
                                    let (tx,ty) = game.units.xy(t_id);
                                    turn_towards_point(game, id, tx, ty);
                                    slow_down(game, id);
                                }
                                else {
                                    move_towards_target(game, id, t_id, mg);
                                }
                            }
                        }
                        None => {
                            proceed_on_path(game, id, mg);
                        }
                    }
                }
                Order::AttackTarget(ref mg, unit_target) => {
                    match unit_target.id(&game.units) {
                        Some(t_id) => {
                            let team = game.units.team(id);
                            let is_visible = game.teams.visible[team][t_id];
                            let (tx,ty) = game.units.xy(t_id);

                            if is_visible {
                                let wpn_id = game.units.weapons(id)[0];
                                let wpn_range = game.weapons.range[wpn_id];
                                let target_in_range = weapon::target_in_range(game, id, t_id, wpn_range);
                                let is_bomber =
                                            match game.weapons.attack_type[wpn_id] {
                                                AttackType::BombAttack(_) | AttackType::LaserBombAttack(_) => {
                                                    true
                                                }
                                                _ => false
                                            };

                                if target_in_range && !is_bomber {
                                    turn_towards_point(game, id, tx, ty);
                                    slow_down(game, id);
                                }
                                else {
                                    let end_goal = game.units.xy(t_id);
                                    mg.set_goal(end_goal);
                                    move_towards_target(game, id, t_id, mg);
                                }
                            }
                            else {
                                game.units.mut_orders(id).pop_front();
                            }
                        }
                        None => {
                            game.units.mut_orders(id).pop_front();
                        }
                    }
                }
                Order::Build(ref bg) => {
                    match bg.build_target() {
                        BuildTarget::Point(xy) => {
                            building::build_at_point(game, bg, id, xy);
                        }
                        BuildTarget::Unit(target) => {
                            match target.id(&game.units) {
                                Some(t_id) => {
                                    building::build_unit(game, bg, id, t_id);
                                }
                                None => {
                                    game.units.mut_orders(id).pop_front();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn move_towards_target(game: &mut Game, id: UnitID, t_id: UnitID, mg: &MoveGroup) {
    let team = game.units.team(id);
    let (ux,uy) = game.units.xy(id);
    let (tx,ty) = game.units.xy(t_id);
    let (ax,ay) = (ux as isize, uy as isize);
    let (bx,by) = (tx as isize, ty as isize);
    let a = (ax,ay);
    let b = (bx,by);

    let a_to_b_open = game.teams.jps_grid[team].is_line_open(a, b);
    let b_to_a_open = game.teams.jps_grid[team].is_line_open(b, a);

    if a_to_b_open && b_to_a_open {
        turn_towards_point(game, id, tx, ty);
        speed_up(game, id);
    }
    else {
        proceed_on_path(game, id, mg);
    }
}

fn proceed_on_path(game: &mut Game, id: UnitID, mg: &MoveGroup) {
    let (x,y) = mg.goal();

    if game.units.target_type(id) == TargetType::Ground {
        calculate_path(game, id, (x as isize, y as isize));
        prune_path(game, id);
        turn_towards_path(game, id);
        let the_end_is_near = approaching_end_of_move_group_path(game, id, mg);
        let the_end_has_come = arrived_at_end_of_move_group_path(game, id, mg);

        if the_end_has_come || game.units.path(id).is_empty() {
            game.units.mut_orders(id).pop_front();
            let radius = game.units.radius(id);
            mg.done_moving(radius);
        }
        else if the_end_is_near {
            slow_down(game, id);
        }
        else {
            speed_up(game, id);
        }
    }
    else if game.units.target_type(id) == TargetType::Flyer {
        turn_towards_point(game, id, x, y);
        let the_end_is_near = approaching_end_of_move_group_path(game, id, mg);
        let the_end_has_come = arrived_at_end_of_move_group_path(game, id, mg);

        if the_end_has_come || game.units.path(id).is_empty() {
            game.units.mut_orders(id).pop_front();
            let radius = game.units.radius(id);
            mg.done_moving(radius);
        }
        else if the_end_is_near {
            slow_down(game, id);
        }
        else {
            speed_up(game, id);
        }
    }
}

pub fn calculate_path(game: &mut Game, id: UnitID, (x,y): (isize,isize)) {
    let team = game.units.team(id);
    let (sx,sy) = {
        let (zx,zy) = game.units.xy(id);
        (zx as isize, zy as isize)
    };

    if !game.units.path(id).is_empty() {
        let a = (sx,sy);
        let b = game.units.path(id)[game.units.path(id).len() - 1];
        let a_to_b_open = game.teams.jps_grid[team].is_line_open(a, b);
        let b_to_a_open = game.teams.jps_grid[team].is_line_open(b, a);
        let destination_changed = (x,y) != game.units.path(id)[0];

        if destination_changed || !a_to_b_open || !b_to_a_open {
            match game.teams.jps_grid[team].find_path((sx,sy),(x,y)) {
                None => {
                    *game.units.mut_path(id) = Vec::new();
                }
                Some(new_path) => {
                    *game.units.mut_path(id) = new_path;
                }
            }
        }
    }
    else {
        match game.teams.jps_grid[game.units.team(id)].find_path((sx,sy),(x,y)) {
            None => {
                *game.units.mut_path(id) = Vec::new();
            }
            Some(new_path) => {
                *game.units.mut_path(id) = new_path;
            }
        }
    }
}

pub fn prune_path(game: &mut Game, id: UnitID) {
    let team = game.units.team(id);
    let (sx,sy) = {
        let (zx,zy) = game.units.xy(id);
        (zx as isize, zy as isize)
    };

    let ref mut path = game.units.mut_path(id);

    if path.len() > 1 {
        let a = (sx,sy);
        let b = path[path.len() - 2];
        let a_to_b_open = game.teams.jps_grid[team].is_line_open(a, b);
        let b_to_a_open = game.teams.jps_grid[team].is_line_open(b, a);

        if a_to_b_open && b_to_a_open {
            path.pop();
        }
    }
}

fn move_forward(game: &Game, id: UnitID) -> (f32,f32) {
    let (x,y) = game.units.xy(id);
    let speed = game.units.speed(id);
    let facing = game.units.facing(id);
    mv::move_in_direction(x, y, speed, facing)
}

fn collide(game: &mut Game, id: UnitID) -> (f32,f32) {
    let (x,y) = game.units.xy(id);
    let r = game.units.collision_radius(id);
    let w = game.units.weight(id);
    let team = game.units.team(id);
    let speed = game.units.speed(id);
    let acceler = game.units.acceleration(id);
    let moving = speed > 0.0;
    let ratio = game.units.collision_ratio(id);
    let resist = game.units.collision_resist(id);

    let colliders = {
        let is_collider = |b: &KDTUnit| {
            game.units.target_type(id) == TargetType::Ground && game.units.target_type(b.id) == game.units.target_type(id) &&
            b.id != id &&
            !(b.x == x && b.y == y) &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.collision_radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
        };
        game.unit_kdt.in_range(&is_collider, &[(x,r),(y,r)])
    };

    let kdtp = KDTUnit {
        id: id,
        team: team,
        x: x,
        y: y,
        radius: 0.0,
        collision_radius: r,
        weight: if moving { w } else { w },
        target_type: TargetType::Ground,
        moving: moving,
    };

    let num_colliders = colliders.len();
    let (x_off, y_off) = mv::collide(kdtp, colliders);
    let (x_repel,y_repel) = game.units.xy_repulsion(id);

    game.units.set_speed(id, speed - acceler * (1.0 - (1.0 / f32::sqrt(1.0 + num_colliders as f32))));

    ((x_repel + x_off * ratio) * resist, (y_repel + y_off * ratio) * resist)
}


// Moves unit forward (using its speed)
// Collides with other nearby units (using radii)
// Corrects the unit to not be on any unpathable terrain
pub fn move_and_collide_and_correct(game: &mut Game, id: UnitID) {
    let (x,y) = game.units.xy(id);
    let (mx, my) = move_forward(&game, id);
    let (xo, yo) = collide(game, id);
    let rx = game.get_random_offset();
    let ry = game.get_random_offset();
    let (new_x, new_y, x_corrected, y_corrected) = game.bytegrid.correct_move((x, y), (mx + xo + rx, my + yo + ry));

    let x_dif = new_x - x;
    let y_dif = new_y - y;

    let dist_traveled = f32::sqrt(x_dif * x_dif + y_dif * y_dif);
    let reduct = f32::max(1.0, dist_traveled / game.units.top_speed(id));

    let x_repel;
    let y_repel;

    if x_corrected {
        x_repel = 0.0;
    }
    else {
        x_repel = xo / reduct;
    }

    if y_corrected {
        y_repel = 0.0;
    }
    else {
        y_repel = yo / reduct;
    }

    game.units.set_xy_repulsion(id, (x_repel, y_repel));

    if game.bytegrid.is_open((new_x as isize, new_y as isize)) {
        game.units.set_xy(id, (new_x, new_y));
    }
}

pub fn missiles_in_vision(game: &Game, id: UnitID) -> Vec<KDTMissile> {
    let (x,y) = game.units.xy(id);
    let r = game.units.sight_range(id);

    let is_visible = |b: &KDTMissile| {
        {
            let dx = b.x - x;
            let dy = b.y - y;
            (dx * dx) + (dy * dy) <= r * r
        }
    };

    game.missile_kdt.in_range(&is_visible, &[(x,r),(y,r)])
}

pub fn slow_down(game: &mut Game, id: UnitID) {
    let new_speed = game.units.speed(id) - game.units.deceleration(id);
    if new_speed <= 0.0 {
        game.units.set_speed(id, 0.0);
    }
    else {
        game.units.set_speed(id, new_speed);
    }
}

pub fn speed_up(game: &mut Game, id: UnitID) {
    let new_speed = game.units.speed(id) + game.units.acceleration(id);
    let top_speed = game.units.top_speed(id);
    game.units.set_speed(id, if new_speed > top_speed { top_speed } else { new_speed });
}

pub fn turn_towards_path(game: &mut Game, id: UnitID) {

    if !game.units.path(id).is_empty() {
        let (nx,ny) = {
            let ref path = game.units.path(id);
            path[path.len() - 1]
        };
        let gx = nx as f32 + 0.5;
        let gy = ny as f32 + 0.5;
        turn_towards_point(game, id, gx, gy);
    }
}

fn turn_towards_point(game: &mut Game, id: UnitID, gx: f32, gy: f32) {
    let (sx,sy) = game.units.xy(id);
    let ang = mv::new(gx - sx, gy - sy);
    let turn_rate = game.units.turn_rate(id);
    let facing = game.units.facing(id);

    game.units.set_facing(id, mv::turn_towards(facing, ang, turn_rate));
}

/*
Returns true if the unit should brake now so it comes to
a complete stop [distance] from the end of the path.
*/
fn should_brake_now(game: &Game, id: UnitID, distance: f32) -> bool {
    let ref path = game.units.path(id);

    if path.len() == 1 {
        let (nx,ny) = path[0];
        let gx = nx as f32 + 0.5;
        let gy = ny as f32 + 0.5;
        let (sx,sy) = game.units.xy(id);
        let dx = gx - sx;
        let dy = gy - sy;

        (distance * distance) > (dx * dx + dy * dy)
    }
    else if path.len() == 0 {
        true
    }
    else {
        false
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

pub fn damage_unit(game: &mut Game, id: UnitID, amount: f32, dmg_type: DamageType) {
    let health = game.units.health(id);
    game.units.set_health(id, health - amount);

    if health > 0.0 && health - amount <= 0.0 {
        log_unit_death(game, id, dmg_type);
    }
}

fn log_unit_death(game: &mut Game, id: UnitID, dmg_type: DamageType) {
    let death = UnitDeath {
        id: id,
        damage_type: dmg_type,
    };

    game.logger.unit_deaths.push(death);
}