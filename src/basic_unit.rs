extern crate rand;
extern crate byteorder;

use data::move_groups::{MoveGroupID};
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

pub fn encode(game: &Game, id: UnitID, vec: &mut Cursor<Vec<u8>>) {
    let ref units = game.units;
    let health = units.health[id];
    let max_health = units.max_health[id];
    let progress = units.progress[id];
    let progress_required = units.progress_required[id];
    let facing = mv::denormalize(units.facing[id]);

    let _ = vec.write_u8(0);
    let _ = vec.write_u8(units.unit_type[id] as u8);
    unsafe {
        let _ = vec.write_u16::<BigEndian>(id.usize_unwrap() as u16);
    }
    let _ = vec.write_u16::<BigEndian>((units.x[id] * 64.0) as u16);
    let _ = vec.write_u16::<BigEndian>((units.y[id] * 64.0) as u16);
    let _ = vec.write_u8(units.anim[id] as u8);
    unsafe {
        let _ = vec.write_u8(units.team[id].usize_unwrap() as u8);
    }
    let _ = vec.write_u8((facing * 255.0 / (2.0 * PI)) as u8);
    let _ = vec.write_u8((health / max_health * 255.0) as u8);
    let _ = vec.write_u8((progress / progress_required * 255.0) as u8);

    for w_id in units.weapons[id].iter() {
        let ref wpns = game.weapons;
        let f = mv::denormalize(wpns.facing[*w_id]);
        let _ = vec.write_u8((f * 255.0 / (2.0 * PI)) as u8);
        let _ = vec.write_u8((wpns.anim[*w_id] as u8));
    }

    let ref capacity = units.capacity[id];
    if *capacity > (0 as usize) {
        let ref passengers = units.passengers[id];
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
    let current_order = match game.units.orders[id].front() {
            None => None,
            Some(ok) => Some((*ok).clone())
    };

    match current_order {
        None => {
            idle(game, id);
        }
        Some(ord) => {
            match ord {
                Order::Move(mg_id) => {
                    proceed_on_path(game, id, mg_id);
                }
                Order::AttackMove(_) => {

                }
                Order::AttackTarget(_) => {

                }
            }
        }
    }
}

fn idle(game: &mut Game, id: UnitID) {
    slow_down(game, id);
    move_and_collide_and_correct(game, id);
}

fn proceed_on_path(game: &mut Game, id: UnitID, mg_id: MoveGroupID) {
    let (x,y) = game.units.move_groups.move_goal(mg_id);

    if game.units.is_ground[id] {
        calculate_path(game, id, x as isize, y as isize);
        prune_path(game, id);
        turn_towards_path(game, id);
        let the_end_is_near = approaching_end_of_path(game, id, mg_id);
        let the_end_has_come = arrived_at_end_of_path(game, id, mg_id);

        if the_end_has_come || game.units.path[id].is_empty() {
            game.units.orders[id].pop_front();
            let radius = game.units.radius[id];
            game.units.move_groups.done_moving(mg_id, radius);
        }
        else if the_end_is_near {
            slow_down(game, id);
        }
        else {
            speed_up(game, id);
        }
        move_and_collide_and_correct(game, id);
    }
    else if game.units.is_flying[id] {
        turn_towards_point(game, id, x, y);
        let the_end_is_near = approaching_end_of_path(game, id, mg_id);
        let the_end_has_come = arrived_at_end_of_path(game, id, mg_id);

        if the_end_has_come || game.units.path[id].is_empty() {
            game.units.orders[id].pop_front();
            let radius = game.units.radius[id];
            game.units.move_groups.done_moving(mg_id, radius);
        }
        else if the_end_is_near {
            slow_down(game, id);
        }
        else {
            speed_up(game, id);
        }
        move_and_collide_and_correct(game, id);
    }
}

fn calculate_path(game: &mut Game, id: UnitID, x: isize, y: isize) {
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

fn prune_path(game: &mut Game, id: UnitID) {
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

fn move_forward(game: &Game, id: UnitID) -> (f32,f32) {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let speed = game.units.speed[id];
    let facing = game.units.facing[id];
    mv::move_in_direction(x, y, speed, facing)
}

fn collide(game: &Game, id: UnitID) -> (f32,f32) {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let r = game.units.radius[id];
    let w = game.units.weight[id];
    let team = game.units.team[id];
    let speed = game.units.speed[id];
    let moving = speed > 0.0;

    let colliders = {
        let is_collider = |b: &KDTPoint| {
            game.units.is_ground[b.id] == game.units.is_ground[id] &&
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

    let kdtp = KDTPoint {
        id: id,
        team: team,
        x: x,
        y: y,
        radius: r,
        weight: if moving { w } else { w },
        flying: false,
        ground: false,
        structure: false,
        moving: moving,
    };

    let num_colliders = colliders.len();
    let (x_off, y_off) = mv::collide(kdtp, colliders);
    let x_repel = game.units.x_repulsion[id];
    let y_repel = game.units.y_repulsion[id];

    if num_colliders == 0 {
        (0.0, 0.0)
    }
    else if num_colliders == 1 {
        (x_repel + x_off, y_repel + y_off)
    }
    else {
        ((x_repel + x_off * 0.625) * 0.8, (y_repel + y_off * 0.625) * 0.8)
    }
}

/*
fn correct(game: &mut Game, id: UnitID, gx: f32, gy: f32) {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let (cx, cy, x_corrected, y_corrected) = game.bytegrid.correct_move((x,y), (gx,gy));

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
}
*/

fn move_and_collide_and_correct(game: &mut Game, id: UnitID) {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let (mx, my) = move_forward(&game, id);
    let (xo, yo) = collide(&game, id);
    let rx = game.get_random_offset();
    let ry = game.get_random_offset();
    let (new_x, new_y, x_corrected, y_corrected) = game.bytegrid.correct_move((x, y), (mx + xo + rx, my + yo + ry));

    let x_dif = new_x - x;
    let y_dif = new_y - y;

    let dist_traveled = f32::sqrt(x_dif * x_dif + y_dif * y_dif);
    let reduct = f32::max(1.0, dist_traveled / game.units.top_speed[id]);

    if x_corrected {
        game.units.x_repulsion[id] = 0.0;
    }
    else {
        game.units.x_repulsion[id] = xo / reduct;
    }

    if y_corrected {
        game.units.y_repulsion[id] = 0.0;
    }
    else {
        game.units.y_repulsion[id] = yo / reduct;
    }

    if game.bytegrid.is_open((new_x as isize, new_y as isize)) {
        game.units.x[id] = new_x;
        game.units.y[id] = new_y;
    }
}

pub fn enemies_in_vision(game: &Game, id: UnitID) -> Vec<KDTPoint> {
    let x = game.units.x[id];
    let y = game.units.y[id];
    let r = game.units.sight_range[id];
    let team = game.units.team[id];

    let is_collider = |b: &KDTPoint| {
        b.team != team &&
        {
            let dx = b.x - x;
            let dy = b.y - y;
            let dr = b.radius + r;
            (dx * dx) + (dy * dy) <= dr * dr
        }
    };

    game.kdt.in_range(&is_collider, &[(x,r),(y,r)])
}

fn slow_down(game: &mut Game, id: UnitID) {
    let new_speed = game.units.speed[id] - game.units.deceleration[id];
    if new_speed <= 0.0 {
        game.units.speed[id] = 0.0;
    }
    else {
        game.units.speed[id] = new_speed;
    }
}

fn speed_up(game: &mut Game, id: UnitID) {
    let new_speed = game.units.speed[id] + game.units.acceleration[id];
    let top_speed = game.units.top_speed[id];
    game.units.speed[id] = if new_speed > top_speed { top_speed } else { new_speed };
}

fn turn_towards_path(game: &mut Game, id: UnitID) {
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

fn turn_towards_point(game: &mut Game, id: UnitID, gx: f32, gy: f32) {
    let sx = game.units.x[id];
    let sy = game.units.y[id];
    let ang = mv::new(gx - sx, gy - sy);
    let turn_rate = game.units.turn_rate[id];
    let facing = game.units.facing[id];

    game.units.facing[id] = mv::turn_towards(facing, ang, turn_rate);
}

fn approaching_end_of_path(game: &mut Game, id: UnitID, mg_id: MoveGroupID) -> bool {
    let speed = game.units.speed[id];
    let deceleration = game.units.deceleration[id];
    let group_dist = game.units.move_groups.dist_to_group(mg_id);
    let ref path = game.units.path[id];

    if path.len() == 1 {
        let (nx,ny) = path[0];
        let gx = nx as f32 + 0.5;
        let gy = ny as f32 + 0.5;
        let sx = game.units.x[id];
        let sy = game.units.y[id];
        let dx = gx - sx;
        let dy = gy - sy;
        let dist_to_stop = mv::dist_to_stop(speed, deceleration);
        let dist_to_end = dist_to_stop + group_dist;

        (dist_to_end * dist_to_end) > (dx * dx + dy * dy)
    }
    else if path.len() == 0 {
        true
    }
    else {
        false
    }
}

fn arrived_at_end_of_path(game: &mut Game, id: UnitID, mg_id: MoveGroupID) -> bool {
    let speed = game.units.speed[id];
    let radius = game.units.radius[id];
    let group_dist = game.units.move_groups.dist_to_group(mg_id);
    let ref path = game.units.path[id];

    if path.len() == 1 {
        let (nx,ny) = path[0];
        let gx = nx as f32 + 0.5;
        let gy = ny as f32 + 0.5;
        let sx = game.units.x[id];
        let sy = game.units.y[id];
        let dx = gx - sx;
        let dy = gy - sy;
        let dist_to_end = group_dist + speed + radius;

        (dist_to_end * dist_to_end) > (dx * dx + dy * dy)
    }
    else if path.len() == 0 {
        true
    }
    else {
        false
    }
}