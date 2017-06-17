use units;
use data::missiles::ProtoMissile;
use data::aliases::*;

#[derive(Clone, Copy)]
pub enum MissileType {
    TestMissile,
    TestStructure,
}

pub fn id(unit_type: MissileType) -> MissileTypeID {
    unsafe { MissileTypeID::usize_wrap(unit_type as usize) }
}

pub fn list() -> VecUID<MissileTypeID, ProtoMissile> {
    let mut vec = VecUID::full_vec(256, missile_proto());

    vec[id(MissileType::TestMissile)] = units::test_unit::missile_proto();
    vec[id(MissileType::TestStructure)] = units::test_structure::missile_proto();

    vec
}

fn missile_proto() -> ProtoMissile {
    ProtoMissile {
        name: "Test Missile",
        speed: 0.0,
        max_travel_dist: 0.0,
        damage: Damage::Single(0.0),
        damage_type: DamageType::SmallBlast,
        turn_rate: 0.0,
    }
}
