use data::game::Game;
use data::build_groups::{BuildGroup, BuildTarget};
use data::kdt_point::KDTUnit;
use behavior::unit::core as unit;
use std::f64;
use data::aliases::*;

pub fn train_unit(game: &mut Game, id: UnitID, train_order: TrainOrder) {
    let team = game.units.team(id);
    let fps = game.fps();
    if let Some(new_id) = game.units.make(fps, train_order.unit_type) {
        let (ux, uy) = game.units.xy(id);
        if let Some((x,y)) = game.teams.jps_grid[team].nearest_open((ux as isize, uy as isize)) {
            let x = x as f64;
            let y = y as f64;
            let build_cost = game.units.build_cost(new_id);
            let max_health = game.units.max_health(id);
            game.units.set_xy(new_id, (x,y));
            game.units.set_team(new_id, team);
            game.units.set_progress(new_id, build_cost);
            game.units.set_health(new_id, max_health);
            game.units.set_train_progress(id, 0.0);
        }

        game.units.mut_train_queue(id).pop_front();
        if train_order.repeat {
            game.units.mut_train_queue(id).push_back(train_order);
        }
    }
}

pub fn build_unit(game: &mut Game, id: UnitID, b_id: UnitID) {
    let team = game.units.team(id);
    let (ux, uy) = game.units.xy(id);
    let (bx, by) = game.units.xy(b_id);
    let build_range = game.units.build_range(id) + game.units.radius(b_id);
    let build_range_sqrd = build_range * build_range;
    let xd = bx - ux;
    let yd = by - uy;
    let distance_sqrd = xd * xd + yd * yd;
    let progress = game.units.progress(b_id);
    let build_cost = game.units.build_cost(b_id);
    let build_rate = game.units.build_rate(id);
    let health = game.units.health(b_id);
    let max_health = game.units.max_health(b_id);

    if progress >= build_cost && health >= max_health {
        unit::complete_assist_order(game, id);
        return;
    }

    if build_range_sqrd >= distance_sqrd {
        unit::slow_down(game, id);
        game.teams.apply_build_power(team, b_id, build_rate);
        game.logger.log_construction(id, b_id);
    } else if let Some(nearest_open) = game.teams.jps_grid[team].nearest_open((bx as isize, by as isize)) {
        let success = unit::calculate_path(game, id, nearest_open);
        if success {
            unit::prune_path(game, id);
            unit::turn_towards_path(game, id);
            unit::speed_up(game, id);
        } else {
            unit::complete_order(game, id);
            return;
        }
    } else {
        panic!("build_unit: There is nowhere open on the map! How is this possible?");
    }
}

pub fn build_at_point(game: &mut Game, bg: &BuildGroup, id: UnitID, (x, y): (f64, f64)) {
    let team = game.units.team(id);
    let (ux, uy) = game.units.xy(id);
    let xd = x - ux;
    let yd = y - uy;
    let distance_sqrd = xd * xd + yd * yd;
    let build_type = bg.build_type();
    let proto = game.units.proto(build_type);
    let build_range = game.units.build_range(id) + proto.radius();
    let build_range_sqrd = build_range * build_range;
    let is_extractor = proto.is_extractor();

    if !proto.is_structure() {
        unit::complete_order(game, id);
        return;
    }

    if build_range_sqrd >= distance_sqrd {
        unit::slow_down(game, id);
        match proto.width_and_height() {
            Some((w, h)) => {
                let hw = w as f64 * 0.5;
                let hh = h as f64 * 0.5;
                let ix = (x - hw + 0.0001) as isize;
                let iy = (y - hh + 0.0001) as isize;
                let fx = ix as f64 + hw;
                let fy = iy as f64 + hh;

                for xo in ix..ix + w {
                    for yo in iy..iy + h {
                        if !game.bytegrid.is_open((xo, yo)) {
                            unit::complete_order(game, id);
                            return;
                        }
                    }
                }

                let colliders = {
                    let is_collider = |c: &KDTUnit| {
                        if let Some(c_id) = game.units.target_id(c.target) {
                            let cx = c.x as isize;
                            let cy = c.y as isize;
                            let tt = game.units.target_type(c_id);
                            let collider_types = TargetType::new().set(TargetTypes::Ground).set(TargetTypes::Hover);
                            tt.has_a_match(collider_types) && cx >= ix && cy >= iy && cx < ix + w && cy < iy + h
                        }
                        else {
                            false
                        }
                    };
                    game.unit_kdt.in_range(&is_collider, &[(fx, hw), (fy, hh)])
                };

                if !colliders.is_empty() {
                    unit::complete_order(game, id);
                    return;
                }

                let fps = game.fps();
                match game.units.make(fps, build_type) {
                    Some(b_id) => {
                        game.units.set_xy(b_id, (fx, fy));
                        game.units.set_team(b_id, team);
                        game.units.set_progress(b_id, 0.0);
                        let unit_targ = game.units.new_unit_target(b_id);
                        bg.set_build_target(BuildTarget::Unit(unit_targ));

                        for xo in ix..ix + w {
                            for yo in iy..iy + h {
                                game.bytegrid.set_point(false, (xo, yo));
                                game.teams.jps_grid[team].close_point((xo, yo));
                            }
                        }
                    }
                    None => panic!("build_at_point: Not enough unit IDs to go around."),
                }
            }
            None => panic!("build_at_point: Building without width and height."),
        }
    } else {
        let success = unit::calculate_path(game, id, (x as isize, y as isize));
        if success {
            unit::prune_path(game, id);
            unit::turn_towards_path(game, id);
            unit::speed_up(game, id);
        } else {
            unit::complete_order(game, id);
            return;
        }
    }
}