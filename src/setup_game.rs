extern crate rand;

use self::rand::Rng;
use data::game::{Game};
//use data::aliases::*;

pub fn setup_game(game: &mut Game) {
    let mut rng = rand::thread_rng();

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
                    match game.units.make_unit(&mut game.weapons, 0) {
                        Some(id) => {
                            let x = rng.gen_range(40.0, 70.0);
                            let y = rng.gen_range(25.0, 45.0);
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