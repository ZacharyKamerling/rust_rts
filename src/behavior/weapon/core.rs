extern crate rand;

use data::game::Game;
use data::units::Weapon;
use self::rand::Rng;
use data::kdt_point as kdtp;
use behavior::unit::core as unit;
use libs::movement as mv;
use data::aliases::*;

pub fn attack_orders(game: &mut Game, wpn: &mut Weapon, u_id: UnitID) {
    let current_order = game.units.orders(u_id).front().cloned();

    cooldown_weapon(wpn);
    match current_order {
        Some(ord) => {
            match (*ord).order_type {
                OrderType::AttackMove(_) => {
                    match wpn.target_id() {
                        Some(unit_target) => {
                            match game.units.target_id(unit_target) {
                                Some(t_id) => {
                                    let wpn_range = wpn.range();
                                    if target_in_range(game, u_id, t_id, wpn_range) {
                                        attack_target(game, wpn, u_id, t_id);
                                    } else {
                                        attack_nearest_enemy(game, wpn, u_id);
                                    }
                                }
                                None => {
                                    wpn.set_target_id(None);
                                    attack_nearest_enemy(game, wpn, u_id);
                                }
                            }
                        }
                        None => {
                            attack_nearest_enemy(game, wpn, u_id);
                        }
                    }
                }
                OrderType::AttackTarget(_, unit_target) => {
                    match game.units.target_id(unit_target) {
                        Some(t_id) => {
                            let wpn_range = wpn.range();
                            if target_in_range(game, u_id, t_id, wpn_range) {
                                attack_target(game, wpn, u_id, t_id);
                            } else {
                                attack_nearest_enemy(game, wpn, u_id);
                            }
                        }
                        None => {
                            attack_nearest_enemy(game, wpn, u_id);
                        }
                    }
                }
                OrderType::Move(_) |
                OrderType::Build(_) |
                OrderType::Assist(_) => {
                    attack_nearest_enemy(game, wpn, u_id);
                }
            }
        }
        None => {
            attack_nearest_enemy(game, wpn, u_id);
        }
    }
}

fn attack_nearest_enemy(game: &mut Game, wpn: &mut Weapon, u_id: UnitID) {
    let is_structure = game.units.is_structure(u_id);
    match kdtp::get_nearest_enemy(game, wpn, u_id) {
        Some(t_id) => {
            wpn.set_target_id(Some(game.units.new_unit_target(t_id)));
            attack_target(game, wpn, u_id, t_id);
        }
        None => {
            // Return weapon to resting position
            let unit_facing = game.units.facing(u_id);
            let wpn_facing = wpn.facing();
            let turn_rate = wpn.turn_rate();
            let wpn_lock_angle = wpn.lock_offset();

            wpn.set_target_id(None);

            if !is_structure {
                wpn.set_facing(mv::turn_towards(wpn_facing, unit_facing + wpn_lock_angle, turn_rate));
            }
        }
    }
}

fn attack_target(game: &mut Game, wpn: &mut Weapon, u_id: UnitID, t_id: UnitID) {
    match wpn.attack() {
        &Attack::Missile(Ok(missile_type)) => {
            turn_towards_target_and_attempt_to_shoot(game, missile_type, wpn, u_id, t_id);
        }
        &Attack::Melee(_) => unimplemented!(),
        &Attack::Suicide(_) => unimplemented!(),
        &Attack::Laser(_) => unimplemented!(),
        &Attack::Bomb(_) => unimplemented!(),
        &Attack::LaserBomb(_) => unimplemented!(),
        _ => panic!("YOU NEED TO IMPLEMENT MORE ATTACKS! {:?}", wpn.attack()),
    }
}

fn turn_towards_target_and_attempt_to_smack(game: &mut Game, damage: Damage, wpn: &mut Weapon, u_id: UnitID, t_id: UnitID) {
    let enemy_xy = game.units.xy(t_id);
    let on_target = turn_weapon_to_point(game, wpn, u_id, enemy_xy);

    if on_target && weapon_is_ready_to_fire(wpn) {
        heatup_weapon(game, wpn);
        match damage {
            Damage::Single(amount) => {
                unit::damage_unit(game, t_id, amount);
            }
            Damage::Splash(amount, radius) => {
                let enemies = kdtp::enemies_in_splash_radius_of_point(game, u_id, wpn, enemy_xy, radius);

                for enemy in enemies {
                    if let Some(id) = game.units.target_id(enemy.target) {
                        unit::damage_unit(game, id, amount);
                    }
                }
            }
        }
    }
}

fn turn_towards_target_and_attempt_to_shoot(game: &mut Game, missile_type: MissileTypeID, wpn: &mut Weapon, u_id: UnitID, t_id: UnitID) {
    let target_facing = game.units.facing(t_id);
    let target_speed = game.units.speed(t_id);
    let missile_speed = wpn.missile_speed();
    let target_xy = game.units.xy(t_id);
    let vec_xy = mv::move_in_direction(0.0, 0.0, target_speed, target_facing);
    let firing_offset = get_firing_offset_position(game, wpn, u_id);

    if let Some(xy) = mv::intercept_point(target_xy, firing_offset, vec_xy, missile_speed) {
        let on_target = turn_weapon_to_point(game, wpn, u_id, xy);

        if on_target {
            fire_missile_salvo_at_target(game, missile_type, wpn, u_id, t_id);
        }
    }
}

