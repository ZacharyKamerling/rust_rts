extern crate rand;

use std::rc::Rc;
use libs::movement::{Angle,normalize};
use std::collections::{HashSet};
use std::collections::vec_deque::{VecDeque};
use data::aliases::*;
use data::kdt_point::{KDTUnit};
use data::weapons::{Weapons};

#[derive(Clone,Copy,Debug)]
pub struct UnitTarget {
    soul_id:    SoulID,
    unit_id:    UnitID,
}

impl UnitTarget {
    pub fn new(units: &Units, unit_id: UnitID) -> UnitTarget {
        UnitTarget {
            soul_id: units.soul_id[unit_id],
            unit_id: unit_id,
        }
    }

    pub fn id(&self, units: &Units) -> Option<UnitID> {
        if self.soul_id == units.soul_id[self.unit_id] {
            Some(self.unit_id)
        }
        else {
            None
        }
    }
}

#[derive(Clone,Debug)]
pub struct ProtoUnit {
    pub name:                       &'static str,
    pub radius:                     f64,
    pub collision_radius:           f64,
    pub collision_ratio:            f64,
    pub collision_resist:           f64,
    pub width_and_height:           Option<(isize,isize)>,
    pub weight:                     f64,
    pub top_speed:                  f64,
    pub acceleration:               f64,
    pub deceleration:               f64,
    pub turn_rate:                  f64,
    pub health_regen:               f64,
    pub max_health:                 f64,
    pub progress_required:          f64,
    pub build_rate:                 f64,
    pub build_range:                f64,
    pub build_roster:               Rc<HashSet<UnitTypeID>>,
    pub sight_range:                f64,
    pub radar_range:                f64,
    pub engagement_range:           f64,
    pub weapons:                    Vec<WeaponTypeID>,
    pub target_type:                TargetType,
    pub move_type:                  MoveType,
    pub collision_type:             TargetType,
    pub is_structure:               bool,
    pub is_automatic:               bool,
    //pub prime_cost:                 u64,
    //pub energy_cost:                u64,
    //pub build_cost:                 u64,
}

pub struct Units {
    available_ids:              UIDPool<UnitID>,
    prototypes:                 Vec<ProtoUnit>,
    soul_id:                    VecUID<UnitID,SoulID>,
    // IDENTITY
    unit_type:                  VecUID<UnitID,UnitTypeID>,
    team:                       VecUID<UnitID,TeamID>,
    anim:                       VecUID<UnitID,AnimID>,
    encoding:                   VecUID<UnitID,Vec<u8>>,
    // MOVEMENT
    xy:                         VecUID<UnitID,(f64,f64)>,
    xy_repulsion:               VecUID<UnitID,(f64,f64)>,
    radius:                     VecUID<UnitID,f64>,
    collision_radius:           VecUID<UnitID,f64>,
    collision_ratio:            VecUID<UnitID,f64>,
    collision_resist:           VecUID<UnitID,f64>,
    weight:                     VecUID<UnitID,f64>,
    speed:                      VecUID<UnitID,f64>,
    top_speed:                  VecUID<UnitID,f64>,
    acceleration:               VecUID<UnitID,f64>,
    deceleration:               VecUID<UnitID,f64>,
    facing:                     VecUID<UnitID,Angle>,
    turn_rate:                  VecUID<UnitID,f64>,
    path:                       VecUID<UnitID,Vec<(isize,isize)>>,
    width_and_height:           VecUID<UnitID,Option<(isize,isize)>>,
    // STATS
    health:                     VecUID<UnitID,f64>,
    health_regen:               VecUID<UnitID,f64>,
    max_health:                 VecUID<UnitID,f64>,
    progress:                   VecUID<UnitID,f64>,
    progress_required:          VecUID<UnitID,f64>,
    // PRODUCTION
    build_rate:                 VecUID<UnitID,f64>,
    build_range:                VecUID<UnitID,f64>,
    build_roster:               VecUID<UnitID,Rc<HashSet<UnitTypeID>>>,
    // COMBAT ORIENTED
    weapons:                    VecUID<UnitID,Vec<WeaponID>>,
    orders:                     VecUID<UnitID,VecDeque<Rc<Order>>>,
    passengers:                 VecUID<UnitID,Vec<UnitID>>,
    capacity:                   VecUID<UnitID,usize>,
    size:                       VecUID<UnitID,usize>,
    sight_range:                VecUID<UnitID,f64>,
    radar_range:                VecUID<UnitID,f64>,
    // FLAGS
    target_type:                VecUID<UnitID,TargetType>,
    move_type:                  VecUID<UnitID,MoveType>,
    collision_type:             VecUID<UnitID,TargetType>,
    is_structure:               VecUID<UnitID,bool>,
    is_automatic:               VecUID<UnitID,bool>,
    // MUTABLE FLAGS
    is_stealthed:               VecUID<UnitID,usize>,
    // OTHER
    engagement_range:           VecUID<UnitID,f64>,
    in_range:                   VecUID<UnitID,Vec<KDTUnit>>,
}

