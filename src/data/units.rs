extern crate rand;

use std::rc::Rc;
use serde_json;
use libs::movement::{Angle, normalize};
use std::collections::{HashSet};
use std::collections::vec_deque::VecDeque;
use data::aliases::*;
use data::kdt_point::KDTUnit;

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

            pub fn from_json(s: &str) -> Option<$singular_name> {
                let mut tmp = $singular_name::new();
                match serde_json::de::from_str(s) {
                    Ok(serde_json::Value::Object(map)) => {
                        $(
                            let field_name = stringify!($field_name);
                            match map.get(field_name) {
                                Some(v) => {
                                    tmp.$field_name.json_configure(field_name, v);
                                }
                                None => (),
                            }
                        )*
                        Some(tmp)
                    }
                    _ => None,
                }
            }

            $(
                copy_or_borrow_getters_setters_single!($field_name, $set_field, $copy_or_borrow, $ty);
            )*
        }

        #[derive(Clone,Debug)]
        pub struct $plural_name {
            available_ids: UIDPool<$uid>,
            uid_mapping: UIDMapping<$type_id>,
            prototypes: VecUID<$type_id, $singular_name>,
            elements: VecUID<$uid, $singular_name>,
        }

        impl $plural_name {
            pub fn new(num: usize, prototypes: VecUID<$type_id, $singular_name>, uid_mapping: UIDMapping<$type_id>) -> $plural_name {
                let available_ids = UIDPool::new(num);
                let element = $singular_name {
                    $(
                        $field_name: $expr
                    ),*
                };

                $plural_name {
                    available_ids: available_ids,
                    uid_mapping: uid_mapping,
                    prototypes: prototypes,
                    elements: VecUID::full_vec(num, element)
                }
            }

            $(
                copy_or_borrow_getters_setters_aos!($uid, $plural_name, $field_name, $set_field, $copy_or_borrow, $ty);
            )*

            pub fn make_from_name(&mut self, fps: f64, name: String) -> Option<$uid> {
                if let Some(type_id) = self.uid_mapping.name_to_id(name) {
                    self.make(fps,type_id)
                }
                else {
                    None
                }
            }

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

            pub fn from_json(val: &serde_json::Value) -> Option<$name> {
                if let &serde_json::value::Value::Object(ref map) = val {
                    let mut tmp = $name::new();
                    $(
                        let field_name = stringify!($field_name);
                        match map.get(field_name) {
                            Some(v) => {
                                tmp.$field_name.json_configure(field_name, v);
                            }
                            None => (),
                        }
                    )*
                    Some(tmp)
                }
                else {
                    panic!("Couldn't configure weapon. Not an object");
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

            pub fn from_json(s: &str) -> Option<$singular_name> {
                let mut tmp = $singular_name::new();
                match serde_json::de::from_str(s) {
                    Ok(serde_json::Value::Object(map)) => {
                        $(
                            let field_name = stringify!($field_name);
                            match map.get(field_name) {
                                Some(v) => {
                                    tmp.$field_name.json_configure(field_name, v);
                                }
                                None => (),
                            }
                        )*
                        Some(tmp)
                    }
                    _ => None,
                }
            }

            $(
                copy_or_borrow_getters_setters_single!($field_name, $set_field, $copy_or_borrow, $ty);
            )*
        }

        #[derive(Clone,Debug)]
        pub struct $plural_name {
            available_ids: UIDPool<$uid>,
            uid_mapping: UIDMapping<$type_id>,
            prototypes: VecUID<$type_id, $singular_name>,
            elements: VecUID<$uid, $singular_name>,
        }

        impl $plural_name {
            pub fn new(num: usize, prototypes: VecUID<$type_id, $singular_name>, uid_mapping: UIDMapping<$type_id>) -> $plural_name {
                let available_ids = UIDPool::new(num);
                let element = $singular_name {
                    $(
                        $field_name: $expr
                    ),*
                };

                $plural_name {
                    available_ids: available_ids,
                    uid_mapping: uid_mapping,
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

trait JsonConfigure {
    fn json_configure(&mut self, _: &str, _: &serde_json::value::Value) {}
}

impl JsonConfigure for usize {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Number(ref num) = v {
            if let Some(f) = num.as_u64() {
                *self = f as usize;
            }
            else {
                panic!("Couldn't configure {}. The value wasn't a u64.", field_name);
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't a number.", field_name);
        }
    }
}

impl JsonConfigure for String {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::String(ref string) = v {
            *self = string.to_string();
        }
        else {
            panic!("Couldn't configure {}. The value wasn't a string.", field_name);
        }
    }
}

impl JsonConfigure for Result<UnitTypeID,String> {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::String(ref string) = v {
            *self = Err(string.to_string());
        }
        else {
            panic!("Couldn't configure {}. The value wasn't a string.", field_name);
        }
    }
}

impl JsonConfigure for HashSet<String> {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Array(ref array) = v {

            for val in array {
                if let &serde_json::value::Value::String(ref name) = val {
                    self.insert(name.clone());
                }
                else {
                    panic!("Couldn't configure {}. The value wasn't a string.", field_name);
                }
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't an array.", field_name);
        }
    }
}

impl JsonConfigure for Vec<String> {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Array(ref array) = v {

            for val in array {
                if let &serde_json::value::Value::String(ref name) = val {
                    self.push(name.clone());
                }
                else {
                    panic!("Couldn't configure {}. The value wasn't a string.", field_name);
                }
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't an array.", field_name);
        }
    }
}

impl JsonConfigure for (f64,f64) {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Object(ref map) = v {
            if let Some(&serde_json::value::Value::Number(ref w)) = map.get("x") {
                if let Some(&serde_json::value::Value::Number(ref h)) = map.get("y") {
                    if let Some(ux) = w.as_f64() {
                        if let Some(uy) = h.as_f64() {
                            *self = (ux, uy);
                        }
                        else {
                            panic!("Couldn't configure {}. The value wasn't an f64.", field_name);
                        }
                    }
                    else {
                        panic!("Couldn't configure {}. The value wasn't an f64.", field_name);
                    }
                }
                else {
                    panic!("Unit had no \"x\" value.");
                }
            }
            else {
                panic!("Unit had no \"y\" value.");
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't an object.", field_name);
        }
    }
}

impl JsonConfigure for f64 {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Number(ref num) = v {
            if let Some(f) = num.as_f64() {
                *self = f;
            }
            else {
                panic!("Couldn't configure {}. The value wasn't an f64.", field_name);
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't a number.", field_name);
        }
    }
}

impl JsonConfigure for Angle {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Number(ref num) = v {
            if let Some(f) = num.as_f64() {
                *self = normalize(f);
            }
            else {
                panic!("Couldn't configure {}. The value wasn't an f64.", field_name);
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't an f64.", field_name);
        }
    }
}

impl JsonConfigure for TargetType {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Array(ref array) = v {

            for val in array {
                if let &serde_json::value::Value::String(ref s) = val {
                    match s.as_ref() {
                        "ground" => {
                            *self = self.set(TargetTypes::Ground);
                        }
                        "air" => {
                            *self = self.set(TargetTypes::Air);
                        }
                        "water" => {
                            *self = self.set(TargetTypes::Water);
                        }
                        "underwater" => {
                            *self = self.set(TargetTypes::Underwater);
                        }
                        "hover" => {
                            *self = self.set(TargetTypes::Hover);
                        }
                        other => {
                            panic!("Couldn't configure {}. {} is not a valid string.", field_name, other);
                        }
                    }
                }
                else {
                    panic!("Couldn't configure {}. One of the values wasn't a string.", field_name);
                }
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't an array.", field_name);
        }
    }
}

impl JsonConfigure for MoveType {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::String(ref s) = v {
            match s.as_ref() {
                "ground" => {
                    *self = MoveType::Ground;
                }
                "air" => {
                    *self = MoveType::Air;
                }
                "water" => {
                    *self = MoveType::Water;
                }
                "none" => {
                    *self = MoveType::None;
                }
                "hover" => {
                    *self = MoveType::Hover;
                }
                other => {
                    panic!("Couldn't configure {}. {} is not a valid string.", field_name, other);
                }
            }
        }
        else {
            panic!("Couldn't configure {}. One of the values wasn't a string.", field_name);
        }
    }
}

impl JsonConfigure for bool {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Bool(t_or_f) = v {
            *self = t_or_f;
        }
        else {
            panic!("Couldn't configure {}. The value wasn't a bool.", field_name);
        }
    }
}

impl JsonConfigure for Option<(isize,isize)> {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Object(ref map) = v {
            if let Some(&serde_json::value::Value::Number(ref w)) = map.get("w") {
                if let Some(&serde_json::value::Value::Number(ref h)) = map.get("h") {
                    if let Some(ux) = w.as_u64() {
                        if let Some(uy) = h.as_u64() {
                            *self = Some((ux as isize, uy as isize));
                        }
                        else {
                            panic!("Couldn't configure {}. The value wasn't a u64.", field_name);
                        }
                    }
                    else {
                        panic!("Couldn't configure {}. The value wasn't a u64.", field_name);
                    }
                }
                else {
                    panic!("Couldn't configure {}. Unit had no \"height\".");
                }
            }
            else {
                panic!("Couldn't configure {}. Unit had no \"width\".");
            }
        }
        else if !v.is_null() {
            panic!("Couldn't configure {}. The value wasn't an object or null.", field_name);
        }
    }
}