fn weapon_is_ready_to_fire(wpn: &Weapon) -> bool {
    let cooldown = wpn.cooldown();
    let salvo_cooldown = wpn.salvo_cooldown();
    let salvo = wpn.salvo();
    let salvo_size = wpn.salvo_size();

    cooldown <= 0.0 || (salvo_cooldown <= 0.0 && salvo < salvo_size)
}

fn heatup_weapon(game: &mut Game, wpn: &mut Weapon) -> usize {
    if wpn.cooldown() <= 0.0 {
        let cooldown = wpn.cooldown();
        let fire_rate = wpn.fire_rate();
        wpn.set_cooldown(cooldown + game.fps() * fire_rate);
        wpn.set_salvo(0);
        wpn.set_salvo_cooldown(0.0);
    }

    let salvo = wpn.salvo();
    wpn.set_salvo(salvo + 1);
    let salvo_cooldown = wpn.salvo_cooldown();
    let salvo_fire_rate = wpn.salvo_fire_rate();
    wpn.set_salvo_cooldown(salvo_cooldown + game.fps() * salvo_fire_rate);
    salvo
}

fn cooldown_weapon(wpn: &mut Weapon) {
    let cooldown = wpn.cooldown();
    let salvo_cooldown = wpn.salvo_cooldown();

    if salvo_cooldown > 0.0 {
        wpn.set_salvo_cooldown(salvo_cooldown - 1.0);
    }

    if cooldown > 0.0 {
        wpn.set_cooldown(cooldown - 1.0);
    }
}

fn fire_missile_salvo_at_target(game: &mut Game, missile_type: MissileTypeID, wpn: &mut Weapon, u_id: UnitID, t_id: UnitID) {
    if weapon_is_ready_to_fire(wpn) {
        let salvo = heatup_weapon(game, wpn);
        let wpn_facing = wpn.facing();
        let alternating = wpn.alternating();
        let wpn_target_type = wpn.target_type();
        let team = game.units.team(u_id);
        let fps = game.fps();

        let range = if alternating {
            salvo..salvo + 1
        }
        else {
            0..wpn.barrels()
        };

        for barrel in range {
            let fire_offset = get_barrel_firing_offset(game, wpn, u_id, barrel);
            for _ in 0..wpn.pellet_count() {
                if let Some(m_id) = game.missiles.make(fps, missile_type) {
                    let random_offset = game.rng.gen_range(-wpn.pellet_spread(), wpn.pellet_spread());
                    game.missiles.set_team(m_id, team);
                    game.missiles.set_target_type(m_id, wpn_target_type);
                    game.missiles.set_target(m_id, Target::Unit(game.units.new_unit_target(t_id)));
                    game.missiles.set_facing(m_id, wpn_facing + mv::normalize(random_offset));
                    game.missiles.set_xy(m_id, fire_offset);
                }
            }
        }
    }
}

fn get_barrel_firing_offset(game: &Game, wpn: &mut Weapon, u_id: UnitID, barrel: usize) -> (f64,f64) {
    let wpn_facing = wpn.facing();
    let wpn_fire_offset = wpn.firing_offset();
    let barrels = wpn.barrels();
    let spacing = wpn.barrel_spacing();
    let barrel_off = 0.0 - spacing / 2.0 * (barrels - 1) as f64 + barrel as f64 * spacing;

    let (wx,wy) = get_weapon_position(game, wpn, u_id);
    let (bx,by) = mv::rotate_point((wpn_fire_offset, barrel_off), wpn_facing);
    (wx + bx, wy + by)
}

fn get_weapon_position(game: &Game, wpn: &mut Weapon, u_id: UnitID) -> (f64, f64) {
    let facing = game.units.facing(u_id);
    let (ux,uy) = game.units.xy(u_id);
    let xy_off = wpn.xy_offset();

    let (rx,ry) = mv::rotate_point(xy_off, facing);
    (ux + rx, uy + ry)
}

// Get the position at the end of the guns barrel
fn get_firing_offset_position(game: &Game, wpn: &mut Weapon, u_id: UnitID) -> (f64, f64) {
    let (x, y) = get_weapon_position(game, wpn, u_id);
    let wpn_facing = wpn.facing();
    let wpn_fire_offset = wpn.firing_offset();

    mv::move_in_direction(x, y, wpn_fire_offset, wpn_facing)
}

fn turn_weapon_to_point(game: &mut Game, wpn: &mut Weapon, u_id: UnitID, (x, y): (f64, f64)) -> bool {
    let (wpn_x, wpn_y) = get_weapon_position(game, wpn, u_id);
    let dx = x - wpn_x;
    let dy = y - wpn_y;
    let wpn_facing = wpn.facing();
    let angle_to_enemy = mv::new(dx, dy);
    let wpn_turn_rate = wpn.turn_rate();

    if mv::distance(wpn_facing, angle_to_enemy) <= wpn_turn_rate {
        wpn.set_facing(angle_to_enemy);
        true
    } else {
        wpn.set_facing(mv::turn_towards(wpn_facing, angle_to_enemy, wpn_turn_rate));
        false
    }
}

pub fn target_in_range(game: &Game, u_id: UnitID, t_id: UnitID, range: f64) -> bool {
    let radius = game.units.radius(u_id);
    let target_radius = game.units.radius(t_id);
    let total_range = range + radius + target_radius;
    let (xa, ya) = game.units.xy(u_id);
    let (xb, yb) = game.units.xy(t_id);
    let dx = xa - xb;
    let dy = ya - yb;
    let team = game.units.team(u_id);
    let is_visible = match game.teams.visible[team][t_id] {
        Visibility::None => false,
        _ => true,
    };

    is_visible && (dx * dx + dy * dy) <= (total_range * total_range)
}