impl Units {
    pub fn new(num: usize, prototypes: Vec<ProtoUnit>) -> Units {
        let available_ids = UIDPool::new(num);
        let empty_roster = Rc::new(HashSet::new());

        Units {
            available_ids:          available_ids,
            prototypes:             prototypes,
            soul_id:                VecUID::full_vec(num, 0),
            encoding:               VecUID::full_vec(num, Vec::new()),
            unit_type:              VecUID::full_vec(num, 0),
            team:                   VecUID::full_vec(num, unsafe { TeamID::usize_wrap(0) }),
            anim:                   VecUID::full_vec(num, 0),
            xy:                     VecUID::full_vec(num, (0.0,0.0)),
            xy_repulsion:           VecUID::full_vec(num, (0.0,0.0)),
            radius:                 VecUID::full_vec(num, 0.0),
            collision_radius:       VecUID::full_vec(num, 0.0),
            collision_ratio:        VecUID::full_vec(num, 0.0),
            collision_resist:       VecUID::full_vec(num, 0.0),
            weight:                 VecUID::full_vec(num, 0.0),
            speed:                  VecUID::full_vec(num, 0.0),
            top_speed:              VecUID::full_vec(num, 0.0),
            acceleration:           VecUID::full_vec(num, 0.0),
            deceleration:           VecUID::full_vec(num, 0.0),
            facing:                 VecUID::full_vec(num, normalize(0.0)),
            turn_rate:              VecUID::full_vec(num, 0.0),
            path:                   VecUID::full_vec(num, Vec::new()),
            health:                 VecUID::full_vec(num, 0.0),
            health_regen:           VecUID::full_vec(num, 0.0),
            max_health:             VecUID::full_vec(num, 0.0),
            progress:               VecUID::full_vec(num, 0.0),
            progress_required:      VecUID::full_vec(num, 0.0),
            orders:                 VecUID::full_vec(num, VecDeque::new()),
            build_rate:             VecUID::full_vec(num, 0.0),
            build_range:            VecUID::full_vec(num, 0.0),
            build_roster:           VecUID::full_vec(num, empty_roster.clone()),
            weapons:                VecUID::full_vec(num, Vec::new()),
            passengers:             VecUID::full_vec(num, Vec::new()),
            capacity:               VecUID::full_vec(num, 0),
            size:                   VecUID::full_vec(num, 0),
            target_type:            VecUID::full_vec(num, TargetType::new()),
            move_type:              VecUID::full_vec(num, MoveType::None),
            collision_type:         VecUID::full_vec(num, TargetType::new()),
            is_structure:           VecUID::full_vec(num, false),
            is_automatic:           VecUID::full_vec(num, false),
            is_stealthed:           VecUID::full_vec(num, 0),
            engagement_range:       VecUID::full_vec(num, 0.0),
            sight_range:            VecUID::full_vec(num, 0.0),
            radar_range:            VecUID::full_vec(num, 0.0),
            width_and_height:       VecUID::full_vec(num, None),
            in_range:               VecUID::full_vec(num, Vec::new()),
        }
    }

    pub fn kill_unit(&mut self, id: UnitID) {
        self.available_ids.put_id(id);
        self.set_is_automatic(id, true);
        self.soul_id[id] += 1;
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
                self.set_health(id, proto.max_health);
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
                self.set_progress_required(id, proto.progress_required);
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
            None => None
        }
    }

    pub fn iter(&self) -> Vec<UnitID>
    {
        self.available_ids.iter()
    }
}

macro_rules! unit_copy_getters_setters {
    ( $( ($field_name:ident, $set_name:ident, $field_type:ty) ),* ) => {
        impl Units {
            $(
                pub fn $field_name(&self, id: UnitID) -> $field_type {
                    self.$field_name[id]
                }

                pub fn $set_name(&mut self, id: UnitID, val: $field_type) {
                    self.$field_name[id] = val;
                }
            )*
        }
    }
}

