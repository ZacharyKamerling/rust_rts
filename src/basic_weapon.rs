//use movement as mv;
use data::game::{Game};
use data::kdt_point::{KDTPoint};
use std::f32;
use movement as mv;
use data::aliases::*;

pub fn fire_orders(game: &mut Game, w_id: WeaponID, u_id: UnitID) {
    match game.units.orders[u_id].front() {
        Some(&order) => {
            match order {
                Order::AttackMove(mg_id) => {
                    match game.weapons.target_id[w_id] {
                        Some(t_id) => {
                            if target_in_range(game, w_id, u_id, t_id) {
                                fire_at_target(game, w_id, u_id, t_id);
                            }
                            else {
                                fire_at_nearest_enemy(game, w_id, u_id);
                            }
                        }
                        None => {
                            fire_at_nearest_enemy(game, w_id, u_id);
                        }
                    }
                }
                Order::AttackTarget(_) => {

                }
                Order::Move(_) => {

                }
            }
        }
        None => {
            fire_at_nearest_enemy(game, w_id, u_id);
        }
    }
}

fn fire_at_nearest_enemy(game: &mut Game, w_id: WeaponID, u_id: UnitID) {
    match get_nearest_enemy(game, w_id, u_id) {
        Some(t_id) => {
            game.weapons.target_id[w_id] = Some(t_id);
            fire_at_target(game, w_id, u_id, t_id);
        }
        None => {
            game.weapons.target_id[w_id] = None;
        }
    }
}

fn fire_at_target(game: &mut Game, w_id: WeaponID, u_id: UnitID, t_id: UnitID) {
    match game.weapons.attack_type[w_id] {
        AttackType::MissileAttack(missile_type) => {

        }
        AttackType::MeleeAttack(damage) => {

        }
        AttackType::LaserAttack(damage) => {

        }
    }
}

fn target_in_range(game: &mut Game, w_id: WeaponID, u_id: UnitID, t_id: UnitID) -> bool {
    let range = game.weapons.range[w_id];
    let radius = game.units.radius[u_id];
    let target_radius = game.units.radius[t_id];
    let total_range = range + radius + target_radius;
    let xa = game.units.x[u_id];
    let ya = game.units.y[u_id];
    let xb = game.units.x[t_id];
    let yb = game.units.y[t_id];
    let dx = xa - xb;
    let dy = ya - yb;

    (dx * dx + dy * dy) <= (total_range * total_range)
}

fn get_nearest_enemy(game: &Game, w_id: WeaponID, u_id: UnitID) -> Option<UnitID> {
    let range = game.weapons.range[w_id];
    let radius = game.units.radius[u_id];
    let enemies = enemies_in_range(game, range + radius, w_id, u_id);

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

            if enemy_dist < nearest_dist && target_in_firing_arc(game, w_id, u_id, enemy.id) {
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

fn enemies_in_range(game: &Game, r: f32, w_id: WeaponID, u_id: UnitID) -> Vec<KDTPoint> {
    let x = game.units.x[u_id];
    let y = game.units.y[u_id];
    let team = game.units.team[u_id];
    let fliers = game.weapons.hits_air[w_id];
    let ground = game.weapons.hits_ground[w_id];
    let structures = game.weapons.hits_structures[w_id];

    let is_collider = |b: &KDTPoint| {
        let is_flier = game.units.is_flying[b.id];
        let is_ground = game.units.is_ground[b.id];
        let is_structure = game.units.is_structure[b.id];

        (b.team != team) &&
        game.teams.visible[team][b.id] &&
        (is_flier == fliers || is_ground == ground || is_structure == structures) &&
        {
            let dx = b.x - x;
            let dy = b.y - y;
            let dr = b.radius + r;
            (dx * dx) + (dy * dy) <= dr * dr
        }
    };

    game.kdt.in_range(&is_collider, &[(x,r),(y,r)])
}

fn target_in_firing_arc(game: &Game, w_id: WeaponID, u_id: UnitID, t_id: UnitID) -> bool {
    let dx = game.units.x[t_id] - game.units.x[u_id];
    let dy = game.units.y[t_id] - game.units.y[u_id];

    let angle_to_enemy = mv::new(dx, dy);
    let lock_angle = game.weapons.lock_offset[w_id] + game.units.facing[u_id];
    let firing_arc = game.weapons.firing_arc[w_id];

    mv::distance(angle_to_enemy, lock_angle) <= firing_arc
}