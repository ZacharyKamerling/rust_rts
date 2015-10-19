extern crate rand;

use std::rc::Rc;
use movement::{normalize};
use std::collections::{HashSet};
//use std::collections::vec_deque::{VecDeque};
use self::rand::Rng;

//use data::aliases::*;
use data::game::{Game};
use data::order::{Order};
//use data::kdt_point::{KDTPoint};
use data::units::{Unit, make_unit};
//use data::weapons::{Weapon, make_weapon};
use basic::{follow_order};

pub fn setup_game(game: &mut Game) {
    let basic_unit = Unit {
        unit_type:          0,
        radius:             0.25,
        weight:             1.0,
        top_speed:          1.0,
        acceleration:       0.25,
        deceleration:       0.25,
        turn_rate:          normalize(3.14 / 10.0),
        health_regen:       0.1,
        max_health:         100.0,
        progress_required:  100.0,
        build_rate:         1.0,
        build_range:        1.0,
        build_roster:       Rc::new(HashSet::new()),
        weapons:            Vec::new(),
        sight_range:        8.0,
        is_ground:          true,
        is_flying:          false,
        is_structure:       false,
        is_automatic:       false,
    };
    let mut rng = rand::thread_rng();

    for _ in 0..2048 {
        let id = make_unit(game, &basic_unit);
        let x = rng.gen_range(0.0, 256.0);
        let y = rng.gen_range(0.0, 256.0);
        game.units.x[id] = x;
        game.units.y[id] = y;
        game.units.team[id] = 0;
        game.units.orders[id].push_front(Order::Move(128.0,128.0));
        game.event_handlers.a_unit_steps[id] = follow_order;
    }
}