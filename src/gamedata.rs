use std::collections::HashMap;

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
    pub producers:                  Vec<Producer>,
}

pub struct Unit {
    x: f32,
    y: f32,
    radius: f32,
    weight: f32,
    speed: f32,
    top_speed: f32,
    acceleration: f32,
    deceleration: f32,
    facing: f32,
    turn_rate: f32,
    path: Vec<(isize,isize)>,
    width_and_height: Option<(isize,isize)>,
    progress: f32,
    progress_required: f32,
    build_rate: f32,
    build_range: f32,
    build_roster: Rc<HashSet<String>>,
    in_production: Option<usize>,
    for_sale: UnitStock,
}

pub struct UnitStock {
    cooldown:  f32,
    progress:  f32,
    stock:     usize,
    max_stock: usize,
}