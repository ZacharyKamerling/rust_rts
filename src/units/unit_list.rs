use units;
use data::units::Unit;
use data::aliases::*;

#[derive(Clone, Copy)]
pub enum UnitType {
    Fast1,
    Medium1,
    Artillery1,
    Extractor1,
}

pub fn id(unit_type: UnitType) -> UnitTypeID {
    unsafe { UnitTypeID::usize_wrap(unit_type as usize) }
}

pub fn list() -> VecUID<UnitTypeID, Unit> {
    let mut vec = VecUID::full_vec(256, Unit::new());

    vec[id(UnitType::Fast1)] = units::fast1::prototype();
    vec[id(UnitType::Medium1)] = units::medium1::prototype();
    vec[id(UnitType::Artillery1)] = units::artillery1::prototype();
    vec[id(UnitType::Extractor1)] = units::extractor1::prototype();

    vec
}