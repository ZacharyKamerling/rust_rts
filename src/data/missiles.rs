use movement::{Angle,normalize};
use data::aliases::*;
use useful_bits::{full_vec};

pub struct Missile;

pub struct Missiles {
    pub alive:                      Vec<bool>,
    pub target:                     Vec<Target>,
    pub facing:                     Vec<Angle>,
    pub turn_rate:                  Vec<Angle>,
    pub x:                          Vec<f32>,
    pub y:                          Vec<f32>,
    pub speed:                      Vec<f32>,
    pub fuel:                       Vec<f32>,
    pub damage:                     Vec<f32>,
    pub damage_radius:              Vec<f32>,
}

impl Missiles {
    pub fn new(num: usize) -> Missiles {
        Missiles {
            alive:              full_vec(num, false),
            target:             full_vec(num, Target::NoTarget),
            facing:             full_vec(num, normalize(0.0)),
            turn_rate:          full_vec(num, normalize(0.0)),
            x:                  full_vec(num, 0.0),
            y:                  full_vec(num, 0.0),
            speed:              full_vec(num, 0.0),
            fuel:               full_vec(num, 0.0),
            damage:             full_vec(num, 0.0),
            damage_radius:      full_vec(num, 0.0),
        }
    }
}