use units;
use data::units::Unit;
use data::aliases::*;

#[derive(Clone, Copy)]
pub enum UnitType {
    TestUnit,
    TestStructure,
}

pub fn id(unit_type: UnitType) -> UnitTypeID {
    unsafe { UnitTypeID::usize_wrap(unit_type as usize) }
}

pub fn list() -> VecUID<UnitTypeID, Unit> {
    let mut vec = VecUID::full_vec(256, Unit::new());

    vec[id(UnitType::TestUnit)] = units::test_unit::prototype();
    vec[id(UnitType::TestStructure)] = units::test_structure::prototype();

    vec
}