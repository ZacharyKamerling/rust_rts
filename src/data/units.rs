extern crate rand;

use std::rc::Rc;
use libs::movement::{Angle, normalize};
use std::collections::HashSet;
use std::collections::vec_deque::VecDeque;
use data::aliases::*;
use data::kdt_point::KDTUnit;
use data::weapons::Weapons;

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

#[derive(Clone, Debug)]
pub struct ProtoUnit {
    pub name: &'static str,
    pub radius: f64,
    pub collision_radius: f64,
    pub collision_ratio: f64,
    pub collision_resist: f64,
    pub width_and_height: Option<(isize, isize)>,
    pub weight: f64,
    pub top_speed: f64,
    pub acceleration: f64,
    pub deceleration: f64,
    pub turn_rate: f64,
    pub health_regen: f64,
    pub max_health: f64,
    pub build_cost: f64,
    pub energy_cost: f64,
    pub prime_cost: f64,
    pub energy_output: f64,
    pub prime_output: f64,
    pub build_rate: f64,
    pub build_range: f64,
    pub build_roster: Rc<HashSet<UnitTypeID>>,
    pub sight_range: f64,
    pub radar_range: f64,
    pub engagement_range: f64,
    pub weapons: Vec<WeaponTypeID>,
    pub target_type: TargetType,
    pub move_type: MoveType,
    pub collision_type: TargetType,
    pub is_structure: bool,
    pub is_automatic: bool,
}

uid_aos!(Units, Unit, UnitID,
    (soul_id,               set_soul_id,            SoulID,                     copy,   0),
    (unit_type,             set_unit_type,          UnitTypeID,                 copy,   unsafe { UnitTypeID::usize_wrap(0) }),
    (team,                  set_team,               TeamID,                     copy,   unsafe { TeamID::usize_wrap(0) }),
    (anim,                  set_anim,               AnimID,                     copy,   0),
    (encoding,              mut_encoding,           Vec<u8>,                    borrow, Vec::new()),
    (xy,                    set_xy,                 (f64,f64),                  copy,   (0.0, 0.0)),
    (xy_repulsion,          set_xy_repulsion,       (f64,f64),                  copy,   (0.0, 0.0)),
    (radius,                set_radius,             f64,                        copy,   0.0),
    (collision_radius,      set_collision_radius,   f64,                        copy,   0.0),
    (collision_ratio,       set_collision_ratio,    f64,                        copy,   0.0),
    (collision_resist,      set_collision_resist,   f64,                        copy,   0.0),
    (weight,                set_weight,             f64,                        copy,   0.0),
    (speed,                 set_speed,              f64,                        copy,   0.0),
    (top_speed,             set_top_speed,          f64,                        copy,   0.0),
    (acceleration,          set_acceleration,       f64,                        copy,   0.0),
    (deceleration,          set_deceleration,       f64,                        copy,   0.0),
    (facing,                set_facing,             Angle,                      copy,   normalize(0.0)),
    (turn_rate,             set_turn_rate,          f64,                        copy,   0.0),
    (path,                  mut_path,               Vec<(isize,isize)>,         borrow, Vec::new()),
    (health,                set_health,             f64,                        copy,   0.0),
    (health_regen,          set_health_regen,       f64,                        copy,   0.0),
    (max_health,            set_max_health,         f64,                        copy,   0.0),
    (progress,              set_progress,           f64,                        copy,   0.0),
    (build_cost,            set_build_cost,         f64,                        copy,   0.0),
    (energy_cost,           set_energy_cost,        f64,                        copy,   0.0),
    (prime_cost,            set_prime_cost,         f64,                        copy,   0.0),
    (energy_output,         set_energy_output,      f64,                        copy,   0.0),
    (prime_output,          set_prime_output,       f64,                        copy,   0.0),
    (orders,                mut_orders,             VecDeque<Rc<Order>>,        borrow, VecDeque::new()),
    (build_rate,            set_build_rate,         f64,                        copy,   0.0),
    (build_range,           set_build_range,        f64,                        copy,   0.0),
    (build_roster,          mut_build_roster,       Rc<HashSet<UnitTypeID>>,    borrow, Rc::new(HashSet::new())),
    (auto_build,            set_auto_build,         (f64,f64,f64),              copy,   (0.0, 0.0, 0.0)),
    (build_queue,           mut_build_queue,        VecDeque<UnitTypeID>,       borrow, VecDeque::new()),
    (repeat_build_queue,    mut_repeat_build_queue, VecDeque<UnitTypeID>,       borrow, VecDeque::new()),
    (weapons,               mut_weapons,            Vec<WeaponID>,              borrow, Vec::new()),
    (passengers,            mut_passengers,         Vec<UnitID>,                borrow, Vec::new()),
    (capacity,              set_capacity,           usize,                      copy,   0),
    (size,                  set_size,               usize,                      copy,   0),
    (target_type,           set_target_type,        TargetType,                 copy,   TargetType::new()),
    (move_type,             set_move_type,          MoveType,                   copy,   MoveType::None),
    (collision_type,        set_collision_type,     TargetType,                 copy,   TargetType::new()),
    (is_structure,          set_is_structure,       bool,                       copy,   false),
    (is_automatic,          set_is_automatic,       bool,                       copy,   false),
    (is_stealthed,          set_is_stealthed,       usize,                      copy,   0),
    (engagement_range,      set_engagement_range,   f64,                        copy,   0.0),
    (sight_range,           set_sight_range,        f64,                        copy,   0.0),
    (radar_range,           set_radar_range,        f64,                        copy,   0.0),
    (width_and_height,      set_width_and_height,   Option<(isize,isize)>,      copy,   None),
    (in_range,              mut_in_range,           Vec<KDTUnit>,               borrow, Vec::new())
);

