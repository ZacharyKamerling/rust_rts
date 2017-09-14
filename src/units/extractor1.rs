use data::units::{Unit};
use std::collections::HashSet;
use units::unit_list as ul;
use data::aliases::*;

pub fn prototype() -> Unit {
    let mut unit = Unit::new();

    unit.set_unit_type(ul::id(ul::UnitType::Extractor1));
    unit.set_radius(2.4);
    unit.set_collision_radius(0.0);
    unit.set_collision_ratio(0.0);
    unit.set_collision_resist(0.0);
    unit.set_width_and_height(Some((3, 3)));
    unit.set_weight(1.0);
    unit.set_top_speed(0.0);
    unit.set_acceleration(0.0);
    unit.set_deceleration(0.0);
    unit.set_turn_rate(0.0);
    unit.set_health_regen(0.0);
    unit.set_max_health(150.0);
    unit.set_build_cost(75.0);
    unit.set_prime_cost(75.0);
    unit.set_energy_cost(75.0);
    unit.set_prime_output(2.0);
    unit.set_energy_output(2.0);
    unit.set_build_rate(0.0);
    unit.set_build_range(0.0);
    *unit.mut_build_roster() = HashSet::new();
    *unit.mut_weapons() = vec![];
    unit.set_sight_range(12.0);
    unit.set_radar_range(0.0);
    unit.set_engagement_range(0.0);
    unit.set_target_type(TargetType::new().set_ground().set_structure());
    unit.set_collision_type(TargetType::new());
    unit.set_move_type(MoveType::None);
    unit.set_is_structure(true);

    unit
}