impl JsonConfigure for Attack {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Object(ref map) = v {
            if let Some(&serde_json::value::Value::String(ref attack_type)) = map.get("attack_type") {
                match attack_type.as_ref() {
                    "missile" => {
                        if let Some(&serde_json::value::Value::String(ref missile_name)) = map.get("missile_name") {
                            *self = Attack::Missile(Err(missile_name.clone()))
                        }
                        else {
                            panic!("Couldn't configure {}. The value of missile_name wasn't a string.", field_name);
                        }
                    }
                    _ => {
                        panic!("Couldn't configure {}. {} is not a recognized {}.", field_name, attack_type, field_name);
                    }
                }
            }
        }
    }
}

impl JsonConfigure for Vec<Weapon> {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Array(ref array) = v {

            for val in array {
                if let Some(wpn) = Weapon::from_json(val) {
                    self.push(wpn);
                }
                else {
                    panic!("Couldn't configure {}. A weapon wasn't properly configured.", field_name);
                }
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't an array.", field_name);
        }
    }
}

impl JsonConfigure for Damage {
    fn json_configure(&mut self, field_name: &str, v: &serde_json::value::Value) {
        if let &serde_json::value::Value::Object(ref obj) = v {
            if let Some(&serde_json::value::Value::String(ref dmg_type)) = obj.get("type") {
                match dmg_type.as_ref() {
                    "single" => {
                        if let Some(&serde_json::value::Value::Number(ref amount)) = obj.get("amount") {
                            if let Some(f) = amount.as_f64() {
                                *self = Damage::Single(f);
                            }
                            else {
                                panic!("Couldn't configure {}. The amount wasn't an f64.", field_name);
                            }
                        }
                    }
                    _ => {
                        panic!("Couldn't configure {}. It had no matching type.", field_name);
                    }
                }
            }
            else {
                panic!("Couldn't configure {}. A weapon wasn't properly configured.", field_name);
            }
        }
        else {
            panic!("Couldn't configure {}. The value wasn't an array.", field_name);
        }
    }
}
impl JsonConfigure for HashSet<UnitTypeID> {}
impl JsonConfigure for Option<UnitTypeID> {}
impl JsonConfigure for Option<MissileTypeID> {}
impl JsonConfigure for Option<UnitTarget> {}
impl JsonConfigure for Vec<(isize,isize)> {}
impl<A> JsonConfigure for VecDeque<A> {}
impl JsonConfigure for Vec<UnitID> {}
impl JsonConfigure for Vec<KDTUnit> {}
impl JsonConfigure for TeamID {}
impl JsonConfigure for Target {}
impl JsonConfigure for Vec<u8> {}

