/*
 This module consists of newtypes/wrappers and many enum types that didn't deserve their own module.
*/

extern crate core;
extern crate num;

use data::move_groups::{MoveGroup};
use data::build_groups::{BuildGroup};
use data::units::UnitTarget;
use self::core::marker::PhantomData;
use std::collections::vec_deque::{VecDeque};
use std::ops::{Index, IndexMut};
use std::fmt::Debug;

pub type SoulID             = usize;
pub type AnimID             = usize;
pub type ProducerID         = usize;
pub type AbilityID          = usize;
pub type ProducerTypeID     = usize;
pub type Milliseconds       = isize;
pub type WeaponTypeID       = usize;
pub type MissileTypeID      = usize;

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
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum MoveType {
    None,
    Ground,
    Air,
    Hover,
    Water,
}

#[derive(Clone,Copy,Debug)]
pub struct TargetType {
    byte: u8,
}

impl TargetType {

    pub fn new() -> TargetType {
        TargetType { byte: 0 }
    }

    pub fn new_all_set() -> TargetType {
        TargetType { byte: 0b11111111 }
    }

    //1
    pub fn set_ground(self) -> TargetType {
        TargetType { byte: self.byte | 1 }
    }

    pub fn ground(self) -> bool {
        self.byte & 1 == 1
    }

    //2
    pub fn set_air(self) -> TargetType {
        TargetType { byte: self.byte | (1 << 1) }
    }

    pub fn air(self) -> bool {
        self.byte & (1 << 1) == (1 << 1)
    }

    //3
    pub fn set_water(self) -> TargetType {
        TargetType { byte: self.byte | (1 << 2) }
    }

    pub fn water(self) -> bool {
        self.byte & (1 << 2) == (1 << 2)
    }

    //4
    pub fn set_structure(self) -> TargetType {
        TargetType { byte: self.byte | (1 << 3) }
    }

    pub fn structure(self) -> bool {
        self.byte & (1 << 3) == (1 << 3)
    }

    pub fn has_a_match(self, other: TargetType) -> bool {
        self.byte & other.byte > 0
    }
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

pub unsafe trait USizeWrapper {
    unsafe fn usize_unwrap(&self) -> usize;
    unsafe fn usize_wrap(usize) -> Self;
}

#[derive(Clone,Debug)]
pub struct VecUID<UID,T> {
    vec: Vec<T>,
    index_type: PhantomData<UID>,
}

impl<UID, T: Clone> VecUID<UID,T> {
    pub fn full_vec(size: usize, default: T) -> VecUID<UID,T> {
        let mut vec = Vec::with_capacity(size);
        for _ in 0..size {
            vec.push(default.clone());
        }

        VecUID {
            vec: vec,
            index_type: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

impl<UID: USizeWrapper, T> Index<UID> for VecUID<UID,T> {
    type Output = T;

    fn index(& self, ix: UID) -> &T {
        unsafe {
            self.vec.get_unchecked(ix.usize_unwrap())
        }
    }
}

impl<UID: USizeWrapper, T> IndexMut<UID> for VecUID<UID,T> {
    fn index_mut(&mut self, ix: UID) -> &mut T {
        unsafe {
            &mut self.vec[ix.usize_unwrap()]
        }
    }
}

pub struct UIDPool<T> {
    available_ids: VecDeque<T>,
    iteratable_ids: Vec<T>,
}

impl<T: USizeWrapper + Ord + Copy + Debug> UIDPool<T> {
    pub fn new(size: usize) -> UIDPool<T> {
        let mut available_ids = VecDeque::with_capacity(size);
        let mut c: usize = size;

        while c > 0 {
            c -= 1;
            unsafe {
                available_ids.push_front(T::usize_wrap(c));
            }
        }
        UIDPool {
            available_ids: available_ids,
            iteratable_ids: Vec::with_capacity(size),
        }
    }

    pub fn get_id(&mut self) -> Option<T> {
        match self.available_ids.pop_front() {
            Some(id) => {
                match self.iteratable_ids.binary_search(&id) {
                    Ok(_) => {
                        println!("I don't know how you did it, but you took the same ID from a UIDPool twice.");
                        None
                    }
                    Err(i) => {
                        self.iteratable_ids.insert(i, id);
                        Some(id)
                    }
                }
            }
            None => None
        }
    }

    pub fn put_id(&mut self, id: T) {
        match self.iteratable_ids.binary_search(&id) {
            Ok(i) => {
                self.available_ids.push_back(id);
                self.iteratable_ids.remove(i);
            }
            Err(_) => {
                println!("You tried to put the same ID into a UIDPool twice. {:?}", id);
            }
        }
    }

    pub fn iter(&self) -> Vec<T> {
        self.iteratable_ids.to_vec()
    }
}

macro_rules! id_wrappers {
    ( $( $x:ident ),* ) => {
        $(
            #[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
            pub struct $x(usize);

            unsafe impl USizeWrapper for $x {
                unsafe fn usize_unwrap(&self) -> usize {
                    let $x(ix) = *self;
                    ix
                }
                unsafe fn usize_wrap(id: usize) -> $x {
                    $x(id)
                }
            }
        )*
    }
}

id_wrappers!(UnitID,TeamID,WeaponID,MissileID,UnitTypeID,OrderID);