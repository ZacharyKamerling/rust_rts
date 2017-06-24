extern crate rand;

use std::rc::Rc;
use libs::movement::{Angle, normalize};
use std::collections::HashSet;
use std::collections::vec_deque::VecDeque;
use data::aliases::*;
use data::kdt_point::KDTUnit;

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

macro_rules! copy_or_borrow_getters_setters_single {
    ($field_name:ident, $set_field:ident, copy, $ty:ty ) => {
        pub fn $field_name(&self) -> $ty {
            self.$field_name
        }

        pub fn $set_field(&mut self, val: $ty) {
            self.$field_name = val;
        }
    };
    ($field_name:ident, $mut_field_name:ident, borrow, $ty:ty ) => {
        pub fn $field_name(&self) -> &$ty {
            &self.$field_name
        }

        pub fn $mut_field_name(&mut self) -> &mut $ty {
            &mut self.$field_name
        }
    };
    ($field_name:ident, $mut_field_name:ident, none, $ty:ty ) => ();
}

macro_rules! copy_or_borrow_getters_setters_aos {
    ($uid: ty, $plural_name:ident, $field_name:ident, $set_field:ident, copy, $ty:ty ) => {
        pub fn $field_name(&self, id: $uid) -> $ty {
            self.elements[id].$field_name
        }

        pub fn $set_field(&mut self, id: $uid, val: $ty) {
            self.elements[id].$field_name = val;
        }
    };
    ($uid: ty, $plural_name:ident, $field_name:ident, $mut_field_name:ident, borrow, $ty:ty ) => {
        pub fn $field_name(&self, id: $uid) -> &$ty {
            &self.elements[id].$field_name
        }

        pub fn $mut_field_name(&mut self, id: $uid) -> &mut $ty {
            &mut self.elements[id].$field_name
        }
    };
    ($uid: ty, $plural_name:ident, $field_name:ident, $mut_field_name:ident, none, $ty:ty ) => ();
}

macro_rules! adjust_for_time_dependency {
    ($proto:ident, $fps:ident, $field_name:ident, time) => {
        $proto.$field_name = $proto.$field_name / $fps;
    };
    ($proto:ident, $fps:ident, $field_name:ident, sqrd) => {
        $proto.$field_name = $proto.$field_name / ($fps * $fps);
    };
    ($proto:ident, $fps:ident, $field_name:ident, none) => ();
}

macro_rules! units {
    ( $plural_name: ident
    , $singular_name: ident
    , $uid: ty
    , $type_id: ty
    , $(
        ( $field_name: ident
        , $set_field: ident
        , $ty: ty
        , $copy_or_borrow: ident
        , $none_time_or_sqrd: ident
        , $expr: expr
        )
    ),* ) => {
        #[derive(Clone,Debug)]
        pub struct $singular_name {
            $(
                $field_name: $ty
            ),*
        }

        impl $singular_name {
            pub fn new() -> $singular_name {
                $singular_name {
                    $(
                        $field_name: $expr
                    ),*
                }
            }

            $(
                copy_or_borrow_getters_setters_single!($field_name, $set_field, $copy_or_borrow, $ty);
            )*
        }

        pub struct $plural_name {
            available_ids: UIDPool<$uid>,
            prototypes: VecUID<$type_id, $singular_name>,
            elements: VecUID<$uid, $singular_name>,
        }

        impl $plural_name {
            pub fn new(num: usize, prototypes: VecUID<$type_id, $singular_name>) -> $plural_name {
                let available_ids = UIDPool::new(num);
                let element = $singular_name {
                    $(
                        $field_name: $expr
                    ),*
                };

                $plural_name {
                    available_ids: available_ids,
                    prototypes: prototypes,
                    elements: VecUID::full_vec(num, element)
                }
            }

            $(
                copy_or_borrow_getters_setters_aos!($uid, $plural_name, $field_name, $set_field, $copy_or_borrow, $ty);
            )*

            pub fn make(&mut self, fps: f64, type_id: $type_id) -> Option<$uid> {
                let mut proto = self.prototypes[type_id].clone();

                match self.available_ids.get_id() {
                    Some(id) => {
                        $(
                            adjust_for_time_dependency!(proto, fps, $field_name, $none_time_or_sqrd);
                        )*
                        for wpn in &mut proto.weapons {
                            wpn.adjust_for_time_dependency(fps);
                        }

                        self.elements[id] = proto;
                        Some(id)
                    }
                    None => None,
                }
            }
        }
    }
}

macro_rules! weapon {
    ( $name: ident
    , $(
        ( $field_name: ident
        , $set_field: ident
        , $ty: ty
        , $copy_or_borrow: ident
        , $none_time_or_sqrd: ident
        , $expr: expr
        )
    ),* ) => {
        #[derive(Clone,Debug)]
        pub struct $name {
            $(
                $field_name: $ty
            ),*
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    $(
                        $field_name: $expr
                    ),*
                }
            }

            pub fn adjust_for_time_dependency(&mut self, fps: f64) {
                $(
                    adjust_for_time_dependency!(self, fps, $field_name, $none_time_or_sqrd);
                )*
            }

            $(
                copy_or_borrow_getters_setters_single!($field_name, $set_field, $copy_or_borrow, $ty);
            )*
        }
    }
}

