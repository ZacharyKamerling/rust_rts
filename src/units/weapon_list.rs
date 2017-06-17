use units;
use data::weapons::ProtoWeapon;
use std::f64::consts::PI;
use libs::movement as mv;
use data::aliases::*;

#[derive(Clone, Copy)]
pub enum WeaponType {
    TestUnit,
    TestStructure,
}

pub fn id(wpn_type: WeaponType) -> WeaponTypeID {
    unsafe { WeaponTypeID::usize_wrap(wpn_type as usize) }
}

pub fn list() -> VecUID<WeaponTypeID, ProtoWeapon> {
    let mut vec = VecUID::full_vec(256, wpn_proto());

    vec[id(WeaponType::TestUnit)] = units::test_unit::wpn_proto();
    vec[id(WeaponType::TestStructure)] = units::test_structure::wpn_proto();

    vec
}

fn wpn_proto() -> ProtoWeapon {
    ProtoWeapon {
        name: "Test Weapon",
        attack_type: AttackType::MeleeAttack(Damage::Single(0.0)),
        x_offset: 0.0,
        y_offset: 0.0,
        turn_rate: PI,
        lock_offset: mv::normalize(0.0),
        firing_arc: PI,
        range: 0.0,
        firing_offset: 0.0,
        fire_rate: 1000,
        salvo_size: 1,
        salvo_fire_rate: 0,
        pellet_count: 1,
        pellet_spacing: 0.0,
        random_offset: 0.0,
        target_type: TargetType::new(),
        missile_speed: 0.0,
    }
}
