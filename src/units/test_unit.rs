use data::units::ProtoUnit;
use data::weapons::Weapon;
use data::missiles::Missile;
use std::rc::Rc;
use std::collections::{HashSet};
use std::f32::consts::{PI};
use data::aliases::*;
use libs::movement as mv;

pub fn prototype() -> ProtoUnit {
    ProtoUnit {
        name:               "Test Unit",
        radius:             0.4,
        collision_radius:   0.6,
        collision_ratio:    0.625,
        collision_resist:   0.8,
        width_and_height:   None,
        weight:             1.0,
        top_speed:          3.0,
        acceleration:       1.5,
        deceleration:       4.5,
        turn_rate:          PI / 1.5,
        health_regen:       0.0,
        max_health:         125.0,
        progress_required:  125.0,
        build_rate:         1.0,
        build_range:        3.0,
        build_roster:       Rc::new(HashSet::new()),
        weapons:            vec!(0),
        sight_range:        12.0,
        radar_range:        16.0,
        engagement_range:   12.0,
        target_type:        TargetType::Ground,
        is_structure:       false,
        is_automatic:       false,
    }
}

pub fn wpn_proto() -> Weapon {
    Weapon {
        name:               "Test Weapon",
        attack_type:        AttackType::MissileAttack(0),
        x_offset:           0.0,
        y_offset:           0.0,
        turn_rate:          PI,
        lock_offset:        mv::normalize(0.0),
        firing_arc:         PI,
        range:              8.0,
        firing_offset:      0.0,
        fire_rate:          1000,
        salvo_size:         1,
        salvo_fire_rate:    0,
        pellet_count:       1,
        random_offset:      0.0,
        hits_air:           false,
        hits_ground:        true,
        hits_structure:     true,
        missile_speed:      12.0,
    }
}

pub fn missile_proto() -> Missile {
    Missile {
        name:               "Test Missile",
        speed:              12.0,
        max_travel_dist:    18.0,
        damage:             Damage::Single(15.0),
        damage_type:        DamageType::SmallBlast,
        turn_rate:          0.0,
    }
}