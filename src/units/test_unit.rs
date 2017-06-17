use data::units::ProtoUnit;
use data::weapons::ProtoWeapon;
use data::missiles::ProtoMissile;
use std::rc::Rc;
use std::collections::HashSet;
use std::f64::consts::PI;
use units::weapon_list as wl;
use units::missile_list as ml;
use data::aliases::*;
use libs::movement as mv;

pub fn prototype() -> ProtoUnit {
    //let mut target_type = TargetType::new();
    ProtoUnit {
        name: "Test Unit",
        radius: 0.64,
        collision_radius: 0.96,
        collision_ratio: 0.625,
        collision_resist: 0.8,
        width_and_height: None,
        weight: 1.0,
        top_speed: 3.0,
        acceleration: 1.5,
        deceleration: 4.5,
        turn_rate: PI / 1.5,
        health_regen: 0.0,
        max_health: 125.0,
        build_cost: 100.0,
        energy_cost: 100.0,
        prime_cost: 100.0,
        prime_output: 0.05,
        energy_output: 0.05,
        build_rate: 5.0,
        build_range: 3.0,
        build_roster: Rc::new(HashSet::new()),
        weapons: vec![wl::id(wl::WeaponType::TestUnit)],
        sight_range: 12.0,
        radar_range: 16.0,
        engagement_range: 12.0,
        target_type: TargetType::new().set_ground(),
        move_type: MoveType::Ground,
        collision_type: TargetType::new().set_ground(),
        is_structure: false,
        is_automatic: false,
    }
}

pub fn wpn_proto() -> ProtoWeapon {
    ProtoWeapon {
        name: "Test Weapon",
        attack_type: AttackType::MissileAttack(ml::id(ml::MissileType::TestMissile)),
        x_offset: 0.0,
        y_offset: 0.0,
        turn_rate: PI,
        lock_offset: mv::normalize(0.0),
        firing_arc: PI,
        range: 8.0,
        firing_offset: 0.75,
        fire_rate: 1000,
        salvo_size: 1,
        salvo_fire_rate: 0,
        pellet_count: 1,
        pellet_spacing: 0.0,
        random_offset: 0.0,
        target_type: TargetType::new().set_ground(),
        missile_speed: 24.0,
    }
}

pub fn missile_proto() -> ProtoMissile {
    ProtoMissile {
        name: "Test Missile",
        speed: 24.0,
        max_travel_dist: 18.0,
        damage: Damage::Single(15.0),
        damage_type: DamageType::SmallBlast,
        turn_rate: 0.0,
    }
}
