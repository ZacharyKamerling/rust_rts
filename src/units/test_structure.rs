use data::units::{Unit,Weapon,Missile};
use std::rc::Rc;
use std::collections::HashSet;
use std::f64::consts::PI;
use units::unit_list as ul;
use units::missile_list as ml;
use data::aliases::*;
use libs::movement as mv;

pub fn prototype() -> Unit {
    let mut unit = Unit::new();

    unit.set_unit_type(ul::id(ul::UnitType::TestStructure));
    unit.set_radius(2.4);
    unit.set_collision_radius(0.0);
    unit.set_collision_ratio(0.0);
    unit.set_collision_resist(0.0);
    unit.set_width_and_height(Some((4, 4)));
    unit.set_weight(1.0);
    unit.set_top_speed(0.0);
    unit.set_acceleration(0.0);
    unit.set_deceleration(0.0);
    unit.set_turn_rate(0.0);
    unit.set_health_regen(0.0);
    unit.set_max_health(2500.0);
    unit.set_build_cost(1500.0);
    unit.set_prime_cost(1500.0);
    unit.set_energy_cost(1500.0);
    unit.set_prime_output(0.0);
    unit.set_energy_output(0.0);
    unit.set_build_rate(0.0);
    unit.set_build_range(0.0);
    *unit.mut_build_roster() = Rc::new(HashSet::new());
    *unit.mut_weapons() = vec![wpn_proto()];
    unit.set_sight_range(12.0);
    unit.set_radar_range(0.0);
    unit.set_engagement_range(0.0);
    unit.set_target_type(TargetType::new().set_ground().set_structure());
    unit.set_collision_type(TargetType::new());
    unit.set_move_type(MoveType::None);
    unit.set_is_structure(true);

    unit
}

fn wpn_proto() -> Weapon {
    let mut wpn = Weapon::new();

    wpn.set_attack_type(AttackType::MissileAttack(ml::id(ml::MissileType::TestStructure)));
    wpn.set_xy_offset((0.0,0.0));
    wpn.set_turn_rate(PI / 8.0);
    wpn.set_lock_offset(mv::normalize(0.0));
    wpn.set_firing_arc(PI);
    wpn.set_range(50.0);
    wpn.set_firing_offset(3.2);
    wpn.set_fire_rate(4.0);
    wpn.set_salvo_size(1);
    wpn.set_salvo_fire_rate(0.0);
    wpn.set_pellet_count(1);
    wpn.set_pellet_spacing(0.3);
    wpn.set_random_offset(0.05);
    wpn.set_target_type(TargetType::new().set_ground());
    wpn.set_missile_speed(24.0);

    wpn
}

pub fn missile_proto() -> Missile {
    let mut msl = Missile::new();

    msl.set_speed(24.0);
    msl.set_max_travel_dist(60.0);
    msl.set_damage(Damage::Single(150.0));
    msl.set_damage_type(DamageType::SmallBlast);
    msl.set_turn_rate(0.0);

    msl
}