// (getter, setter, type, copy/borrow, time dependent?, default value)
units!(Units, Unit, UnitID, UnitTypeID,
    (name,                  mut_name,               String,                         borrow, none, "No Name".to_string()),
    (unit_type,             set_unit_type,          Option<UnitTypeID>,             copy,   none, None),
    (soul_id,               set_soul_id,            usize,                          copy,   none, 0),
    (team,                  set_team,               TeamID,                         copy,   none, unsafe { TeamID::usize_wrap(0) }),
    (anim,                  set_anim,               AnimID,                         copy,   none, 0),
    (encoding,              mut_encoding,           Vec<u8>,                        borrow, none, Vec::new()),
    (xy,                    set_xy,                 (f64,f64),                      copy,   none, (0.0, 0.0)),
    (xy_repulsion,          set_xy_repulsion,       (f64,f64),                      copy,   none, (0.0, 0.0)),
    (radius,                set_radius,             f64,                            copy,   none, 0.0),
    (collision_radius,      set_collision_radius,   f64,                            copy,   none, 0.0),
    (collision_ratio,       set_collision_ratio,    f64,                            copy,   none, 0.0),
    (collision_resist,      set_collision_resist,   f64,                            copy,   none, 0.0),
    (weight,                set_weight,             f64,                            copy,   none, 0.0),
    (speed,                 set_speed,              f64,                            copy,   time, 0.0),
    (top_speed,             set_top_speed,          f64,                            copy,   time, 0.0),
    (acceleration,          set_acceleration,       f64,                            copy,   sqrd, 0.0),
    (deceleration,          set_deceleration,       f64,                            copy,   sqrd, 0.0),
    (facing,                set_facing,             Angle,                          copy,   none, normalize(0.0)),
    (turn_rate,             set_turn_rate,          f64,                            copy,   time, 0.0),
    (path,                  mut_path,               Vec<(isize,isize)>,             borrow, none, Vec::new()),
    (health,                set_health,             f64,                            copy,   none, 0.0),
    (health_regen,          set_health_regen,       f64,                            copy,   time, 0.0),
    (max_health,            set_max_health,         f64,                            copy,   none, 0.0),
    (progress,              set_progress,           f64,                            copy,   none, 0.0),
    (build_cost,            set_build_cost,         f64,                            copy,   none, 0.0),
    (prime_cost,            set_prime_cost,         f64,                            copy,   none, 0.0),
    (energy_cost,           set_energy_cost,        f64,                            copy,   none, 0.0),
    (prime_output,          set_prime_output,       f64,                            copy,   time, 0.0),
    (energy_output,         set_energy_output,      f64,                            copy,   time, 0.0),
    (prime_storage,         set_prime_storage,      f64,                            copy,   none, 0.0),
    (energy_storage,        set_energy_storage,     f64,                            copy,   none, 0.0),
    (orders,                mut_orders,             VecDeque<Rc<Order>>,            borrow, none, VecDeque::new()),
    (build_rate,            set_build_rate,         f64,                            copy,   time, 0.0),
    (build_range,           set_build_range,        f64,                            copy,   none, 0.0),
    (build_roster_names,    mut_build_roster_names, Vec<String>,                    borrow, none, Vec::new()),
    (train_roster_names,    mut_train_roster_names, Vec<String>,                    borrow, none, Vec::new()),
    (build_roster,          mut_build_roster,       HashSet<UnitTypeID>,            borrow, none, HashSet::new()),
    (train_roster,          mut_train_roster,       HashSet<UnitTypeID>,            borrow, none, HashSet::new()),
    (train_rate,            set_train_rate,         f64,                            copy,   time, 0.0),
    (train_progress,        set_train_progress,     f64,                            copy,   none, 0.0),
    (train_queue,           mut_train_queue,        VecDeque<TrainOrder>,           borrow, none, VecDeque::new()),
    (weapons,               mut_weapons,            Vec<Weapon>,                    borrow, none, Vec::new()),
    (passengers,            mut_passengers,         Vec<UnitID>,                    borrow, none, Vec::new()),
    (capacity,              set_capacity,           usize,                          copy,   none, 0),
    (size,                  set_size,               usize,                          copy,   none, 0),
    (target_type,           set_target_type,        TargetType,                     copy,   none, TargetType::new()),
    (move_type,             set_move_type,          MoveType,                       copy,   none, MoveType::None),
    (collision_type,        set_collision_type,     TargetType,                     copy,   none, TargetType::new()),
	(ignores_stealth,		set_ignores_stealth,	bool,							copy,	none, false),
    (ignores_cloak,         set_ignores_cloak,      bool,                           copy,   none, false),
    (is_structure,          set_is_structure,       bool,                           copy,   none, false),
    (is_automatic,          set_is_automatic,       bool,                           copy,   none, false),
    (is_extractor,          set_is_extractor,       bool,                           copy,   none, false),
    (is_stealthed,          set_is_stealthed,       usize,                          copy,   none, 0), // Anything greater than 0 is stealthed
    (is_cloaked,            set_is_cloaked,         usize,                          copy,   none, 0), // Anything greater than 0 is cloaked
    (engagement_range,      set_engagement_range,   f64,                            copy,   none, 0.0),
    (sight_range,           set_sight_range,        f64,                            copy,   none, 0.0),
    (sight_duration,        set_sight_duration,     f64,                            copy,   none, 1.0), // Not time dependent because it is time
    (radar_range,           set_radar_range,        f64,                            copy,   none, 0.0),
    (radar_duration,        set_radar_duration,     f64,                            copy,   none, 1.0), // Not time dependent because it is time
    (stealth_range,         set_stealth_range,      f64,                            copy,   none, 0.0),
    (stealth_duration,      set_stealth_duration,   f64,                            copy,   none, 1.0), // Not time dependent because it is time
	(cloak_range,			set_cloak_range,		f64,							copy,	none, 0.0),
	(cloak_duration,		set_cloak_duration,		f64,							copy,	none, 1.0),
    (width_and_height,      set_width_and_height,   Option<(isize,isize)>,          copy,   none, None),
    (in_range,              mut_in_range,           Vec<KDTUnit>,                   borrow, none, Vec::new())
);

