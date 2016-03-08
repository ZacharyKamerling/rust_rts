use data::game::{Game};
use data::units::UnitTarget;
use data::kdt_point::{KDTUnit};
use data::kdt_point as kdtp;
use std::f32;
use movement as mv;
use data::aliases::*;

pub fn attack_orders(game: &mut Game, w_id: WeaponID, u_id: UnitID) {
    cooldown_weapon(game, w_id);
    match game.units.orders[u_id].front() {
        Some(&order) => {
            match order {
                Order::AttackMove(_) => {
                    match game.weapons.target_id[w_id] {
                        Some(unit_target) => {
                            match unit_target.id(&game.units) {
                                Some(t_id) => {
                                    let wpn_range = game.weapons.range[w_id];
                                    if target_in_range(game, u_id, t_id, wpn_range) {
                                        attack_target(game, w_id, u_id, t_id);
                                    }
                                    else {
                                        attack_nearest_enemy(game, w_id, u_id);
                                    }
                                }
                                None => {
                                    game.weapons.target_id[w_id] = None;
                                    attack_nearest_enemy(game, w_id, u_id);
                                }
                            }
                        }
                        None => {
                            attack_nearest_enemy(game, w_id, u_id);
                        }
                    }
                }
                Order::AttackTarget(unit_target) => {
                    match unit_target.id(&game.units) {
                        Some(t_id) => {
                            let wpn_range = game.weapons.range[w_id];
                            if target_in_range(game, u_id, t_id, wpn_range) {
                                attack_target(game, w_id, u_id, t_id);
                            }
                            else {
                                attack_nearest_enemy(game, w_id, u_id);
                            }
                        }
                        None => {
                            attack_nearest_enemy(game, w_id, u_id);
                        }
                    }
                }
                Order::Move(_) => {
                    attack_nearest_enemy(game, w_id, u_id);
                }
            }
        }
        None => {
            attack_nearest_enemy(game, w_id, u_id);
        }
    }
}

fn attack_nearest_enemy(game: &mut Game, w_id: WeaponID, u_id: UnitID) {
    match get_nearest_enemy(game, w_id, u_id) {
        Some(t_id) => {
            game.weapons.target_id[w_id] = Some(UnitTarget::new(&game.units, t_id));
            attack_target(game, w_id, u_id, t_id);
        }
        None => {
            // Return weapon to resting position
            let unit_facing = game.units.facing[u_id];
            let wpn_facing = game.weapons.facing[w_id];
            let turn_rate = game.weapons.turn_rate[w_id];
            let wpn_lock_angle = game.weapons.lock_offset[w_id];

            game.weapons.target_id[w_id] = None;
            game.weapons.facing[w_id] = mv::turn_towards(wpn_facing, unit_facing + wpn_lock_angle, turn_rate);
        }
    }
}

fn attack_target(game: &mut Game, w_id: WeaponID, u_id: UnitID, t_id: UnitID) {
    match game.weapons.attack_type[w_id] {
        AttackType::MissileAttack(missile_type) => {
            attack_target_with_missile_salvo(game, missile_type, w_id, u_id, t_id);
        }
        AttackType::MeleeAttack(damage) => {

        }
        AttackType::LaserAttack(damage) => {

        }
        AttackType::BombAttack(missile_type) => {

        }
        AttackType::LaserBombAttack(damage) => {

        }
    }
}

fn attack_target_with_missile_salvo(game: &mut Game, missile_type: MissileTypeID, w_id: WeaponID, u_id: UnitID, t_id: UnitID) {
    let target_facing = game.units.facing[t_id];
    let target_speed = game.units.speed[t_id];
    let missile_speed = game.weapons.missile_speed[w_id];
    let tx = game.units.x[t_id];
    let ty = game.units.y[t_id];
    let (vx,vy) = mv::move_in_direction(0.0, 0.0, target_speed, target_facing);
    let (wpn_x, wpn_y) = get_weapon_position(game, w_id, u_id);

    match mv::intercept_point((tx,ty), (wpn_x,wpn_y), (vx,vy), missile_speed) {
        Some((ix,iy)) => {
            let on_target = turn_weapon_to_point(game, w_id, u_id, (ix, iy));

            if on_target {
                fire_missile_salvo_at_target(game, missile_type, w_id, u_id, t_id);
            }
        }
        None => ()
    }
}

fn weapon_is_ready_to_fire(game: &Game, w_id: WeaponID) -> bool {
    let cooldown = game.weapons.cooldown[w_id];
    let salvo_cooldown = game.weapons.salvo_cooldown[w_id];
    let salvo = game.weapons.salvo[w_id];
    let salvo_size = game.weapons.salvo_size[w_id];

    cooldown <= 0 || (salvo_cooldown <= 0 && salvo < salvo_size)
}

