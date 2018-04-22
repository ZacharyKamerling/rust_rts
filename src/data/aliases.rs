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
pub use data::units::{Missile};

pub type AnimID = usize;
pub type ProducerID = usize;
pub type AbilityID = usize;
pub type ProducerTypeID = usize;
pub type Milliseconds = isize;

pub const FPS: usize = 10;

#[derive(Clone, Copy, Debug)]
pub struct Visibility {
	radar: Option<f64>,
	vision: Option<f64>,
}

impl Visibility {
	pub fn new() -> Visibility {
		Visibility {
			radar: None,
			vision: None,
		}
	}

	pub fn spot_vision(&self, new_dur: f64) -> Visibility {
		let vision = self.vision.map_or(new_dur, |dur| if dur > new_dur { dur } else { new_dur });

		Visibility {
			radar: self.radar,
			vision: Some(vision),
		}
	}

	pub fn spot_radar(&self, new_dur: f64) -> Visibility {
		let radar = self.radar.map_or(new_dur, |dur| if dur > new_dur { dur } else { new_dur });

		Visibility {
			radar: Some(radar),
			vision: self.vision,
		}
	}

	pub fn step(&self, time_delta: f64) -> Visibility {
		let opt_decrement = |opt| {
			if let Some(dur) = opt {
				let new_dur = dur - time_delta;
				if new_dur > 0.0 {
					Some(new_dur)
				}
				else {
					None
				}
			}
			else {
				None
			}
		};
		Visibility {
			radar: opt_decrement(self.radar),
			vision: opt_decrement(self.vision),
		}
	}

	pub fn is_visible(&self) -> bool {
		self.vision.is_some()
	}

	pub fn is_blip(&self) -> bool {
		self.radar.is_some()
	}
}

#[derive(Clone, Copy, Debug)]
pub enum Damage {
    Single(f64),
    Splash(f64, f64),
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

#[derive(Clone, Debug)]
pub enum Attack {
    // A homing or non-homing projectile
    // that may take more than 1 frame to hit its target.
    Missile(Result<MissileTypeID,String>),
    // An attack that creates no missile
    Melee(Damage),
    // A suicidal attack that creates no missile
    Suicide(Damage),
    // An attack that hits instantly
    Laser(Damage),
    // An attack where the unit doesn't slow down when it engages
    Bomb(Result<MissileTypeID,String>),
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

#[derive(Clone, Copy, Debug)]
pub struct TrainOrder {
    pub order_id: OrderID,
    pub unit_type: UnitTypeID,
    pub repeat: bool,
}

#[derive(Clone, Debug)]
pub enum OrderType {
    Move(MoveGroup),
    AttackMove(MoveGroup),
    AttackTarget(MoveGroup, UnitTarget),
    Build(BuildGroup),
    Assist(UnitTarget),
	Stop,
}

enum_from_primitive! {
#[derive(Clone,Copy, Debug)]
pub enum QueueOrder {
    Prepend,
    Append,
    Replace,
    Clear,
}
}

#[derive(Clone, Copy, Debug)]
pub enum ClientMessage {
    UnitMove,
    UnitDeath,
    OrderCompleted,
    TrainingCompleted,
    MeleeSmack,
    MissileMove,
    MissileExplode,
    Construction,
    TeamInfo,
    MapInfo,
    UnitInfo,
    MissileInfo,
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug)]
pub enum ServerMessage {
    Move,
    AttackMove,
    AttackTarget,
    Build,
    Train,
    Assist,
	Stop,
    MapInfoRequest,
    UnitInfoRequest,
    MissileInfoRequest,
}
}

#[derive(Clone, Debug)]
pub struct Builder {
    rate: f64,
    range: f64,
    roster: Rc<HashSet<String>>,
}

#[derive(Clone, Debug)]
pub struct Trainer {
    pub rate: f64,
    pub roster: Rc<HashSet<String>>,
    pub queue: VecDeque<String>,
    pub repeat_queue: VecDeque<String>,
    pub progress: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct BuildCharge {
    pub prime_cost: f64,
    pub energy_cost: f64,
    pub build_cost: f64,
    pub build_rate: f64,
    pub progress: f64,
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

#[derive(Clone, Debug)]
pub enum Effect {
    SpawnUnits(SpawnUnits),
    Attack(Attack),
}

#[derive(Clone, Debug)]
pub struct SpawnUnits {
    amount: usize,
    unit_type: String,
}