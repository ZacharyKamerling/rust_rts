use units;
use data::units::Unit;
use data::aliases::*;

pub fn list() -> (VecUID<UnitTypeID, Unit>, VecUID<MissileTypeID, Missile>) {
    let mut unit_list = Vec::new();
    let mut misl_list = Vec::new();
    let mut unit_uids = UIDMapping::new(256);
    let mut misl_uids = UIDMapping::new(256);
    unit_list.push(units::medium1::prototype());
    unit_list.push(units::artillery1::prototype());
    unit_list.push(units::extractor1::prototype());
    misl_list.push(units::fast1::missile_proto());
    misl_list.push(units::medium1::missile_proto());
    misl_list.push(units::artillery1::missile_proto());

    for proto in unit_list.iter_mut() {
        match unit_uids.name_to_id(proto.name().clone()) {
            Some(unit_type_id) => {
                proto.set_unit_type(Some(unit_type_id));
                println!("Name: {:?}, ID: {:?}", proto.name(), unit_type_id);
            }
            None => {
                panic!("No more ids available");
            }
        }
    }

    for proto in misl_list.iter_mut() {
        match misl_uids.name_to_id(proto.name().clone()) {
            Some(misl_type_id) => {
                proto.set_missile_type_id(Some(misl_type_id));
                println!("Name: {:?}, ID: {:?}", proto.name(), misl_type_id);
            }
            None => {
                panic!("No more ids available");
            }
        }
    }

    for unit in unit_list.iter_mut() {
        for wpn in unit.mut_weapons().iter_mut() {
            match wpn.attack_type().clone() {
                Attack::Missile(Err(ref missile_name)) => {
                    match misl_uids.name_to_id(missile_name.clone()) {
                        Some(misl_type_id) => {
                            *wpn.mut_attack_type() = Attack::Missile(Ok(misl_type_id))
                        }
                        None => {
                            panic!("No more ids available");
                        }
                    }
                }
                _ => (),
            }
        }
    }

    let mut unit_vec = VecUID::full_vec(256, Unit::new());
    for i in 0..unit_list.len() {
        let utid = unsafe { UnitTypeID::usize_wrap(i) };
        unit_vec[utid] = unit_list[i].clone();
    }

    let mut misl_vec = VecUID::full_vec(256, Missile::new());
    for i in 0..misl_list.len() {
        let utid = unsafe { MissileTypeID::usize_wrap(i) };
        misl_vec[utid] = misl_list[i].clone();
    }

    (unit_vec, misl_vec)
}