fn heatup_weapon(game: &mut Game, w_id: WeaponID) {
    if game.weapons.cooldown[w_id] <= 0 {
        game.weapons.cooldown[w_id] += (FPS as isize) * game.weapons.fire_rate[w_id];
        game.weapons.salvo[w_id] = 0;
        game.weapons.salvo_cooldown[w_id] = 0;
    }

    game.weapons.salvo[w_id] += 1;
    game.weapons.salvo_cooldown[w_id] += (FPS as isize) * game.weapons.salvo_fire_rate[w_id];
}

fn cooldown_weapon(game: &mut Game, w_id: WeaponID) {
    let cooldown = game.weapons.cooldown[w_id];
    let salvo_cooldown = game.weapons.salvo_cooldown[w_id];

    if salvo_cooldown > 0 {
        game.weapons.salvo_cooldown[w_id] -= 1000;
    }

    if cooldown > 0 {
        game.weapons.cooldown[w_id] -= 1000;
    }
}

fn fire_missile_salvo_at_target(game: &mut Game, missile_type: MissileTypeID, w_id: WeaponID, u_id: UnitID, t_id: UnitID) {
    if weapon_is_ready_to_fire(game, w_id) {
        heatup_weapon(game, w_id);

        let (wpn_x, wpn_y) = get_weapon_position(game, w_id, u_id);
        let wpn_facing = game.weapons.facing[w_id];

        for _ in 0..game.weapons.pellet_count[w_id] {
            match game.missiles.make_missile(missile_type) {
                Some(m_id) => {
                    game.missiles.target[m_id] = Target::Unit(UnitTarget::new(&game.units, t_id));
                    game.missiles.facing[m_id] = wpn_facing;
                    game.missiles.x[m_id] = wpn_x;
                    game.missiles.y[m_id] = wpn_y;
                    game.missiles.team[m_id] = game.units.team[u_id];
                    game.missiles.target_type[m_id] = game.units.target_type[t_id];
                }
                None => ()
            }
        }
    }
}

fn get_weapon_position(game: &Game, w_id: WeaponID, u_id: UnitID) -> (f32,f32) {
    let facing = game.units.facing[u_id];
    let x = game.units.x[u_id];
    let y = game.units.y[u_id];
    let x_off = game.weapons.x_offset[w_id];
    let y_off = game.weapons.y_offset[w_id];

    mv::get_offset_position((x,y), facing, (x_off, y_off))
}

fn turn_weapon_to_point(game: &mut Game, w_id: WeaponID, u_id: UnitID, (x,y): (f32,f32)) -> bool {
    let (wpn_x, wpn_y) = get_weapon_position(game, w_id, u_id);
    let dx = x - wpn_x;
    let dy = y - wpn_y;
    let wpn_facing = game.weapons.facing[w_id];
    let angle_to_enemy = mv::new(dx, dy);
    let wpn_turn_rate = game.weapons.turn_rate[w_id];

    if mv::distance(wpn_facing, angle_to_enemy) <= wpn_turn_rate {
        game.weapons.facing[w_id] = angle_to_enemy;
        true
    }
    else {
        game.weapons.facing[w_id] = mv::turn_towards(wpn_facing, angle_to_enemy, wpn_turn_rate);
        false
    }
}

pub fn target_in_range(game: &mut Game, u_id: UnitID, t_id: UnitID, range: f32) -> bool {
    let radius = game.units.radius[u_id];
    let target_radius = game.units.radius[t_id];
    let total_range = range + radius + target_radius;
    let xa = game.units.x[u_id];
    let ya = game.units.y[u_id];
    let xb = game.units.x[t_id];
    let yb = game.units.y[t_id];
    let dx = xa - xb;
    let dy = ya - yb;
    let team = game.units.team[u_id];
    let is_visible = game.teams.visible[team][t_id];

    is_visible && (dx * dx + dy * dy) <= (total_range * total_range)
}

fn get_nearest_enemy(game: &Game, w_id: WeaponID, u_id: UnitID) -> Option<UnitID> {
    let range = game.weapons.range[w_id];
    let radius = game.units.radius[u_id];
    let enemies = kdtp::enemies_in_range_and_firing_arc(game, range + radius, u_id, w_id);

    if !enemies.is_empty() {
        let mut nearest_enemy = None;
        let mut nearest_dist = f32::MAX;
        let xa = game.units.x[u_id];
        let ya = game.units.y[u_id];

        for enemy in enemies {
            let xb = enemy.x;
            let yb = enemy.y;
            let dx = xb - xa;
            let dy = yb - ya;
            let enemy_dist = dx * dx + dy * dy;

            if enemy_dist < nearest_dist {
                nearest_enemy = Some(enemy.id);
                nearest_dist = enemy_dist;
            }
        }

        nearest_enemy
    }
    else {
        None
    }
}