macro_rules! missiles {
    ( $plural_name: ident
    , $singular_name: ident
    , $uid: ty
    , $type_id: ty
    , $(
        ( $field_name: ident
        , $set_field: ident
        , $ty: ty
        , $copy_or_borrow: ident
        , $none_time_or_sqrd: ident
        , $expr: expr
        )
    ),* ) => {
        #[derive(Clone,Debug)]
        pub struct $singular_name {
            $(
                $field_name: $ty
            ),*
        }

        impl $singular_name {
            pub fn new() -> $singular_name {
                $singular_name {
                    $(
                        $field_name: $expr
                    ),*
                }
            }

            $(
                copy_or_borrow_getters_setters_single!($field_name, $set_field, $copy_or_borrow, $ty);
            )*
        }

        pub struct $plural_name {
            available_ids: UIDPool<$uid>,
            prototypes: VecUID<$type_id, $singular_name>,
            elements: VecUID<$uid, $singular_name>,
        }

        impl $plural_name {
            pub fn new(num: usize, prototypes: VecUID<$type_id, $singular_name>) -> $plural_name {
                let available_ids = UIDPool::new(num);
                let element = $singular_name {
                    $(
                        $field_name: $expr
                    ),*
                };

                $plural_name {
                    available_ids: available_ids,
                    prototypes: prototypes,
                    elements: VecUID::full_vec(num, element)
                }
            }

            $(
                copy_or_borrow_getters_setters_aos!($uid, $plural_name, $field_name, $set_field, $copy_or_borrow, $ty);
            )*

            pub fn make(&mut self, fps: f64, type_id: $type_id) -> Option<$uid> {
                let mut proto = self.prototypes[type_id].clone();

                match self.available_ids.get_id() {
                    Some(id) => {
                        $(
                            adjust_for_time_dependency!(proto, fps, $field_name, $none_time_or_sqrd);
                        )*

                        self.elements[id] = proto;
                        Some(id)
                    }
                    None => None,
                }
            }
        }
    }
}

units!(Units, Unit, UnitID, UnitTypeID,
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
    (prime_cost,            set_prime_cost,         f64,                        copy,   none, 0.0),
    (energy_cost,           set_energy_cost,        f64,                        copy,   none, 0.0),
    (prime_output,          set_prime_output,       f64,                        copy,   time, 0.0),
    (energy_output,         set_energy_output,      f64,                        copy,   time, 0.0),
    (prime_storage,         set_prime_storage,      f64,                        copy,   none, 0.0),
    (energy_storage,        set_energy_storage,     f64,                        copy,   none, 0.0),
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

weapon!(Weapon,
    (attack_type,       set_attack_type,        AttackType,         copy, none, AttackType::MissileAttack(unsafe { MissileTypeID::usize_wrap(0) })),
    (target_id,         set_target_id,          Option<UnitTarget>, copy, none, None),
    (xy_offset,         set_xy_offset,          (f64,f64),          copy, none, (0.0, 0.0)),
    (facing,            set_facing,             Angle,              copy, none, normalize(0.0)),
    (turn_rate,         set_turn_rate,          f64,                copy, time, 0.0),
    (lock_offset,       set_lock_offset,        Angle,              copy, none, normalize(0.0)),
    (firing_arc,        set_firing_arc,         f64,                copy, none, 0.0),
    (missile_speed,     set_missile_speed,      f64,                copy, time, 0.0),
    (range,             set_range,              f64,                copy, none, 0.0),
    (firing_offset,     set_firing_offset,      f64,                copy, none, 0.0),
    (fire_rate,         set_fire_rate,          f64,                copy, none, 0.0),
    (cooldown,          set_cooldown,           f64,                copy, none, 0.0),
    (salvo_size,        set_salvo_size,         usize,              copy, none, 0),
    (salvo,             set_salvo,              usize,              copy, none, 0),
    (salvo_fire_rate,   set_salvo_fire_rate,    f64,                copy, none, 0.0),
    (salvo_cooldown,    set_salvo_cooldown,     f64,                copy, none, 0.0),
    (pellet_count,      set_pellet_count,       usize,              copy, none, 0),
    (pellet_spacing,    set_pellet_spacing,     f64,                copy, none, 0.0),
    (random_offset,     set_random_offset,      f64,                copy, none, 0.0),
    (target_type,       set_target_type,        TargetType,         copy, none, TargetType::new())
);

missiles!(Missiles, Missile, MissileID, MissileTypeID,
    (missile_type_id,   set_missile_type_id,    MissileTypeID,  copy,   none,   unsafe { MissileTypeID::usize_wrap(0) }),
    (target,            set_target,             Target,         copy,   none,   Target::None),
    (facing,            set_facing,             Angle,          copy,   none,   normalize(0.0)),
    (turn_rate,         set_turn_rate,          f64,            copy,   time,   0.0),
    (xy,                set_xy,                 (f64,f64),      copy,   none,   (0.0,0.0)),
    (speed,             set_speed,              f64,            copy,   time,   0.0),
    (travel_dist,       set_travel_dist,        f64,            copy,   none,   0.0),
    (max_travel_dist,   set_max_travel_dist,    f64,            copy,   none,   0.0),
    (damage,            set_damage,             Damage,         copy,   none,   Damage::Single(0.0)),
    (damage_type,       set_damage_type,        DamageType,     copy,   none,   DamageType::SmallBlast),
    (team,              set_team,               TeamID,         copy,   none,   unsafe { TeamID::usize_wrap(0) }),
    (target_type,       set_target_type,        TargetType,     copy,   none,   TargetType::new())
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

impl Missiles {
    pub fn kill_missile(&mut self, id: MissileID) {
        self.available_ids.put_id(id);
    }

    pub fn iter(&self) -> Vec<MissileID> {
        self.available_ids.iter()
    }
}