use data::move_groups::{MoveGroupID};

pub type Damage             = f32;
pub type UnitID             = usize;
pub type TeamID             = usize;
pub type AnimID             = usize;
pub type WeaponID           = usize;
pub type ProducerID         = usize;
pub type AbilityID          = usize;
pub type MissileID          = usize;
pub type UnitTypeID         = usize;
pub type WeaponTypeID       = usize;
pub type MissileTypeID      = usize;
pub type ProducerTypeID     = usize;

#[derive(Clone,Copy)]
pub enum Target {
    GroundTarget((f32,f32)),
    UnitTarget(UnitID),
    NoTarget
}

#[derive(Clone,Copy)]
pub enum AttackType {
    MissileAttack(MissileTypeID),
    MeleeAttack(Damage),
    LaserAttack(Damage),
}

#[derive(Clone,Copy)]
pub enum Flag {
    IsUnit,
    IsStructure,
    IsMobile,
    IsGround,
    IsFlying,
    IsMissile,
    IsAutomated,
    IsTransportable,
}

#[derive(Clone,Copy)]
pub enum UnitEvent {
    UnitSteps(UnitID),
    UnitDies(UnitID, UnitID),
    UnitKills(UnitID, UnitID),
    UnitIsDamaged(UnitID, UnitID, Damage),
    UnitDealsDamage(UnitID, UnitID, Damage),
    UnitUsesAbility(UnitID, AbilityID, Target),
    UnitEndsAbility(UnitID, AbilityID, Target),
}

#[derive(Clone,Copy,Debug)]
pub enum Order {
    Move(f32,f32,MoveGroupID)
}