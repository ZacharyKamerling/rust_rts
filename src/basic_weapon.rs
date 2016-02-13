use movement as mv;
use data::game::{Game};
use data::kdt_point::{KDTPoint};
use data::aliases::*;

pub fn fire_orders(game: &mut Game, w_id: WeaponID) {
    let u_id = game.weapons.unit_id[w_id];
    let range = game.weapons.range[w_id];

    match game.units.orders[u_id].front() {
        Some(&order) => {
            match order {
                Order::AttackMove(mg_id) => {
                    //let flying = game.weapons.
                    //let targets = enemies_in_range(game, w_id, )
                }
                Order::AttackTarget(t_id) => {

                }
                Order::Move(mg_id) => {

                }
            }
        }
        _ => return,
    }
}

fn enemies_in_range(game: &Game, r: f32, w_id: WeaponID, fliers: bool, walkers: bool, structures: bool, visible_req: bool) -> Vec<KDTPoint> {
    let id = game.weapons.unit_id[w_id];
    let x = game.units.x[id];
    let y = game.units.y[id];
    let team = game.units.team[id];

    let is_collider = |b: &KDTPoint| {
        let is_flier = game.units.is_flying[b.id];
        let is_walker = game.units.is_ground[b.id];
        let is_structure = game.units.is_structure[b.id];

        (b.team != team) &&
        (!visible_req || game.teams.visible[team][b.id]) &&
        (is_flier == fliers || is_walker == walkers || is_structure == structures) &&
        {
            let dx = b.x - x;
            let dy = b.y - y;
            let dr = b.radius + r;
            (dx * dx) + (dy * dy) <= dr * dr
        }
    };

    game.kdt.in_range(&is_collider, &[(x,r),(y,r)])
}

/*
pub fn aquire_target(game: &mut Game, w_id: WeaponID) -> Option<UnitID> {

}
*/