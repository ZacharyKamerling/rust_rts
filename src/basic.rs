extern crate rand;
extern crate byteorder;

use data::move_groups::{MoveGroupID};
use self::byteorder::{WriteBytesExt, BigEndian};
use self::rand::distributions::{Sample};
use std::io::Cursor;
use std::f32::consts::{PI};
use movement as mv;
use data::game::{Game};
use data::kdt_point::{KDTPoint};
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

pub fn encode(game: &Game, id: usize, vec: &mut Cursor<Vec<u8>>) {
    let ref units = game.units;
    let health = units.health[id];
    let max_health = units.max_health[id];
    let progress = units.progress[id];
    let progress_required = units.progress_required[id];
    let facing = mv::denormalize(units.facing[id]);

    let _ = vec.write_u8(0);
    let _ = vec.write_u8(units.unit_type[id] as u8);
    let _ = vec.write_u16::<BigEndian>(id as u16);
    let _ = vec.write_u16::<BigEndian>((units.x[id] * 64.0) as u16);
    let _ = vec.write_u16::<BigEndian>((units.y[id] * 64.0) as u16);
    let _ = vec.write_u8(units.anim[id] as u8);
    let _ = vec.write_u8(units.team[id] as u8);
    let _ = vec.write_u8((facing * 255.0 / (2.0 * PI)) as u8);
    let _ = vec.write_u8((health / max_health * 255.0) as u8);
    let _ = vec.write_u8((progress / progress_required * 255.0) as u8);

    for wpn_id in units.weapons[id].iter() {
        let ref wpns = game.weapons;
        let f = mv::denormalize(wpns.facing[*wpn_id]);
        let _ = vec.write_u8((f * 255.0 / (2.0 * PI)) as u8);
        let _ = vec.write_u8((wpns.anim[*wpn_id] as u8));
    }

    let ref capacity = units.capacity[id];
    if *capacity > (0 as usize) {
        let ref passengers = units.passengers[id];
        let _ = vec.write_u8((passengers.len() as u8));

        for psngr in passengers.iter() {
            let _ = vec.write_u16::<BigEndian>(*psngr as u16);
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

pub fn follow_order(game: &mut Game, id: usize) {
    let front = match game.units.orders[id].front() {
            None => None,
            Some(ok) => Some((*ok).clone())
    };

    match front {
        None => {
            game.units.is_moving[id] = false;
            slow_down(game, id);
        }
        Some(ord) => {
            match ord {
                Order::Move(x, y, mg_id) => {
                    game.units.is_moving[id] = true;
                    calculate_path(game, id, x as isize, y as isize);
                    prune_path(game, id);
                    turn_towards_path(game, id);
                    let the_end_is_near = approaching_end_of_path(game, id, mg_id);

                    if the_end_is_near || game.units.path[id].is_empty() {
                        game.units.orders[id].pop_front();
                        game.move_groups.done_moving(mg_id, game.units.radius[id]);
                    }
                    else {
                        speed_up(game, id);
                    }
                }
            }
        }
    }
    move_forward_and_collide_and_correct(game, id);
}

fn calculate_path(game: &mut Game, id: usize, x: isize, y: isize) {
    let team = game.units.team[id];
    let sx = game.units.x[id] as isize;
    let sy = game.units.y[id] as isize;

    if !game.units.path[id].is_empty() {
        let a = (sx,sy);
        let b = game.units.path[id][game.units.path[id].len() - 1];
        let a_to_b_open = game.teams.jps_grid[team].is_line_open(a, b);
        let b_to_a_open = game.teams.jps_grid[team].is_line_open(b, a);
        let destination_changed = (x,y) != game.units.path[id][0];

        if destination_changed || !a_to_b_open || !b_to_a_open {
            match game.teams.jps_grid[team].find_path((sx,sy),(x,y)) {
                None => {
                    game.units.path[id] = Vec::new();
                }
                Some(new_path) => {
                    game.units.path[id] = new_path;
                }
            }
        }
    }
    else {
        match game.teams.jps_grid[game.units.team[id]].find_path((sx,sy),(x,y)) {
            None => {
                game.units.path[id] = Vec::new();
            }
            Some(new_path) => {
                game.units.path[id] = new_path;
            }
        }
    }
}

fn prune_path(game: &mut Game, id: usize) {
    let ref mut path = game.units.path[id];
    let team = game.units.team[id];
    let sx = game.units.x[id] as isize;
    let sy = game.units.y[id] as isize;

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

fn move_forward_and_collide_and_correct(game: &mut Game, id: usize) {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let r = game.units.radius[id];
    let w = game.units.weight[id];
    let team = game.units.team[id];
    let speed = game.units.speed[id];
    let facing = game.units.facing[id];
    let moving = game.units.is_moving[id];

    let colliders = {
        let is_collider = |b: &KDTPoint| {
            game.units.is_flying[b.id] == game.units.is_flying[id] &&
            !game.units.is_structure[b.id] &&
            b.id != id &&
            !(b.x == x && b.y == y) &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
        };
        game.kdt.in_range(&is_collider, &[(x,r),(y,r)])
    };

    let num_colliders = colliders.len();
    let (mx,my) = mv::move_in_direction(x, y, speed, facing);

    let kdtp = KDTPoint {
        id: id,
        team: team,
        x: x,
        y: y,
        radius: r,
        weight: if moving { w * 6.0} else { w },
        flying: false,
        ground: false,
        structure: false,
        moving: moving,
    };

    let (ox,oy) = mv::collide(kdtp, colliders);
    let rx = game.random_offset_gen.sample(&mut game.game_rng);
    let ry = game.random_offset_gen.sample(&mut game.game_rng);

    if num_colliders == 0 {
        game.units.x_repulsion[id] = 0.0;
        game.units.y_repulsion[id] = 0.0;
    }
    else if num_colliders == 1 {
        game.units.x_repulsion[id] = game.units.x_repulsion[id] + ox;
        game.units.y_repulsion[id] = game.units.y_repulsion[id] + oy;
    }
    else {
        game.units.x_repulsion[id] = (game.units.x_repulsion[id] + ox * 0.625) * 0.9;
        game.units.y_repulsion[id] = (game.units.y_repulsion[id] + oy * 0.625) * 0.9;
    }

    let new_x = mx + rx + game.units.x_repulsion[id];
    let new_y = my + ry + game.units.y_repulsion[id];

    let (cx, cy, x_corrected, y_corrected) = game.bytegrid.correct_move((x, y), (new_x, new_y));

    if x_corrected {
        game.units.x_repulsion[id] = 0.0;
    }

    if y_corrected {
        game.units.y_repulsion[id] = 0.0;
    }

    if game.bytegrid.is_open((cx as isize, cy as isize)) {
        game.units.x[id] = cx;
        game.units.y[id] = cy;
    }

    let dx = cx - x;
    let dy = cy - y;

    /*
    let dist_traveled = f32::sqrt(dx * dx + dy * dy);

    if dist_traveled < speed {
        game.units.speed[id] = (speed * 4.0 + dist_traveled) / 5.0;
    }
    */
}

pub fn enemies_in_range(game: &Game, r: f32, id: usize, flying: bool, ground: bool, structure: bool) -> Vec<KDTPoint> {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let team = game.units.team[id];

    let predicate = |b: &KDTPoint| {
        if b.team != team && (b.flying == flying || b.ground == ground || b.structure == structure) {
            if b.structure {
                let min_x = b.x - b.radius;
                let min_y = b.y - b.radius;
                let max_x = b.x + b.radius;
                let max_y = b.y + b.radius;
                let bx = if x < min_x {min_x} else
                         if x > max_x {max_x} else {x};
                let by = if y < min_y {min_y} else
                         if y > max_y {max_y} else {y};
                let dx = bx - x;
                let dy = by - x;
                (dx * dx) + (dy * dy) <= r * r
            }
            else {
                let dx = b.x - x;
                let dy = b.y - x;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
        }
        else {
            false
        }
    };

    game.kdt.in_range(&predicate, &[(x,r),(y,r)])
}

fn slow_down(game: &mut Game, id: usize) {
    let new_speed = game.units.speed[id] - game.units.deceleration[id];
    game.units.speed[id] = if new_speed < 0.0 { 0.0 } else { new_speed };
}

fn speed_up(game: &mut Game, id: usize) {
    let new_speed = game.units.speed[id] + game.units.acceleration[id];
    let top_speed = game.units.top_speed[id];
    game.units.speed[id] = if new_speed > top_speed { top_speed } else { new_speed };
}

fn turn_towards_path(game: &mut Game, id: usize) {
    let ref path = game.units.path[id];

    if !path.is_empty() {
        let (nx,ny) = path[path.len() - 1];
        let gx = nx as f32 + 0.5;
        let gy = ny as f32 + 0.5;
        let sx = game.units.x[id];
        let sy = game.units.y[id];
        let ang = mv::new(gx - sx, gy - sy);
        let turn_rate = game.units.turn_rate[id];
        let facing = game.units.facing[id];

        game.units.facing[id] = mv::turn_towards(facing, ang, turn_rate);
    }
}

fn approaching_end_of_path(game: &mut Game, id: usize, mg_id: MoveGroupID) -> bool {
    let speed = game.units.speed[id];
    let radius = game.units.radius[id];
    let deceleration = game.units.deceleration[id];
    let ref path = game.units.path[id];
    let group_dist = game.move_groups.dist_to_group(mg_id);

    if path.len() == 1 {
        let (nx,ny) = path[0];
        let gx = nx as f32;
        let gy = ny as f32;
        let sx = game.units.x[id];
        let sy = game.units.y[id];
        let dx = gx - sx;
        let dy = gy - sy;
        let dist_to_stop = mv::dist_to_stop(speed, deceleration);
        let dist_to_end = dist_to_stop + group_dist + radius;

        (dist_to_end * dist_to_end) > (dx * dx + dy * dy)
    }
    else if path.len() == 0 {
        true
    }
    else {
        false
    }
}