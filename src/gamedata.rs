use std::collections::HashMap;
use std::rc::Rc;

pub enum Target {
    NoTarget,
    GroundTarget((f32,f32)),
    UnitTarget(UnitID),
}

pub enum AttackType {
    MissileAttack(MissileTypeID),
    MeleeAttack(Damage),
    LaserAttack(Damage),
}

pub struct EventHandlers {
    a_unit_dies:            Vec<fn(&mut Game, UnitID) -> ()>,
    a_unit_steps:           Vec<fn(&mut Game, UnitID) -> ()>,
    a_unit_is_attacked:     Vec<fn(&mut Game, UnitID, UnitID) -> ()>,
    a_unit_is_created:      Vec<fn(&mut Game, UnitID, UnitID) -> ()>,
    a_unit_is_damaged:      Vec<fn(&mut Game, UnitID, UnitID, f32) -> ()>,
    a_unit_uses_ability:    Vec<fn(&mut Game, UnitID, AbilityID, Target) -> ()>,
    a_unit_ends_ability:    Vec<fn(&mut Game, UnitID, AbilityID, Target) -> ()>,
}

pub struct Game {
    pub game_rng:                   ThreadRng,
    pub random_offset_gen:          Range<f32>,
    pub event_handlers:             EventHandlers,
    pub kdt:                        KDTree<KDTPoint>,
    pub bytegrid:                   ByteGrid,
    //
    pub blueprints_of_units:        HashMap<String, Unit, ()>,
    pub blueprints_of_weapons:      HashMap<String, Weapon, ()>,
    pub blueprints_of_missiles:     HashMap<String, Missile, ()>,
    //
    pub units:                      Vec<Unit>,
    pub teams:                      Vec<Team>,
    pub weapons:                    Vec<Weapon>,
    pub missiles:                   Vec<Missile>,
}