// (getter,             setter,                 type,               easy,   time dependent?, default value)
weapon!(Weapon,
    (name,              mut_name,               String,             borrow, none, "No Name".to_string()),
    (attack,            mut_attack,             Attack,             borrow, none, Attack::Missile(Err("No Type".to_string()))),
    (target_id,         set_target_id,          Option<UnitTarget>, copy,   none, None),
    (xy_offset,         set_xy_offset,          (f64,f64),          copy,   none, (0.0, 0.0)),
    (facing,            set_facing,             Angle,              copy,   none, normalize(0.0)),
    (turn_rate,         set_turn_rate,          f64,                copy,   time, 0.0),
    (lock_offset,       set_lock_offset,        Angle,              copy,   none, normalize(0.0)),
    (firing_arc,        set_firing_arc,         f64,                copy,   none, 0.0),
    (missile_speed,     set_missile_speed,      f64,                copy,   time, 0.0),
    (range,             set_range,              f64,                copy,   none, 0.0),
    (firing_offset,     set_firing_offset,      f64,                copy,   none, 0.0),
    (fire_rate,         set_fire_rate,          f64,                copy,   none, 0.0),
    (cooldown,          set_cooldown,           f64,                copy,   none, 0.0),
    (alternating,       set_alternating,        bool,               copy,   none, false),
    (barrels,           set_barrels,            usize,              copy,   none, 0),
    (barrel_spacing,    set_barrel_spacing,     f64,                copy,   none, 0.0),
    (salvo_size,        set_salvo_size,         usize,              copy,   none, 0),
    (salvo,             set_salvo,              usize,              copy,   none, 0),
    (salvo_fire_rate,   set_salvo_fire_rate,    f64,                copy,   none, 0.0),
    (salvo_cooldown,    set_salvo_cooldown,     f64,                copy,   none, 0.0),
    (pellet_count,      set_pellet_count,       usize,              copy,   none, 0),
    (pellet_spread,     set_pellet_spread,      f64,                copy,   none, 0.0),
    (target_type,       set_target_type,        TargetType,         copy,   none, TargetType::new())
);

