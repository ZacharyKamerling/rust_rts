extern crate rand;

use self::rand::Rng;
use data::game::{Game};
//use data::aliases::*;

pub fn setup_game(game: &mut Game) {
    let mut rng = rand::thread_rng();

    for _ in 0..2 {
        match game.teams.make_team() {
            Some(team) => {
                for _ in 0..1024 {
                    match game.units.make_unit(&mut game.weapons, 0) {
                        Some(id) => {
                            let x = rng.gen_range(0.0, 512.0);
                            let y = rng.gen_range(0.0, 512.0);
                            game.units.xy[id] = (x,y);
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