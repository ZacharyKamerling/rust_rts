extern crate byteorder;

use data::game::{Game};
use data::kdt_point::{KDTUnit};
use data::logger::{MissileBoom};
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io::Cursor;
use std::f32;
use movement as mv;
use basic_unit;
use data::aliases::*;

pub fn encode(game: &Game, id: MissileID, vec: &mut Cursor<Vec<u8>>) {
    let ref misls = game.missiles;
    let _ = vec.write_u8(1);
    let _ = vec.write_u8(misls.missile_type[id] as u8);
    unsafe {
        let _ = vec.write_u16::<BigEndian>(id.usize_unwrap() as u16);
    }
    let _ = vec.write_u16::<BigEndian>((misls.x[id] * 64.0) as u16);
    let _ = vec.write_u16::<BigEndian>((misls.y[id] * 64.0) as u16);
}

pub fn step_missile(game: &mut Game, m_id: MissileID) {
    let dmg = game.missiles.damage[m_id];
    let mx = game.missiles.x[m_id];
    let my = game.missiles.y[m_id];
    move_missile(game, m_id);
    let mx2 = game.missiles.x[m_id];
    let my2 = game.missiles.y[m_id];
    let max_travel_dist = game.missiles.max_travel_dist[m_id];

    if game.missiles.travel_dist[m_id] > max_travel_dist {
        log_missile_boom(game, m_id, (mx2,my2));
        return;
    }

    let dist = game.missiles.speed[m_id];
    let nipae = nearest_intersected_point_and_enemy(game, m_id, (mx,my), (mx2,my2), dist);

    match dmg {
        Damage::Single(amount) => {
            match nipae {
                Some((t_id,(ix,iy))) => {
                    let dmg_type = game.missiles.damage_type[m_id];

                    basic_unit::damage_unit(game, t_id, amount, dmg_type);
                    log_missile_boom(game, m_id, (ix,iy));
                }
                None => ()
            }
        }
        Damage::Splash(amount,radius) => {
            match nipae {
                Some((_,(ix,iy))) => {
                    let dmg_type = game.missiles.damage_type[m_id];
                    let enemies = enemies_in_range(game, m_id, radius);

                    for enemy in enemies {
                        basic_unit::damage_unit(game, enemy.id, amount, dmg_type);
                    }

                    log_missile_boom(game, m_id, (ix,iy));
                }
                None => ()
            }
        }
    }
}

fn log_missile_boom(game: &mut Game, m_id: MissileID, (x,y): (f32,f32)) {
    let missile_type = game.missiles.missile_type[m_id];
    let boom = MissileBoom {
        id: m_id,
        missile_type: missile_type,
        x: x,
        y: y,
    };
    game.logger.missile_booms.push(boom);
}

fn move_missile(game: &mut Game, m_id: MissileID) {
    let m_x = game.missiles.x[m_id];
    let m_y = game.missiles.y[m_id];
    let facing = game.missiles.facing[m_id];
    let turn_rate = game.missiles.turn_rate[m_id];
    let speed = game.missiles.speed[m_id];

    game.missiles.travel_dist[m_id] += speed;

    match game.missiles.target[m_id] {
        Target::Unit(t_id) => {
            let t_x = game.units.x[t_id];
            let t_y = game.units.y[t_id];
            let t_speed = game.units.speed[t_id];
            let t_facing = game.units.facing[t_id];
            let (vx,vy) = mv::move_in_direction(0.0, 0.0, t_speed, t_facing);

            match mv::intercept_point((t_x,t_y), (m_x,m_y), (vx,vy), speed) {
                Some((ix,iy)) => {
                    let dx = ix - m_x;
                    let dy = iy - m_y;
                    let intercept_angle = mv::new(dx, dy);
                    let (m_x2,m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

                    game.missiles.x[m_id] = m_x2;
                    game.missiles.y[m_id] = m_y2;
                    game.missiles.facing[m_id] = mv::turn_towards(facing, intercept_angle, turn_rate);
                }
                None => {
                    let dx = t_x - m_x;
                    let dy = t_y - m_y;
                    let angle_to_target = mv::new(dx,dy);
                    let (m_x2,m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

                    game.missiles.x[m_id] = m_x2;
                    game.missiles.y[m_id] = m_y2;
                    game.missiles.facing[m_id] = mv::turn_towards(facing, angle_to_target, turn_rate);
                }
            }
        }
        Target::Point(px,py) => {
            let dx = px - m_x;
            let dy = py - m_y;
            let angle_to_target = mv::new(dx,dy);
            let (m_x2,m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

            game.missiles.x[m_id] = m_x2;
            game.missiles.y[m_id] = m_y2;
            game.missiles.facing[m_id] = mv::turn_towards(facing, angle_to_target, turn_rate);
        }
        Target::None => {
            let (m_x2,m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

            game.missiles.x[m_id] = m_x2;
            game.missiles.y[m_id] = m_y2;
        }
    }
}

fn enemies_in_range(game: &Game, m_id: MissileID, r: f32) -> Vec<KDTUnit> {
    let x = game.missiles.x[m_id];
    let y = game.missiles.y[m_id];
    let team = game.missiles.team[m_id];
    let target_type = game.missiles.target_type[m_id];

    let is_target = |b: &KDTUnit| {
        (b.team != team) &&
        (b.target_type == target_type) &&
        {
            let dx = b.x - x;
            let dy = b.y - y;
            let dr = b.radius + r;
            (dx * dx) + (dy * dy) <= dr * dr
        }
    };

    game.unit_kdt.in_range(&is_target, &[(x,r),(y,r)])
}

// Takes the game, a missile, where the missile was, where the missile is, and the distance traveled.
// Returns the nearest intersected point and the enemy unit that was intersected.
fn nearest_intersected_point_and_enemy(game: &Game, m_id: MissileID, (x,y): (f32,f32), (x2,y2): (f32,f32), dist: f32) -> Option<(UnitID,(f32,f32))> {
    let enemies = enemies_in_range(game, m_id, dist);

    if !enemies.is_empty() {
        let mut nearest_enemy = None;
        let mut nearest_dist = f32::MAX;

        for enemy in enemies {
            let ex = enemy.x;
            let ey = enemy.y;
            let er = enemy.radius;

            match mv::circle_line_intersection((x,y), (x2,y2), (ex,ey), er) {
                Some((ix,iy)) => {
                    let dx = ix - x;
                    let dy = iy - y;
                    let enemy_dist = dx * dx + dy * dy;

                    if enemy_dist < nearest_dist {
                        nearest_enemy = Some((enemy.id,(ix,iy)));
                        nearest_dist = enemy_dist;
                    }
                }
                None => ()
            }
        }

        nearest_enemy
    }
    else {
        None
    }
}