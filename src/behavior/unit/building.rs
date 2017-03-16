use data::game::{Game};
use data::build_groups::{BuildGroup,BuildTarget};
use data::units::{UnitTarget};
use data::kdt_point::{KDTUnit,KDTMissile};
use behavior::unit::core as unit;
use data::aliases::*;

pub fn build_unit(game: &mut Game, bg: &BuildGroup, id: UnitID, b_id: UnitID) {
    let team = game.units.team(id);
    let (ux,uy) = game.units.xy(id);
    let (bx,by) = game.units.xy(b_id);
    let build_range = game.units.build_range(id) + game.units.radius(b_id);
    let build_range_sqrd = build_range * build_range;
    let xd = bx - ux;
    let yd = by - uy;
    let distance_sqrd = xd * xd + yd * yd;
    let progress = game.units.progress(b_id);
    let progress_required = game.units.progress_required(b_id);
    let new_progress = progress + game.units.build_rate(id);

    if progress >= progress_required {
        game.units.mut_orders(id).pop_front();
        return;
    }

    if build_range_sqrd >= distance_sqrd {
        unit::slow_down(game, id);
        if new_progress >= progress_required {
            game.units.set_progress(b_id, progress_required);
            game.units.mut_orders(id).pop_front();
            return;
        }
        else {
            game.units.set_progress(b_id, new_progress);
        }
    }
    else if let Some(nearest_open) = game.teams.jps_grid[team].nearest_open((bx as isize, by as isize)) {
        unit::calculate_path(game, id, nearest_open);
        unit::prune_path(game, id);
        unit::turn_towards_path(game, id);
        unit::speed_up(game, id);
    }
    else {
        panic!("There is nowhere open on the map! How is this possible?");
    }
}

pub fn build_at_point(game: &mut Game, bg: &BuildGroup, id: UnitID, (x,y): (f32,f32)) {
    let team = game.units.team(id);
    let (ux,uy) = game.units.xy(id);
    let xd = x - ux;
    let yd = y - uy;
    let distance_sqrd = xd * xd + yd * yd;
    let build_type = bg.build_type();
    let proto = game.units.proto(build_type);
    let build_range = game.units.build_range(id) + proto.radius;
    let build_range_sqrd = build_range * build_range;

    if !proto.is_structure {
        game.units.mut_orders(id).pop_front();
        return;
    }

    if build_range_sqrd >= distance_sqrd {
        unit::slow_down(game, id);
        match proto.width_and_height {
            Some((w,h)) => {
                let hw = w as f32 / 2.0;
                let hh = h as f32 / 2.0;
                let ix = (x - hw + 0.00001) as isize;
                let iy = (y - hh + 0.00001) as isize;
                let fx = ix as f32 + hw;
                let fy = iy as f32 + hh;

                for xo in ix..ix + w {
                    for yo in iy..iy + h {
                        if !game.bytegrid.is_open((xo,yo)) {
                            game.units.mut_orders(id).pop_front();
                            return;
                        }
                    }
                }

                let colliders = {
                    let is_collider = |c: &KDTUnit| {
                        let cx = c.x as isize;
                        let cy = c.y as isize;
                        c.target_type.has_a_match(TargetType::new().set_ground()) &&
                        cx >= ix &&
                        cy >= iy &&
                        cx < ix + w &&
                        cy < iy + h
                    };
                    game.unit_kdt.in_range(&is_collider, &[(fx,hw),(fy,hh)])
                };

                if colliders.len() > 0 {
                    game.units.mut_orders(id).pop_front();
                    return;
                }

                match game.units.make_unit(&mut game.weapons, build_type) {
                    Some(b_id) => {
                        game.units.set_xy(b_id, (fx, fy));
                        game.units.set_team(b_id, team);
                        game.units.set_progress(b_id, 0.0);
                        let unit_targ = UnitTarget::new(&game.units, b_id);
                        bg.set_build_target(BuildTarget::Unit(unit_targ));

                        for xo in ix..ix + w {
                            for yo in iy..iy + h {
                                game.bytegrid.set_point(1, (xo,yo));
                                game.teams.jps_grid[team].close_point((xo,yo));
                            }
                        }
                    }
                    None => {
                        panic!("build_at_point: Not enough unit IDs to go around.")
                    }
                }
            }
            None => {
                panic!("build_at_point: Building without width and height.")
            }
        }
    }
    else {
        unit::calculate_path(game, id, (x as isize, y as isize));
        unit::prune_path(game, id);
        unit::turn_towards_path(game, id);
        unit::speed_up(game, id);
    }
}