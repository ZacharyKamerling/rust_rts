extern crate core;

use data::move_groups::{MoveGroupID};
use std::ops::{Index, IndexMut};
use self::core::marker::PhantomData;

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

pub trait USizeWrapper {
    fn unsafe_unwrap(&self) -> usize;
    fn unsafe_wrap(usize) -> Self;
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
        &self.vec[ix.unsafe_unwrap()]
    }
}

impl<UID: USizeWrapper, T> IndexMut<UID> for VecUID<UID,T> {
    fn index_mut<'a>(&'a mut self, ix: UID) -> &'a mut T {
        &mut self.vec[ix.unsafe_unwrap()]
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct TeamID(usize);

impl USizeWrapper for TeamID {
    fn unsafe_unwrap(&self) -> usize {
        let TeamID(ix) = *self;
        ix
    }
    fn unsafe_wrap(id: usize) -> TeamID {
        TeamID(id)
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct UnitID(usize);

impl USizeWrapper for UnitID {
    fn unsafe_unwrap(&self) -> usize {
        let UnitID(ix) = *self;
        ix
    }
    fn unsafe_wrap(id: usize) -> UnitID {
        UnitID(id)
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct WeaponID(usize);

impl USizeWrapper for WeaponID {
    fn unsafe_unwrap(&self) -> usize {
        let WeaponID(ix) = *self;
        ix
    }
    fn unsafe_wrap(id: usize) -> WeaponID {
        WeaponID(id)
    }
}