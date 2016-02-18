use data::units::Unit;
use std::rc::Rc;
use std::collections::{HashSet};
use std::f32::consts::{PI};
use data::aliases::*;

pub fn prototype() -> Unit {
    Unit {
        name:               "Test Unit",
        radius:             0.5,
        weight:             1.0,
        top_speed:          3.0,
        acceleration:       2.0,
        deceleration:       2.0,
        turn_rate:          PI / 4.0,
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
        target_type:        TargetType::Ground,
        is_automatic:       false,
    }
}