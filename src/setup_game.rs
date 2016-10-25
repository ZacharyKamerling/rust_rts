extern crate rand;

use self::rand::Rng;
use data::game::{Game};
//use data::aliases::*;

pub fn setup_game(game: &mut Game) {
    let mut rng = rand::thread_rng();

    match game.teams.make_team() {
        Some(team) => {
            for _ in 0..1024 {
                match game.units.make_unit(&mut game.weapons, 0) {
                    Some(id) => {
                        let x = rng.gen_range(0.0, 128.0);
                        let y = rng.gen_range(0.0, 128.0);
                        game.units.set_xy(id, (x,y));
                        game.units.set_team(id, team);
                        let prog_required = game.units.progress_required(id);
                        game.units.set_progress(id, prog_required);
                    }
                    None => {
                        panic!("setup_game: Not enough unit IDs to go around.")
                    }
                }
            }
        }
        None => ()
    }

    match game.teams.make_team() {
        Some(team) => {
            for _ in 0..1024 {
                match game.units.make_unit(&mut game.weapons, 0) {
                    Some(id) => {
                        let x = rng.gen_range(128.0, 256.0);
                        let y = rng.gen_range(0.0, 128.0);
                        game.units.set_xy(id, (x,y));
                        game.units.set_team(id, team);
                        let prog_required = game.units.progress_required(id);
                        game.units.set_progress(id, prog_required);
                    }
                    None => {
                        panic!("setup_game: Not enough unit IDs to go around.")
                    }
                }
            }
        }
        None => ()
    }
}