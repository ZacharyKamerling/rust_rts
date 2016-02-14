/*
 This module consists of newtypes/wrappers and many enum types that didn't deserve their own module.
 This is strictly to enforce type safety.
*/

extern crate core;

use data::move_groups::{MoveGroupID};
use self::core::marker::PhantomData;
use std::collections::vec_deque::{VecDeque};
use std::ops::{Index, IndexMut};

pub type Damage             = f32;
pub type AnimID             = usize;
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

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct TeamID(usize);

unsafe impl USizeWrapper for TeamID {
    unsafe fn usize_unwrap(&self) -> usize {
        let TeamID(ix) = *self;
        ix
    }
    unsafe fn usize_wrap(id: usize) -> TeamID {
        TeamID(id)
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct UnitID(usize);

unsafe impl USizeWrapper for UnitID {
    unsafe fn usize_unwrap(&self) -> usize {
        let UnitID(ix) = *self;
        ix
    }
    unsafe fn usize_wrap(id: usize) -> UnitID {
        UnitID(id)
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct WeaponID(usize);

unsafe impl USizeWrapper for WeaponID {
    unsafe fn usize_unwrap(&self) -> usize {
        let WeaponID(ix) = *self;
        ix
    }
    unsafe fn usize_wrap(id: usize) -> WeaponID {
        WeaponID(id)
    }
}

pub struct UIDPool<T: USizeWrapper> {
    available_ids: VecDeque<T>,
}

impl<T: USizeWrapper> UIDPool<T> {
    pub fn new(size: usize) -> UIDPool<T> {
        let mut available_ids = VecDeque::with_capacity(size);
        let mut c: usize = size;

        while c > 0 {
            c -= 1;
            unsafe {
                available_ids.push_front(T::usize_wrap(c));
            }
        }
        UIDPool { available_ids: available_ids }
    }

    pub fn get_id(&mut self) -> Option<T> {
        self.available_ids.pop_front()
    }

    pub fn put_id(&mut self, id: T) {
        self.available_ids.push_back(id);
    }
}