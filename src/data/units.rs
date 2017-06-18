extern crate rand;

use std::rc::Rc;
use libs::movement::{Angle, normalize};
use std::collections::HashSet;
use std::collections::vec_deque::VecDeque;
use data::aliases::*;
use data::kdt_point::KDTUnit;
use data::weapons::Weapon;

#[derive(Clone, Copy, Debug)]
pub struct UnitTarget {
    soul_id: SoulID,
    unit_id: UnitID,
}

impl UnitTarget {
    pub fn new(units: &Units, unit_id: UnitID) -> UnitTarget {
        UnitTarget {
            soul_id: units.soul_id(unit_id),
            unit_id: unit_id,
        }
    }

    pub fn id(&self, units: &Units) -> Option<UnitID> {
        if self.soul_id == units.soul_id(self.unit_id) {
            Some(self.unit_id)
        } else {
            None
        }
    }
}

uid_aos!(Units, Unit, UnitID, UnitTypeID,
    (soul_id,               set_soul_id,            SoulID,                     copy,   none, 0),
    (unit_type,             set_unit_type,          UnitTypeID,                 copy,   none, unsafe { UnitTypeID::usize_wrap(0) }),
    (team,                  set_team,               TeamID,                     copy,   none, unsafe { TeamID::usize_wrap(0) }),
    (anim,                  set_anim,               AnimID,                     copy,   none, 0),
    (encoding,              mut_encoding,           Vec<u8>,                    borrow, none, Vec::new()),
    (xy,                    set_xy,                 (f64,f64),                  copy,   none, (0.0, 0.0)),
    (xy_repulsion,          set_xy_repulsion,       (f64,f64),                  copy,   none, (0.0, 0.0)),
    (radius,                set_radius,             f64,                        copy,   none, 0.0),
    (collision_radius,      set_collision_radius,   f64,                        copy,   none, 0.0),
    (collision_ratio,       set_collision_ratio,    f64,                        copy,   none, 0.0),
    (collision_resist,      set_collision_resist,   f64,                        copy,   none, 0.0),
    (weight,                set_weight,             f64,                        copy,   none, 0.0),
    (speed,                 set_speed,              f64,                        copy,   time, 0.0),
    (top_speed,             set_top_speed,          f64,                        copy,   time, 0.0),
    (acceleration,          set_acceleration,       f64,                        copy,   sqrd, 0.0),
    (deceleration,          set_deceleration,       f64,                        copy,   sqrd, 0.0),
    (facing,                set_facing,             Angle,                      copy,   none, normalize(0.0)),
    (turn_rate,             set_turn_rate,          f64,                        copy,   time, 0.0),
    (path,                  mut_path,               Vec<(isize,isize)>,         borrow, none, Vec::new()),
    (health,                set_health,             f64,                        copy,   none, 0.0),
    (health_regen,          set_health_regen,       f64,                        copy,   time, 0.0),
    (max_health,            set_max_health,         f64,                        copy,   none, 0.0),
    (progress,              set_progress,           f64,                        copy,   none, 0.0),
    (build_cost,            set_build_cost,         f64,                        copy,   none, 0.0),
    (energy_cost,           set_energy_cost,        f64,                        copy,   none, 0.0),
    (prime_cost,            set_prime_cost,         f64,                        copy,   none, 0.0),
    (energy_output,         set_energy_output,      f64,                        copy,   time, 0.0),
    (prime_output,          set_prime_output,       f64,                        copy,   time, 0.0),
    (orders,                mut_orders,             VecDeque<Rc<Order>>,        borrow, none, VecDeque::new()),
    (build_rate,            set_build_rate,         f64,                        copy,   time, 0.0),
    (build_range,           set_build_range,        f64,                        copy,   none, 0.0),
    (build_roster,          mut_build_roster,       Rc<HashSet<UnitTypeID>>,    borrow, none, Rc::new(HashSet::new())),
    (auto_build,            set_auto_build,         (f64,f64,f64),              copy,   none, (0.0, 0.0, 0.0)),
    (build_queue,           mut_build_queue,        VecDeque<UnitTypeID>,       borrow, none, VecDeque::new()),
    (repeat_build_queue,    mut_repeat_build_queue, VecDeque<UnitTypeID>,       borrow, none, VecDeque::new()),
    (weapons,               mut_weapons,            Vec<Weapon>,                borrow, none, Vec::new()),
    (passengers,            mut_passengers,         Vec<UnitID>,                borrow, none, Vec::new()),
    (capacity,              set_capacity,           usize,                      copy,   none, 0),
    (size,                  set_size,               usize,                      copy,   none, 0),
    (target_type,           set_target_type,        TargetType,                 copy,   none, TargetType::new()),
    (move_type,             set_move_type,          MoveType,                   copy,   none, MoveType::None),
    (collision_type,        set_collision_type,     TargetType,                 copy,   none, TargetType::new()),
    (is_structure,          set_is_structure,       bool,                       copy,   none, false),
    (is_automatic,          set_is_automatic,       bool,                       copy,   none, false),
    (is_stealthed,          set_is_stealthed,       usize,                      copy,   none, 0),
    (engagement_range,      set_engagement_range,   f64,                        copy,   none, 0.0),
    (sight_range,           set_sight_range,        f64,                        copy,   none, 0.0),
    (radar_range,           set_radar_range,        f64,                        copy,   none, 0.0),
    (width_and_height,      set_width_and_height,   Option<(isize,isize)>,      copy,   none, None),
    (in_range,              mut_in_range,           Vec<KDTUnit>,               borrow, none, Vec::new())
);

impl Units {
    pub fn kill_unit(&mut self, id: UnitID) {
        self.available_ids.put_id(id);
        self.set_is_automatic(id, true);
        let soul_id = self.soul_id(id);
        self.set_soul_id(id, soul_id + 1);
    }

    pub fn proto(&self, type_id: UnitTypeID) -> Unit {
        self.prototypes[type_id].clone()
    }

    pub fn iter(&self) -> Vec<UnitID> {
        self.available_ids.iter()
    }
}