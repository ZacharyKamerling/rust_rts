use units;
use data::units::Unit;
use data::aliases::*;

#[derive(Clone, Copy)]
pub enum UnitType {
    Medium1,
    Artillery1,
    Extractor1,
    Fast1,
}

pub fn id(unit_type: UnitType) -> UnitTypeID {
    unsafe { UnitTypeID::usize_wrap(unit_type as usize) }
}

pub fn list() -> VecUID<UnitTypeID, Unit> {
    let mut unit_list = Vec::new();
    //let mut misl_list = Vec::new();
    let mut unit_uids = UIDMapping::new(256);
    //let mut misl_uids = UIDMapping::new(256);
    unit_list.push(units::medium1::prototype());
    unit_list.push(units::artillery1::prototype());
    unit_list.push(units::extractor1::prototype());

    for proto in unit_list.iter_mut() {
        match unit_uids.name_to_id(proto.name().clone()) {
            Some(unit_type_id) => {
                proto.set_unit_type(Some(unit_type_id));
            }
            None => {
                panic!("No more ids available");
            }
        }
    }

    let mut vec_uid = VecUID::full_vec(256, Unit::new());
    for i in 0..unit_list.len() {
        let utid = unsafe { UnitTypeID::usize_wrap(i) };
        vec_uid[utid] = unit_list[i].clone();
    }

    vec_uid
}