extern crate rand;

use std::rc::Rc;
use movement::{normalize};
use std::collections::{HashSet};
use self::rand::Rng;
use data::game::{Game};
use data::units::{Unit};

pub fn setup_game(game: &mut Game) {
    let mut rng = rand::thread_rng();
    let fps = game.fps();
    let basic_unit = Unit {
        unit_type:          0,
        radius:             0.55,
        weight:             1.0,
        top_speed:          10.0 / fps,
        acceleration:       0.5 / fps,
        deceleration:       0.5 / fps,
        turn_rate:          normalize(3.14 / (fps * 1.0)),
        health_regen:       0.5 / fps,
        max_health:         100.0,
        progress_required:  100.0,
        build_rate:         1.0,
        build_range:        1.0,
        build_roster:       Rc::new(HashSet::new()),
        weapons:            Vec::new(),
        sight_range:        12.0,
        radar_range:        16.0,
        active_range:       8.0,
        is_ground:          true,
        is_flying:          false,
        is_structure:       false,
        is_automatic:       false,
    };

    for _ in 0..2 {
        match game.teams.make_team() {
            Some(team) => {
                for y in 16..49 {
                    for x in 16..49 {
                        game.bytegrid.set_point(1, (x,y * 3));
                        game.teams.jps_grid[team].open_or_close_points(1, (x,y * 3), (x,y * 3));
                    }
                }

                for _ in 0..512 {
                    match game.units.make_unit(&mut game.weapons, &basic_unit) {
                        Some(id) => {
                            let x = rng.gen_range(50.0, 100.0);
                            let y = rng.gen_range(50.0, 75.0);
                            game.units.x[id] = x;
                            game.units.y[id] = y;
                            game.units.team[id] = team;
                            game.units.progress[id] = game.units.progress_required[id];
                        }
                        None => {
                            panic!("make_unit: Not enough unit IDs to go around.")
                        }
                    }
                }
            }
            None => ()
        }
    }
}