/*
 This module consists of newtypes/wrappers and many enum types that didn't deserve their own module.
*/

use data::move_groups::MoveGroup;
use data::build_groups::BuildGroup;
use data::units::UnitTarget;
use std::rc::Rc;
use std::collections::HashSet;
use std::collections::vec_deque::VecDeque;

pub use data::uid_types::*;
pub use data::target_type::*;
pub use data::move_stats::*;
pub use data::units::{Missile};

pub type AnimID = usize;
pub type ProducerID = usize;
pub type AbilityID = usize;
pub type ProducerTypeID = usize;
pub type Milliseconds = isize;

pub const FPS: usize = 10;

#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    None,
    Full(f64),
    Partial(f64),
    RadarBlip(f64),
}

#[derive(Clone, Copy, Debug)]
pub enum Damage {
    Single(f64),
    Splash(f64, f64),
}

#[derive(Clone, Copy, Debug)]
pub enum DamageType {
    Physical,
    SmallBlast,
    Laser,
}

/*
Potential things a weapon can aim for.
*/
#[derive(Clone, Copy, Debug)]
pub enum Target {
    Point(f64, f64),
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

#[derive(Clone, Copy, Debug)]
pub enum Attack {
    // A homing or non-homing projectile
    // that may take more than 1 frame to hit its target.
    Missile(MissileTypeID),
    // An attack that creates no missile
    Melee(Damage),
    // A suicidal attack that creates no missile
    Suicide(Damage),
    // An attack that hits instantly
    Laser(Damage),
    // An attack where the unit doesn't slow down when it engages
    Bomb(MissileTypeID),
    // Same as bomb but with lasers
    LaserBomb(Damage),
}

#[derive(Clone, Copy, Debug)]
pub enum UnitEvent {
    UnitSteps(UnitID),
    UnitDies(UnitID, UnitTarget), // Killed, Killer
    UnitIsDamaged(UnitID, UnitTarget, Damage), // Victim, Attacker, Damage
    UnitDealsDamage(UnitTarget, UnitID, Damage), // Attacker, Victim, Damage
    UnitUsesAbility(UnitID, AbilityID, Target),
    UnitEndsAbility(UnitID, AbilityID, Target),
}

#[derive(Clone, Debug)]
pub struct Order {
    pub order_id: OrderID,
    pub order_type: OrderType,
}

#[derive(Clone, Debug)]
pub enum OrderType {
    Move(MoveGroup),
    AttackMove(MoveGroup),
    AttackTarget(MoveGroup, UnitTarget),
    Build(BuildGroup),
    Assist(UnitTarget),
}

enum_from_primitive! {
#[derive(Clone,Copy, Debug)]
pub enum QueueOrder {
    Prepend,
    Append,
    Replace,
}
}

#[derive(Clone, Copy, Debug)]
pub enum ClientMessage {
    UnitMove,
    UnitDeath,
    OrderCompleted,
    MeleeSmack,
    MissileMove,
    MissileExplode,
    Construction,
    TeamInfo,
    MapInfo,
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug)]
pub enum ServerMessage {
    Move,
    AttackMove,
    AttackTarget,
    Build,
    Assist,
    MapInfoRequest,
}
}

#[derive(Clone, Debug)]
pub struct Builder {
    rate: f64,
    range: f64,
    roster: Rc<HashSet<UnitTypeID>>,
}

#[derive(Clone, Debug)]
pub struct Trainer {
    rate: f64,
    roster: Rc<HashSet<UnitTypeID>>,
    queue: VecDeque<UnitTypeID>,
    repeat_queue: VecDeque<UnitTypeID>,
    progress: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct BuildCharge {
    pub prime_cost: f64,
    pub energy_cost: f64,
    pub build_cost: f64,
    pub current_charges: usize,
    pub max_charges: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Ability {
    range: f64,
    cooldown: f64,
    cooldown_progress: f64,
    targeting: Option<TargetType>,
}

#[derive(Clone, Copy, Debug)]
pub enum Effect {
    SpawnUnits(SpawnUnits),
    Attack(Attack),
}

#[derive(Clone, Copy, Debug)]
pub struct SpawnUnits {
    amount: usize,
    unit_type: UnitTypeID,
}