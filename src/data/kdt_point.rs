use std::f64;
use data::game::Game;
use data::units::{Units,Weapon,Missiles};
use libs::kdt::{KDTree, Dimensions};
use libs::movement::Collider;
use libs::movement as mv;
use data::units::UnitTarget;
use data::aliases::*;

#[derive(Clone, Copy, Debug)]
pub struct KDTUnit {
    pub target: UnitTarget,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub collision_radius: f64,
    pub weight: f64,
}

impl Dimensions for KDTUnit {
    fn bucket_size() -> usize {
        32
    }
    fn num_dims() -> usize {
        2
    }
    fn dimensions(&self, dim: usize) -> f64 {
        match dim {
            0 => self.x,
            _ => self.y,
        }
    }
    fn radii(&self, _: usize) -> f64 {
        self.collision_radius
    }
}

impl Collider for KDTUnit {
    fn x_y_radius_weight(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.collision_radius, self.weight)
    }
}

pub fn populate_with_kdtunits(game: &Game) -> KDTree<KDTUnit> {
    let mut vec = Vec::new();
    let units = &game.units;

    for id in units.iter() {
        let (x, y) = units.xy(id);
        let target = units.new_unit_target(id);
        let par = KDTUnit {
            target: target,
            x: x,
            y: y,
            radius: units.radius(id),
            collision_radius: units.collision_radius(id),
            weight: units.weight(id),
        };
        vec.push(par);
    }

    KDTree::new(vec)
}

#[derive(Clone, Copy)]
pub struct KDTMissile {
    pub id: MissileID,
    pub x: f64,
    pub y: f64,
}

impl Dimensions for KDTMissile {
    fn bucket_size() -> usize {
        256
    }
    fn num_dims() -> usize {
        2
    }
    fn dimensions(&self, dim: usize) -> f64 {
        match dim {
            0 => self.x,
            _ => self.y,
        }
    }
    fn radii(&self, _: usize) -> f64 {
        0.0
    }
}

pub fn populate_with_kdtmissiles(missiles: &Missiles) -> KDTree<KDTMissile> {
    let mut vec = Vec::new();

    for id in missiles.iter() {
        let (x, y) = missiles.xy(id);
        let par = KDTMissile { id: id, x: x, y: y };
        vec.push(par);
    }

    KDTree::new(vec)
}

#[inline]
fn get_range_matching(
    game: &Game,
    (x, y): (f64, f64),
    team: TeamID,
    r: f64,
    (visible, allies, enemies): (bool, bool, bool),
    target_type: TargetType,
) -> Vec<KDTUnit> {
    let is_matching = |b: &KDTUnit| {
        if let Some(id) = game.units.target_id(b.target) {
            let b_team = game.units.team(id);
            let b_target_type = game.units.target_type(id);
            let b_visible = game.teams.visible[team][id].is_visible();
            (b_team != team && enemies || b_team == team && allies) && (b_visible && visible || !visible) &&
            (target_type.has_a_match(b_target_type)) &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
        }
        else {
            false
        }
    };

    game.unit_kdt.in_range(&is_matching, &[(x, r), (y, r)])
}

pub fn enemies_in_splash_radius_of_point(game: &Game, u_id: UnitID, wpn: &Weapon, xy: (f64, f64), radius: f64) -> Vec<KDTUnit> {
    let target_type = wpn.target_type();
    let team = game.units.team(u_id);
    get_range_matching(game, xy, team, radius, (false, false, true), target_type)
}

pub fn all_enemies_in_range(game: &Game, u_id: UnitID, range: f64) -> Vec<KDTUnit> {
    let xy = game.units.xy(u_id);
    let team = game.units.team(u_id);
    get_range_matching(
        game,
        xy,
        team,
        range,
        (false, false, true),
        TargetType::new_all_set(),
    )
}

pub fn weapon_targets_in_active_range(game: &Game, u_id: UnitID, wpn: &Weapon) -> Vec<KDTUnit> {
    let active_range = game.units.engagement_range(u_id);
    let target_type = wpn.target_type();
    let xy = game.units.xy(u_id);
    let team = game.units.team(u_id);

    get_range_matching(
        game,
        xy,
        team,
        active_range,
        (true, false, true),
        target_type,
    )
}

fn enemies_in_range_and_firing_arc(game: &Game, u_id: UnitID, wpn: &Weapon) -> Vec<KDTUnit> {
    let xy = game.units.xy(u_id);
    let team = game.units.team(u_id);
    let target_type = wpn.target_type();
	let wpn_range = wpn.range();
	let radius = game.units.radius(u_id);
	let range = wpn_range + radius;

	get_range_matching(
        game,
        xy,
        team,
        range,
        (true, false, true),
        target_type,
    ).into_iter()
    .filter(|kdtp| target_in_firing_arc(game, wpn, u_id, kdtp.target))
    .collect()
}

#[inline]
fn target_in_firing_arc(game: &Game, wpn: &Weapon, u_id: UnitID, target: UnitTarget) -> bool {
	if let Some(t_id) = game.units.target_id(target) {
		let (ux, uy) = game.units.xy(u_id);
		let unit_facing = game.units.facing(u_id);
		let coeff = f64::cos(mv::denormalize(unit_facing));
		let (x_off, y_off) = wpn.xy_offset();
		let wpn_x = ux + coeff * x_off;
		let wpn_y = uy + coeff * y_off;

		let (tx, ty) = game.units.xy(t_id);
		let dx = tx - wpn_x;
		let dy = ty - wpn_y;

		let angle_to_enemy = mv::new(dx, dy);
		let lock_angle = wpn.lock_offset() + game.units.facing(u_id);
		let firing_arc = wpn.firing_arc();

		mv::distance(angle_to_enemy, lock_angle) <= firing_arc
	}
	else {
		false
	}
}

pub fn get_nearest_enemy(game: &Game, wpn: &Weapon, u_id: UnitID) -> Option<UnitID> {
    let enemies = enemies_in_range_and_firing_arc(game, u_id, wpn);
    let xy = game.units.xy(u_id);

    nearest_in_group(&game.units, xy, &enemies)
}

pub fn nearest_visible_enemy_in_active_range(game: &Game, u_id: UnitID) -> Option<UnitID> {
    let no_weapon = game.units.weapons(u_id).is_empty();
    let xy = game.units.xy(u_id);

    if no_weapon {
        None
    } else {
        let wpn = &game.units.weapons(u_id)[0];
        let enemies = weapon_targets_in_active_range(game, u_id, wpn);

        nearest_in_group(&game.units, xy, &enemies)
    }
}

fn nearest_in_group(units: &Units, (xa, ya): (f64, f64), group: &[KDTUnit]) -> Option<UnitID> {
    if !group.is_empty() {
        let mut nearest_unit = None;
        let mut nearest_dist = f64::MAX;

        for unit in group {
            if let Some(id) = units.target_id(unit.target) {
                let xb = unit.x;
                let yb = unit.y;
                let dx = xb - xa;
                let dy = yb - ya;
                let unit_dist = dx * dx + dy * dy;

                if unit_dist < nearest_dist {
                    nearest_unit = Some(id);
                    nearest_dist = unit_dist;
                }
            }
        }

        nearest_unit
    } else {
        None
    }
}