use data::aliases::*;
use data::game::{Game};


fn a_unit_dies(_: &mut Game, _: UnitID) -> () {}
fn a_unit_steps(_: &mut Game, _: UnitID) -> () {}
fn a_unit_is_attacked (_: &mut Game, _: UnitID, _: UnitID) -> () {}
fn a_unit_is_created(_: &mut Game, _: UnitID, _: UnitID) -> () {}
fn a_unit_is_damaged(_: &mut Game, _: UnitID, _: UnitID, _: f32) -> () {}
fn a_unit_uses_ability(_: &mut Game, _: UnitID, _: AbilityID, _: Target) -> () {}
fn a_unit_ends_ability(_: &mut Game, _: UnitID, _: AbilityID, _: Target) -> () {}

fn full_vec<T: Copy>(n: usize, default: T) -> Vec<T> {
    let mut vec = Vec::with_capacity(n);
    for _ in 0..n {
        vec.push(default);
    }
    vec
}

pub struct EventHandlers {
    a_unit_dies:            Vec<fn(&mut Game, UnitID) -> ()>,
    a_unit_steps:           Vec<fn(&mut Game, UnitID) -> ()>,
    a_unit_is_attacked:     Vec<fn(&mut Game, UnitID, UnitID) -> ()>,
    a_unit_is_created:      Vec<fn(&mut Game, UnitID, UnitID) -> ()>,
    a_unit_is_damaged:      Vec<fn(&mut Game, UnitID, UnitID, f32) -> ()>,
    a_unit_uses_ability:    Vec<fn(&mut Game, UnitID, AbilityID, Target) -> ()>,
    a_unit_ends_ability:    Vec<fn(&mut Game, UnitID, AbilityID, Target) -> ()>,
}

impl EventHandlers {
    pub fn new(num: usize) -> EventHandlers {
        EventHandlers {
            a_unit_dies:            full_vec(num, a_unit_dies),
            a_unit_steps:           full_vec(num, a_unit_steps),
            a_unit_is_attacked:     full_vec(num, a_unit_is_attacked),
            a_unit_is_created:      full_vec(num, a_unit_is_created),
            a_unit_is_damaged:      full_vec(num, a_unit_is_damaged),
            a_unit_uses_ability:    full_vec(num, a_unit_uses_ability),
            a_unit_ends_ability:    full_vec(num, a_unit_ends_ability),
        }
    }
}