impl Units {
    pub fn kill_unit(&mut self, id: UnitID) {
        self.available_ids.put_id(id);
        self.set_is_automatic(id, true);
        let soul_id = self.soul_id(id);
        self.set_soul_id(id, soul_id + 1);
    }

    pub fn proto(&self, type_id: UnitTypeID) -> ProtoUnit {
        self.prototypes[type_id].clone()
    }

    pub fn make_unit(&mut self, wpns: &mut Weapons, unit_type: UnitTypeID) -> Option<UnitID> {
        let fps = FPS as f64;
        let proto = self.prototypes[unit_type].clone();
        match self.available_ids.get_id() {
            Some(id) => {
                // Special Stats
                self.mut_encoding(id).clear();
                self.mut_path(id).clear();
                self.mut_weapons(id).clear();
                self.set_unit_type(id, unit_type);
                self.set_anim(id, 0);
                self.set_progress(id, 0.0);
                self.set_speed(id, 0.0);
                self.set_xy_repulsion(id, (0.0, 0.0));
                self.set_health(id, 0.0);
                self.set_is_stealthed(id, 0);
                // Proto Stats
                self.set_radius(id, proto.radius);
                self.set_collision_radius(id, proto.collision_radius);
                self.set_collision_ratio(id, proto.collision_ratio);
                self.set_collision_resist(id, proto.collision_resist);
                self.set_weight(id, proto.weight);
                self.set_top_speed(id, proto.top_speed / fps);
                self.set_acceleration(id, proto.acceleration / (fps * fps));
                self.set_deceleration(id, proto.deceleration / (fps * fps));
                self.set_turn_rate(id, proto.turn_rate / fps);
                self.set_health_regen(id, proto.health_regen / fps);
                self.set_max_health(id, proto.max_health);
                self.set_build_cost(id, proto.build_cost);
                self.set_prime_cost(id, proto.prime_cost);
                self.set_energy_cost(id, proto.energy_cost);
                self.set_prime_output(id, proto.prime_output / fps);
                self.set_energy_output(id, proto.energy_output / fps);
                self.set_build_rate(id, proto.build_rate / fps);
                self.set_build_range(id, proto.build_range);
                *self.mut_build_roster(id) = proto.build_roster.clone();
                self.set_engagement_range(id, proto.engagement_range);
                self.set_sight_range(id, proto.sight_range);
                self.set_radar_range(id, proto.radar_range);
                self.set_target_type(id, proto.target_type);
                self.set_move_type(id, proto.move_type);
                self.set_collision_type(id, proto.collision_type);
                self.set_is_automatic(id, proto.is_automatic);
                self.set_is_structure(id, proto.is_structure);
                self.set_width_and_height(id, proto.width_and_height);

                for wpn_type in &proto.weapons {
                    let wpn_id = wpns.make_weapon(*wpn_type, id);
                    self.mut_weapons(id).push(wpn_id);
                }

                Some(id)
            }
            None => None,
        }
    }

    pub fn iter(&self) -> Vec<UnitID> {
        self.available_ids.iter()
    }
}
