#[macro_use]
extern crate num_derive;
extern crate num_traits;

use num_traits::FromPrimitive;

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum MoveType {
    GROUND,
    FLYING,
    FLOATING,
    HOVERING,
    AMPHIBIOUS,
    ORBITAL,
    NONE,
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum UnitStatus {
    ALIVE,
    DEAD,
    NON_EXISTENT,
    BEING_MADE,
    BEING_TRANSPORTED,
}

#[derive(Debug, Copy, Clone)]
pub enum Mobility {
    Mobile(MoveData, Option<CollisionData>),
    Structure(StructureData),
}

pub struct MoveData {
    pub speed: f64,
    pub maxSpeed: f64,
    pub acceleration: f64,
    pub deceleration: f64,
    pub facing: f64,
    pub turnRate: f64,
}

pub struct Unit {
    pub x:                          f64,
    pub y:                          f64,
    pub sightRadius:                f64,
    pub radarRadius:                f64,
    pub ignoreStealth:              bool,
    pub ignoreCloak:                bool,
    pub isCloaked:                  usize,
    pub isStealthed:                usize,
    pub weapons:                    Vec<Weapon>,
    pub resources:                  VecUID<ResourceID,Resource>,
    pub flags:                      VecUID<FlagID,bool>,
    pub 
}

pub struct Resource {
    pub max: f64,
    pub regen: f64,
    pub current: f64,
}