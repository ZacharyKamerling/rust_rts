extern crate rand;

use self::rand::Rng;
use data::game::{Game};
use data::aliases::*;
//use libs::tmx_decode::{MapData,decode};

pub fn setup_game(game: &mut Game) {
    let mut rng = rand::thread_rng();
    let unit_type = unsafe { UnitTypeID::usize_wrap(0) };

    if let Some(team) = game.teams.make_team() {
        for _ in 0..1024 {

            match game.units.make_unit(&mut game.weapons, unit_type) {
                Some(id) => {
                    let x = rng.gen_range(0.0, 64.0);
                    let y = rng.gen_range(0.0, 64.0);
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

    if let Some(team) = game.teams.make_team() {
        for _ in 0..1024 {
            match game.units.make_unit(&mut game.weapons, unit_type) {
                Some(id) => {
                    let x = rng.gen_range(92.0, 156.0);
                    let y = rng.gen_range(0.0, 64.0);
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
}