use units;
use data::units::{Missile};
use data::aliases::*;

#[derive(Clone, Copy)]
pub enum MissileType {
    Fast1,
    Medium1,
    Artillery1,
}

pub fn id(unit_type: MissileType) -> MissileTypeID {
    unsafe { MissileTypeID::usize_wrap(unit_type as usize) }
}

pub fn list() -> VecUID<MissileTypeID, Missile> {
    let mut vec = VecUID::full_vec(256, Missile::new());

    vec[id(MissileType::Fast1)] = units::fast1::missile_proto();
    vec[id(MissileType::Medium1)] = units::medium1::missile_proto();
    vec[id(MissileType::Artillery1)] = units::artillery1::missile_proto();

    vec
}