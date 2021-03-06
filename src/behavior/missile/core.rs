extern crate byteorder;

use data::game::Game;
use data::kdt_point::KDTUnit;
use self::byteorder::{WriteBytesExt, BigEndian};
use std::io::Cursor;
use std::f64;
use libs::movement as mv;
use behavior::unit::core as unit;
use data::aliases::*;

pub fn encode(game: &Game, id: MissileID, vec: &mut Cursor<Vec<u8>>) {
    let misls = &game.missiles;
    let (x, y) = misls.xy(id);

    if x < 0.0 || y < 0.0 || x >= 1024.0 || y >= 1024.0 {
        return;
    }

    if let Some(missile_type_id) = misls.missile_type_id(id).clone() {
        unsafe {
            let _ = vec.write_u8(ClientMessage::MissileMove as u8);
            let _ = vec.write_u8(missile_type_id.usize_unwrap() as u8);
            let _ = vec.write_u16::<BigEndian>(id.usize_unwrap() as u16);
            let _ = vec.write_u16::<BigEndian>((x * 64.0) as u16);
            let _ = vec.write_u16::<BigEndian>((y * 64.0) as u16);
            let _ = vec.write_u8(misls.team(id).usize_unwrap() as u8);
        }
    }
    else {
        let name = misls.name(id);
        panic!("You have a bad missile reference for {}.", name);
    }
}

pub fn step_missile(game: &mut Game, m_id: MissileID) {
    let dmg = game.missiles.damage(m_id);
    let (mx, my) = game.missiles.xy(m_id);
    move_missile(game, m_id);
    let (mx2, my2) = game.missiles.xy(m_id);
    let max_travel_dist = game.missiles.max_travel_dist(m_id);
    let missile_type_id = game.missiles.missile_type_id(m_id).clone().unwrap();
    let team = game.missiles.team(m_id);

    if game.missiles.travel_dist(m_id) > max_travel_dist {
        game.logger.log_missile_boom(
            missile_type_id,
            m_id,
            team,
            (mx2, my2),
        );
        return;
    }

    let dist = game.missiles.speed(m_id);
    let nipae = nearest_intersected_point_and_enemy(game, m_id, (mx, my), (mx2, my2), dist);

    match dmg {
        Damage::Single(amount) => {
            if let Some((t_id, (ix, iy))) = nipae {
                unit::damage_unit(game, t_id, amount);
                game.logger.log_missile_boom(
                    missile_type_id,
                    m_id,
                    team,
                    (ix, iy),
                );
            }
        }
        Damage::Splash(amount, radius) => {
            if let Some((_, (ix, iy))) = nipae {
                let enemies = enemies_in_range(game, m_id, radius);

                for enemy in enemies {
                    if let Some(id) = game.units.target_id(enemy.target) {
                        unit::damage_unit(game, id, amount);
                    }
                }

                game.logger.log_missile_boom(
                    missile_type_id,
                    m_id,
                    team,
                    (ix, iy),
                );
            }
        }
    }
}

fn move_missile(game: &mut Game, m_id: MissileID) {
    let (m_x, m_y) = game.missiles.xy(m_id);
    let facing = game.missiles.facing(m_id);
    let turn_rate = game.missiles.turn_rate(m_id);
    let speed = game.missiles.speed(m_id);
    let travel_dist = game.missiles.travel_dist(m_id);

    game.missiles.set_travel_dist(m_id, travel_dist + speed);

    match game.missiles.target(m_id) {
        Target::Unit(target) => {
            match game.units.target_id(target) {
                Some(t_id) => {
                    let (t_x, t_y) = game.units.xy(t_id);
                    let t_speed = game.units.speed(t_id);
                    let t_facing = game.units.facing(t_id);
                    let (vx, vy) = mv::move_in_direction(0.0, 0.0, t_speed, t_facing);

                    match mv::intercept_point((t_x, t_y), (m_x, m_y), (vx, vy), speed) {
                        Some((ix, iy)) => {
                            let dx = ix - m_x;
                            let dy = iy - m_y;
                            let intercept_angle = mv::new(dx, dy);
                            let (m_x2, m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

                            game.missiles.set_xy(m_id, (m_x2, m_y2));
                            game.missiles.set_facing(m_id, mv::turn_towards(facing, intercept_angle, turn_rate));
                        }
                        None => {
                            let dx = t_x - m_x;
                            let dy = t_y - m_y;
                            let angle_to_target = mv::new(dx, dy);
                            let (m_x2, m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

                            game.missiles.set_xy(m_id, (m_x2, m_y2));
                            game.missiles.set_facing(m_id, mv::turn_towards(facing, angle_to_target, turn_rate));
                        }
                    }
                }
                None => {
                    let (m_x2, m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);
                    game.missiles.set_xy(m_id, (m_x2, m_y2));
                }
            }
        }
        Target::Point(px, py) => {
            let dx = px - m_x;
            let dy = py - m_y;
            let angle_to_target = mv::new(dx, dy);
            let (m_x2, m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

            game.missiles.set_xy(m_id, (m_x2, m_y2));
            game.missiles.set_facing(m_id, mv::turn_towards(facing, angle_to_target, turn_rate));
        }
        Target::None => {
            let (m_x2, m_y2) = mv::move_in_direction(m_x, m_y, speed, facing);

            game.missiles.set_xy(m_id, (m_x2, m_y2));
        }
    }
}

fn enemies_in_range(game: &Game, m_id: MissileID, r: f64) -> Vec<KDTUnit> {
    let (x, y) = game.missiles.xy(m_id);
    let team = game.missiles.team(m_id);
    let target_type = game.missiles.target_type(m_id);

    let is_target = |b: &KDTUnit| {
        if let Some(b_id) = game.units.target_id(b.target) {
            let b_team = game.units.team(b_id);
            let tt = game.units.target_type(b_id);
            (b_team != team) && (tt.has_a_match(target_type)) &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
        }
        else {
            false
        }
    };

    game.unit_kdt.in_range(&is_target, &[(x, r), (y, r)])
}

// Takes the game, a missile, where the missile was, where the missile is, and the distance traveled.
// Returns the nearest intersected point and the enemy unit that was intersected.
fn nearest_intersected_point_and_enemy(
    game: &Game,
    m_id: MissileID,
    (x, y): (f64, f64),
    (x2, y2): (f64, f64),
    dist: f64,
) -> Option<(UnitID, (f64, f64))> {
    let enemies = enemies_in_range(game, m_id, dist);

    if !enemies.is_empty() {
        let mut nearest_enemy = None;
        let mut nearest_dist = f64::MAX;

        for enemy in enemies {
            if let Some(id) = game.units.target_id(enemy.target) {
                let ex = enemy.x;
                let ey = enemy.y;
                let er = enemy.radius;

                if let Some((ix, iy)) = mv::circle_line_intersection((x, y), (x2, y2), (ex, ey), er) {
                    let dx = ix - x;
                    let dy = iy - y;
                    let enemy_dist = dx * dx + dy * dy;

                    if enemy_dist < nearest_dist {
                        nearest_enemy = Some((id, (ix, iy)));
                        nearest_dist = enemy_dist;
                    }
                }
            }
        }

        nearest_enemy
    } else {
        None
    }
}