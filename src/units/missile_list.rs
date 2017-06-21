use units;
use data::units::{Missile};
use data::aliases::*;

#[derive(Clone, Copy)]
pub enum MissileType {
    TestMissile,
    TestStructure,
}

pub fn id(unit_type: MissileType) -> MissileTypeID {
    unsafe { MissileTypeID::usize_wrap(unit_type as usize) }
}

pub fn list() -> VecUID<MissileTypeID, Missile> {
    let mut vec = VecUID::full_vec(256, Missile::new());

    vec[id(MissileType::TestMissile)] = units::test_unit::missile_proto();
    vec[id(MissileType::TestStructure)] = units::test_structure::missile_proto();

    vec
}