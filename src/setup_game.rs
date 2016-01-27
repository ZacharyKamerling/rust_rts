extern crate rand;

use std::rc::Rc;
use movement::{normalize};
use std::collections::{HashSet};
use self::rand::Rng;
use data::game::{Game};
use data::units::{Unit, make_unit};
use data::aliases::*;

pub fn setup_game(game: &mut Game) {
    let basic_unit = Unit {
        unit_type:          0,
        radius:             0.50 * 1.1,
        weight:             1.0,
        top_speed:          0.25,
        acceleration:       0.0125,
        deceleration:       0.0125,
        turn_rate:          normalize(3.14 / 10.0),
        health_regen:       0.05,
        max_health:         100.0,
        progress_required:  100.0,
        build_rate:         1.0,
        build_range:        1.0,
        build_roster:       Rc::new(HashSet::new()),
        weapons:            Vec::new(),
        sight_range:        12.0,
        is_ground:          true,
        is_flying:          false,
        is_structure:       false,
        is_automatic:       false,
    };
    let mut rng = rand::thread_rng();

    for x in 16..49 {
        game.bytegrid.set_point(1, (x,32));
        game.teams.jps_grid[0].open_or_close_points(1, (x,32), (x,32));
    }

    for _ in 0..512 {
        let opt_id = make_unit(game, &basic_unit);
        match opt_id {
            Some(id) => {
                let x = rng.gen_range(48.0, 128.0);
                let y = rng.gen_range(48.0, 128.0);
                game.units.x[id] = x;
                game.units.y[id] = y;
                game.units.team[id] = 0;
                game.units.orders[id].push_front(Order::Move(32.0,33.0));
                game.units.progress[id] = game.units.progress_required[id];
            }
            None => {
                panic!("make_unit: Not enough unit IDs to go around.")
            }
        }
    }
}