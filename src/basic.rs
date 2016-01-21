extern crate byteorder;

use self::byteorder::{WriteBytesExt, BigEndian};
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
    let x = game.units.x[id];
    let y = game.units.y[id];
    let r = game.units.radius[id];

    let colliders = {
        let is_collider = |b: &KDTPoint| {
            game.units.is_flying[b.id] == game.units.is_flying[id] &&
            !game.units.is_structure[b.id] &&
            b.id != id &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
        };
        game.kdt.in_range(&is_collider, &[(x,r),(y,r)])
    };

    match front {
        None => {
            slow_down(game, id);
        }
        Some(ord) => {
            match ord {
                Order::Move(x,y) => {
                    calculate_path(game, id, x as isize, y as isize);
                    prune_path(game, id);
                    turn_towards_path(game, id);
                    let the_end_is_near = approaching_end_of_path(game, id);

                    if colliders.len() > 5 || the_end_is_near {
                        slow_down(game, id);
                        if the_end_is_near && game.units.speed[id] == 0.0 {
                            game.units.orders[id].pop_front();
                        }
                    }
                    else {
                        speed_up(game, id);
                    }
                }
            }
        }
    }
    move_forward_and_collide_and_correct(game, id, colliders);
}

fn calculate_path(game: &mut Game, id: usize, x: isize, y: isize) {
    let sx = game.units.x[id] as isize;
    let sy = game.units.y[id] as isize;

    if !game.units.path[id].is_empty() {
        if (x,y) != game.units.path[id][0] {
            match game.teams.jps_grid[game.units.team[id]].find_path((sx,sy),(x,y)) {
                None => {
                    game.units.path[id] = Vec::new();
                }
                Some(path) => {
                    game.units.path[id] = path;
                }
            }
        }
    }
    else {
        match game.teams.jps_grid[game.units.team[id]].find_path((sx,sy),(x,y)) {
            None => {
                println!("{} no path found from {} : {}", id, sx, sy);
                game.units.path[id] = Vec::new();
            }
            Some(path) => {
                game.units.path[id] = path;
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
        if game.teams.jps_grid[team].is_line_open((sx,sy), path[path.len() - 2]) {
            path.pop();
        }
    }
}

fn move_forward_and_collide_and_correct(game: &mut Game, id: usize, colliders: Vec<KDTPoint>) {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let r = game.units.radius[id];
    let w = game.units.weight[id];
    let team = game.units.team[id];
    let speed = game.units.speed[id];
    let facing = game.units.facing[id];

    let (mx,my) = mv::move_in_direction(x, y, speed, facing);

    let kdtp = KDTPoint {
        id: id,
        team: team,
        x: mx,
        y: my,
        radius: r,
        weight: w,
        flying: false,
        ground: false,
        structure: false,
        moving: false,
    };

    let (ox,oy) = mv::collide(kdtp, colliders);
    let (cx,cy) = game.bytegrid.correct_move((x, y), (mx + ox, my + oy));

    /*
    println!("Before move {} {} {}", x, y, speed);
    println!("After move {} {}", mx, my);
    println!("Offsets {} {}", ox, oy);
    println!("After correct {} {}", cx, cy);
    */

    game.units.x[id] = cx;
    game.units.y[id] = cy;
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
    let ref mut path = game.units.path[id];

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

fn approaching_end_of_path(game: &mut Game, id: usize) -> bool {
    let ref mut path = game.units.path[id];

    if path.len() == 1 {
        let (nx,ny) = path[0];
        let gx = nx as f32;
        let gy = ny as f32;
        let sx = game.units.x[id];
        let sy = game.units.y[id];
        let dx = gx - sx;
        let dy = gy - sy;
        let dist_to_stop = mv::dist_to_stop(game.units.top_speed[id], game.units.deceleration[id]);

        (dist_to_stop * dist_to_stop) > (dx * dx + dy * dy)
    }
    else if path.len() == 0 {
        true
    }
    else {
        false
    }
}