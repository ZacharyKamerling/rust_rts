use data::units::{Unit,Weapon,Missile};
use std::collections::HashSet;
use std::f64::consts::PI;
use units::unit_list as ul;
use units::missile_list as ml;
use data::aliases::*;
use libs::movement as mv;

pub fn prototype() -> Unit {
    let mut unit = Unit::new();

    unit.set_unit_type(ul::id(ul::UnitType::Medium1));
    unit.set_radius(0.64);
    unit.set_collision_radius(0.96);
    unit.set_collision_ratio(0.625);
    unit.set_collision_resist(0.8);
    unit.set_width_and_height(None);
    unit.set_weight(1.0);
    unit.set_top_speed(3.0);
    unit.set_acceleration(1.5);
    unit.set_deceleration(4.5);
    unit.set_turn_rate(PI / 1.5);
    unit.set_health_regen(0.0);
    unit.set_max_health(125.0);
    unit.set_build_cost(100.0);
    unit.set_prime_cost(100.0);
    unit.set_energy_cost(100.0);
    unit.set_prime_output(0.0);
    unit.set_energy_output(0.0);
    unit.set_build_rate(5.0);
    unit.set_build_range(4.0);
    *unit.mut_build_roster() = HashSet::new();
    *unit.mut_weapons() = vec![wpn_proto()];
    unit.set_sight_range(16.0);
    unit.set_radar_range(0.0);
    unit.set_engagement_range(16.0);
    unit.set_target_type(TargetType::new().set_ground());
    unit.set_collision_type(TargetType::new().set_ground());
    unit.set_move_type(MoveType::Ground);
    unit.set_is_structure(false);
    unit.set_is_automatic(false);

    unit
}

fn wpn_proto() -> Weapon {
    let mut wpn = Weapon::new();

    wpn.set_attack_type(Attack::Missile(ml::id(ml::MissileType::Medium1)));
    wpn.set_xy_offset((0.0,0.0));
    wpn.set_turn_rate(PI);
    wpn.set_lock_offset(mv::normalize(0.0));
    wpn.set_firing_arc(PI);
    wpn.set_range(12.0);
    wpn.set_firing_offset(0.75);
    wpn.set_fire_rate(1.0);
    wpn.set_alternating(false);
    wpn.set_barrels(1);
    wpn.set_barrel_spacing(0.0);
    wpn.set_salvo_size(1);
    wpn.set_salvo_fire_rate(0.0);
    wpn.set_pellet_count(1);
    wpn.set_pellet_spread(0.01);
    wpn.set_target_type(TargetType::new().set_ground());
    wpn.set_missile_speed(24.0);

    wpn
}

pub fn missile_proto() -> Missile {
    let mut msl = Missile::new();

    msl.set_speed(24.0);
    msl.set_max_travel_dist(15.0);
    msl.set_damage(Damage::Single(15.0));
    msl.set_damage_type(DamageType::SmallBlast);
    msl.set_turn_rate(0.0);

    msl
}