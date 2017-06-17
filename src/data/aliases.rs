/*
 This module consists of newtypes/wrappers and many enum types that didn't deserve their own module.
*/

use data::move_groups::{MoveGroup};
use data::build_groups::{BuildGroup};
use data::units::UnitTarget;

pub use data::uid_types::*;
pub use data::target_type::*;
pub use data::move_stats::*;

pub type SoulID             = usize;
pub type AnimID             = usize;
pub type ProducerID         = usize;
pub type AbilityID          = usize;
pub type ProducerTypeID     = usize;
pub type Milliseconds       = isize;

pub const FPS: usize = 10;

pub enum Visibility {
    None,
    Full(usize),
    Partial(usize),
    RadarBlip(usize),
}

#[derive(Clone,Copy)]
pub enum Damage {
    Single(f64),
    Splash(f64, f64),
}

#[derive(Clone,Copy)]
pub enum DamageType {
    Physical,
    SmallBlast,
    Laser,
}

/*
Potential things a weapon can aim for.
*/
#[derive(Clone,Copy)]
pub enum Target {
    Point(f64,f64),
    Unit(UnitTarget),
    None,
}

/*
Different ways a unit can move.
*/
enum_from_primitive! {
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum MoveType {
    None = 143,
    Ground = 134,
    Hover = 137,
    Water = 140,
    Air = 99999,
}
}

#[derive(Clone,Copy,Debug)]
pub struct BuildCharge {
    pub prime_cost: f64,
    pub energy_cost: f64,
    pub build_cost: f64,
    pub current_charges: usize,
    pub max_charges: usize,
}

#[derive(Clone,Copy)]
pub enum AttackType {
    // A homing or non-homing projectile
    // that may take more than 1 frame to hit its target.
    MissileAttack(MissileTypeID),
    // An attack that creates no missile
    MeleeAttack(Damage),
    // An attack that hits instantly
    LaserAttack(Damage),
    // An attack where the unit doesn't slow down when it engages
    BombAttack(MissileTypeID),
    // Same as bomb but with lasers
    LaserBombAttack(Damage),
}

#[derive(Clone,Copy)]
pub enum UnitEvent {
    UnitSteps(UnitID),
    UnitDies(UnitID, UnitTarget), // Killed, Killer
    UnitIsDamaged(UnitID, UnitTarget, Damage), // Victim, Attacker, Damage
    UnitDealsDamage(UnitTarget, UnitID, Damage), // Attacker, Victim, Damage
    UnitUsesAbility(UnitID, AbilityID, Target),
    UnitEndsAbility(UnitID, AbilityID, Target),
}

#[derive(Clone,Debug)]
pub struct Order {
    pub order_id:   OrderID,
    pub order_type: OrderType,
}

#[derive(Clone,Debug)]
pub enum OrderType {
    Move(MoveGroup),
    AttackMove(MoveGroup),
    AttackTarget(MoveGroup,UnitTarget),
    Build(BuildGroup),
}

enum_from_primitive! {
#[derive(Clone,Copy)]
pub enum QueueOrder {
    Prepend,
    Append,
    Replace,
}
}

#[derive(Clone,Copy)]
pub enum ClientMessage {
    UnitMove,
    UnitDeath,
    OrderCompleted,
    MissileMove,
    MissileExplode,
    TeamInfo,
    MapInfo,
}

enum_from_primitive! {
#[derive(Clone,Copy)]
pub enum ServerMessage {
    Move,
    AttackMove,
    AttackTarget,
    Build,
    MapInfoRequest,
}
}