macro_rules! unit_borrow_getters_setters {
    ( $( ($field_name:ident, $mut_field_name:ident, $field_type:ty) ),* ) => {
        impl Units {
            $(
                pub fn $field_name(&self, id: UnitID) -> &$field_type {
                    &self.$field_name[id]
                }

                pub fn $mut_field_name(&mut self, id: UnitID) -> &mut $field_type {
                    &mut self.$field_name[id]
                }
            )*
        }
    }
}

unit_copy_getters_setters!(
    (unit_type,         set_unit_type,          UnitTypeID),
    (team,              set_team,               TeamID),
    (anim,              set_anim,               AnimID),
    (xy,                set_xy,                 (f64,f64)),
    (xy_repulsion,      set_xy_repulsion,       (f64,f64)),
    (radius,            set_radius,             f64),
    (collision_radius,  set_collision_radius,   f64),
    (collision_ratio,   set_collision_ratio,    f64),
    (collision_resist,  set_collision_resist,   f64),
    (weight,            set_weight,             f64),
    (speed,             set_speed,              f64),
    (top_speed,         set_top_speed,          f64),
    (acceleration,      set_acceleration,       f64),
    (deceleration,      set_deceleration,       f64),
    (facing,            set_facing,             Angle),
    (turn_rate,         set_turn_rate,          f64),
    (width_and_height,  set_width_and_height,   Option<(isize,isize)>),
    (health,            set_health,             f64),
    (health_regen,      set_health_regen,       f64),
    (max_health,        set_max_health,         f64),
    (progress,          set_progress,           f64),
    (progress_required, set_progress_required,  f64),
    (build_rate,        set_build_rate,         f64),
    (build_range,       set_build_range,        f64),
    (capacity,          set_capacity,           usize),
    (size,              set_size,               usize),
    (sight_range,       set_sight_range,        f64),
    (radar_range,       set_radar_range,        f64),
    (target_type,       set_target_type,        TargetType),
    (move_type,         set_move_type,          MoveType),
    (collision_type,    set_collision_type,     TargetType),
    (is_structure,      set_is_structure,       bool),
    (is_automatic,      set_is_automatic,       bool),
    (is_stealthed,      set_is_stealthed,       usize),
    (engagement_range,  set_engagement_range,   f64)
);

unit_borrow_getters_setters!(
    (encoding,      mut_encoding,       Vec<u8>),
    (path,          mut_path,           Vec<(isize,isize)>),
    (weapons,       mut_weapons,        Vec<WeaponID>),
    (orders,        mut_orders,         VecDeque<Rc<Order>>),
    (passengers,    mut_passengers,     Vec<UnitID>),
    (in_range,      mut_in_range,       Vec<KDTUnit>),
    (build_roster,  mut_build_roster,   Rc<HashSet<UnitTypeID>>)
);

/*
#[derive(Clone,Debug)]
struct Unit {
    soul_id:                    SoulID,
    unit_type:                  UnitTypeID,
    team:                       TeamID,
    anim:                       AnimID,
    encoding:                   Vec<u8>,
    xy:                         (f64,f64),
    xy_repulsion:               (f64,f64),
    radius:                     f64,
    collision_radius:           f64,
    weight:                     f64,
    speed:                      f64,
    top_speed:                  f64,
    acceleration:               f64,
    deceleration:               f64,
    facing:                     Angle,
    turn_rate:                  f64,
    path:                       Vec<(isize,isize)>,
    width_and_height:           Option<(isize,isize)>,
    health:                     f64,
    health_regen:               f64,
    max_health:                 f64,
    progress:                   f64,
    progress_required:          f64,
    build_rate:                 f64,
    build_range:                f64,
    build_roster:               Rc<HashSet<UnitTypeID>>,
    weapons:                    Vec<WeaponID>,
    orders:                     VecDeque<Order>,
    passengers:                 Vec<UnitID>,
    capacity:                   usize,
    size:                       usize,
    sight_range:                f64,
    radar_range:                f64,
    target_type:                TargetType,
    is_automatic:               bool,
    is_stealthed:               usize,
    engagement_range:           f64,
    in_range:                   Vec<KDTUnit>,
}
*/