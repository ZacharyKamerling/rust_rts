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
    ProtoUnit {
        name: "Test Structure",
        radius: 2.4,
        collision_radius: 0.0,
        collision_ratio: 0.0,
        collision_resist: 0.0,
        width_and_height: Some((4, 4)),
        weight: 1.0,
        top_speed: 0.0,
        acceleration: 0.0,
        deceleration: 0.0,
        turn_rate: 0.0,
        health_regen: 0.0,
        max_health: 1000.0,
        build_cost: 1500.0,
        energy_cost: 1500.0,
        prime_cost: 1500.0,
        prime_output: 0.1,
        energy_output: 0.1,
        build_rate: 0.0,
        build_range: 0.0,
        build_roster: Rc::new(HashSet::new()),
        weapons: vec![wl::id(wl::WeaponType::TestStructure)],
        sight_range: 12.0,
        radar_range: 0.0,
        engagement_range: 0.0,
        target_type: TargetType::new().set_ground().set_structure(),
        collision_type: TargetType::new(),
        move_type: MoveType::None,
        is_structure: true,
        is_automatic: false,
    }
}

pub fn wpn_proto() -> ProtoWeapon {
    ProtoWeapon {
        name: "Test Weapon",
        attack_type: AttackType::MissileAttack(ml::id(ml::MissileType::TestStructure)),
        x_offset: 0.0,
        y_offset: 0.0,
        turn_rate: PI / 8.0,
        lock_offset: mv::normalize(0.0),
        firing_arc: PI,
        range: 50.0,
        firing_offset: 3.2,
        fire_rate: 4000,
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
        max_travel_dist: 60.0,
        damage: Damage::Single(150.0),
        damage_type: DamageType::SmallBlast,
        turn_rate: 0.0,
    }
}
