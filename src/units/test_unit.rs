use data::units::Unit;
use std::rc::Rc;
use std::collections::{HashSet};

pub fn prototype() -> Unit {
    Unit {
        name:               "Test Unit",
        radius:             0.55,
        weight:             1.0,
        top_speed:          10.0,
        acceleration:       0.5,
        deceleration:       0.5,
        turn_rate:          3.14,
        health_regen:       0.5,
        max_health:         100.0,
        progress_required:  100.0,
        build_rate:         1.0,
        build_range:        1.0,
        build_roster:       Rc::new(HashSet::new()),
        weapons:            Vec::new(),
        sight_range:        12.0,
        radar_range:        16.0,
        active_range:       8.0,
        is_ground:          true,
        is_flying:          false,
        is_structure:       false,
        is_automatic:       false,
    }
}