// (getter, setter, type, copy/borrow, time dependent?, default value)
missiles!(Missiles, Missile, MissileID, MissileTypeID,
    (name,              mut_name,               String,                         borrow, none,   "No Name".to_string()),
    (missile_type_id,   set_missile_type_id,    Option<MissileTypeID>,          copy,   none,   None),
    (target,            set_target,             Target,                         copy,   none,   Target::None),
    (facing,            set_facing,             Angle,                          copy,   none,   normalize(0.0)),
    (turn_rate,         set_turn_rate,          f64,                            copy,   time,   0.0),
    (xy,                set_xy,                 (f64,f64),                      copy,   none,   (0.0,0.0)),
    (speed,             set_speed,              f64,                            copy,   time,   0.0),
    (travel_dist,       set_travel_dist,        f64,                            copy,   none,   0.0),
    (max_travel_dist,   set_max_travel_dist,    f64,                            copy,   none,   0.0),
    (damage,            set_damage,             Damage,                         copy,   none,   Damage::Single(0.0)),
    (team,              set_team,               TeamID,                         copy,   none,   unsafe { TeamID::usize_wrap(0) }),
    (target_type,       set_target_type,        TargetType,                     copy,   none,   TargetType::new())
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnitTarget {
    soul_id: usize,
    unit_id: UnitID,
}

impl Units {
    pub fn kill_unit(&mut self, id: UnitID) {
        self.available_ids.put_id(id);
        let soul_id = self.soul_id(id);
        self.set_soul_id(id, soul_id + 1);
    }

    pub fn proto(&self, type_id: UnitTypeID) -> Unit {
        self.prototypes[type_id].clone()
    }

    pub fn iter(&self) -> Vec<UnitID> {
        self.available_ids.iter()
    }

    pub fn new_unit_target(&self, unit_id: UnitID) -> UnitTarget {
        UnitTarget {
            soul_id: self.soul_id(unit_id),
            unit_id: unit_id,
        }
    }

    pub fn target_id(&self, target: UnitTarget) -> Option<UnitID> {
        if target.soul_id == self.soul_id(target.unit_id) {
            Some(target.unit_id)
        } else {
            None
        }
    }

    pub fn is_active(&self, id: UnitID) -> bool {
        self.progress(id) >= self.build_cost(id)
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