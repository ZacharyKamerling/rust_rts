/*
 This module consists of newtypes/wrappers and many enum types that didn't deserve their own module.
*/

extern crate core;

use data::move_groups::{MoveGroupID};
use self::core::marker::PhantomData;
use std::collections::vec_deque::{VecDeque};
use std::ops::{Index, IndexMut};
use std::fmt::Debug;

pub type AnimID             = usize;
pub type ProducerID         = usize;
pub type AbilityID          = usize;
pub type ProducerTypeID     = usize;
pub type Milliseconds       = isize;
pub type UnitTypeID = usize;
pub type WeaponTypeID = usize;
pub type MissileTypeID = usize;

#[derive(Clone,Copy)]
pub enum Damage {
    Single(f32),
    Splash(f32, f32),
}

#[derive(Clone,Copy)]
pub enum DamageType {
    SmallBlast,
}

/*
Potential things a weapon can aim for.
*/
#[derive(Clone,Copy)]
pub enum Target {
    Point(f32,f32),
    Unit(UnitID),
    None,
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum TargetType {
    Ground,
    Flyer,
    Structure,
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
    UnitIsDamaged(UnitID, UnitID, Damage),
    UnitDealsDamage(UnitID, UnitID, Damage),
    UnitUsesAbility(UnitID, AbilityID, Target),
    UnitEndsAbility(UnitID, AbilityID, Target),
}

#[derive(Clone,Copy,Debug)]
pub enum Order {
    Move(MoveGroupID),
    AttackMove(MoveGroupID),
    AttackTarget(UnitID),
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

    fn index<'a>(&'a self, ix: UID) -> &'a T {
        unsafe {
            &self.vec[ix.usize_unwrap()]
        }
    }
}

impl<UID: USizeWrapper, T> IndexMut<UID> for VecUID<UID,T> {
    fn index_mut<'a>(&'a mut self, ix: UID) -> &'a mut T {
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

id_wrappers!(UnitID,TeamID,WeaponID,MissileID);