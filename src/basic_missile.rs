use data::game::{Game};
use data::kdt_point::{KDTPoint};
use std::f32;
use movement as mv;
use data::aliases::*;

pub fn step_missile(game: &mut Game, m_id: MissileID) {
    let dmg = game.missiles.damage[m_id];
    let mx = game.missiles.x[m_id];
    let my = game.missiles.y[m_id];
    move_missile(game, m_id);
    let mx2 = game.missiles.x[m_id];
    let my2 = game.missiles.y[m_id];
    let dist = game.missiles.speed[m_id];
    let nipae = nearest_intersected_point_and_enemy(game, m_id, (mx,my), (mx2,my2), dist);

    match dmg {
        Damage::Single(amount) => {
            match nipae {
                Some((t_id,(ix,iy))) => {
                    game.units.health[t_id] -= amount;
                    write_boom_message(game, m_id, t_id, (ix,iy));
                    game.missiles.kill_missile(m_id);
                }
                None => ()
            }
        }
        Damage::Splash(amount,radius) => {
            match nipae {
                Some((t_id,(ix,iy))) => {
                    game.missiles.x[m_id] = ix;
                    game.missiles.y[m_id] = iy;
                    let enemies = enemies_in_range(game, m_id, radius);

                    for enemy in enemies {
                        game.units.health[enemy.id] -= amount;
                    }

                    write_boom_message(game, m_id, t_id, (ix,iy));
                    game.missiles.kill_missile(m_id);
                }
                None => ()
            }
        }
    }
}

fn write_boom_message(game: &mut Game, m_id: MissileID, t_id: UnitID, (x,y): (f32,f32)) {

}

fn move_missile(game: &mut Game, m_id: MissileID) {
    let m_x = game.missiles.x[m_id];
    let m_y = game.missiles.y[m_id];
    let facing = game.missiles.facing[m_id];
    let turn_rate = game.missiles.turn_rate[m_id];
    let speed = game.missiles.speed[m_id];
    let max_travel_dist = game.missiles.max_travel_dist[m_id];

    if game.missiles.travel_dist[m_id] > max_travel_dist {
        game.missiles.kill_missile(m_id);
        return;
    }

    game.missiles.travel_dist[m_id] += speed;

    match game.missiles.target[m_id] {
        Target::Unit(t_id) => {
            let t_x = game.units.x[t_id];
            let t_y = game.units.y[t_id];
            let t_speed = game.units.speed[t_id];
            let t_facing = game.units.facing[t_id];
            let (vx,vy) = mv::move_in_direction(0.0, 0.0, t_speed, t_facing);

            match mv::intercept_point((m_x,m_y), (t_x,t_y), (vx,vy), speed) {
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

fn enemies_in_range(game: &Game, m_id: MissileID, r: f32) -> Vec<KDTPoint> {
    let x = game.missiles.x[m_id];
    let y = game.missiles.y[m_id];
    let team = game.missiles.team[m_id];
    let target_type = game.missiles.target_type[m_id];

    let is_target = |b: &KDTPoint| {
        (b.team != team) &&
        (b.target_type == target_type) &&
        {
            let dx = b.x - x;
            let dy = b.y - y;
            let dr = b.radius + r;
            (dx * dx) + (dy * dy) <= dr * dr
        }
    };

    game.kdt.in_range(&is_target, &[(x,r),(y,r)])
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