use data::units::ProtoUnit;
use data::weapons::Weapon;
use data::missiles::Missile;
use std::rc::Rc;
use std::collections::{HashSet};
use std::f32::consts::{PI};
use data::aliases::*;
use movement as mv;

pub fn prototype() -> ProtoUnit {
    ProtoUnit {
        name:               "Test Structure",
        radius:             1.5,
        collision_radius:   1.5,
        width_and_height:   Some((3,3)),
        weight:             1.0,
        top_speed:          0.0,
        acceleration:       0.0,
        deceleration:       0.0,
        turn_rate:          0.0,
        health_regen:       0.0,
        max_health:         1000.0,
        progress_required:  1000.0,
        build_rate:         0.0,
        build_range:        0.0,
        build_roster:       Rc::new(HashSet::new()),
        weapons:            vec!(),
        sight_range:        12.0,
        radar_range:        0.0,
        engagement_range:   0.0,
        target_type:        TargetType::Ground,
        is_structure:       true,
        is_automatic:       false,
    }
}