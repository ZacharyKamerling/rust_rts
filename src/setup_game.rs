extern crate rand;

use self::rand::Rng;
use data::game::{Game};
use data::aliases::*;

pub fn setup_game(game: &mut Game) {
    let mut rng = rand::thread_rng();
    let unit_type = unsafe { UnitTypeID::usize_wrap(0) };

    if let Some(team) = game.teams.make_team() {
        for _ in 0..1024 {

            match game.units.make_unit(&mut game.weapons, unit_type) {
                Some(id) => {
                    let x = rng.gen_range(0.0, 32.0);
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
                    let x = rng.gen_range(48.0, 80.0);
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

    let (width,height) = game.map_data.width_and_height();

    for team in game.teams.iter() {
        for i in 0..game.map_data.collisions().len() {
            let collision = game.map_data.collisions()[i];
            let x = i % width;
            let y = i / width;
            let xy = (x as isize, (height - y - 1) as isize);

            match collision {
                0 | 3 | 4 => {
                    game.teams.jps_grid[team].close_point(xy);
                }
                _ => (),
            }
        }
    }

    for i in 0..game.map_data.collisions().len() {
        let collision = game.map_data.collisions()[i];
        let x = i % width;
        let y = i / width;
        let xy = (x as isize, (height - y - 1) as isize);

        match collision {
            0 | 3 | 4 => {
                game.bytegrid.set_point(false, xy);
            }
            _ => (),
        }
    }
}