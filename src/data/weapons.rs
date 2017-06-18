use libs::movement::{Angle, normalize};
use data::aliases::*;
use data::units::UnitTarget;

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