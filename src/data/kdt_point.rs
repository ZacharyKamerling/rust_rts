use std::f64;
use data::game::Game;
use data::units::Units;
use data::missiles::Missiles;
use libs::kdt::{KDTree, Dimensions};
use libs::movement::Collider;
use libs::movement as mv;
use data::aliases::*;

#[derive(Clone, Copy, Debug)]
pub struct KDTUnit {
    pub id: UnitID,
    pub team: TeamID,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub collision_radius: f64,
    pub weight: f64,
    pub target_type: TargetType,
    pub moving: bool,
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

pub fn populate_with_kdtunits(units: &Units) -> KDTree<KDTUnit> {
    let mut vec = Vec::new();

    for id in units.iter() {
        let (x, y) = units.xy(id);
        let par = KDTUnit {
            id: id,
            team: units.team(id),
            x: x,
            y: y,
            radius: units.radius(id),
            collision_radius: units.collision_radius(id),
            weight: units.weight(id),
            target_type: units.target_type(id),
            moving: units.speed(id) > 0.0,
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
        let (x, y) = missiles.xy[id];
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
        let tt = game.units.target_type(b.id);

        (b.team != team && enemies || b.team == team && allies) && (game.teams.visible[team][b.id] && visible || !visible) &&
            (target_type.has_a_match(tt)) &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
    };

    game.unit_kdt.in_range(&is_matching, &[(x, r), (y, r)])
}

pub fn enemies_in_splash_radius_of_point(game: &Game, u_id: UnitID, w_id: WeaponID, xy: (f64, f64), radius: f64) -> Vec<KDTUnit> {
    let target_type = game.weapons.target_type[w_id];
    let team = game.units.team(u_id);
    get_range_matching(game, xy, team, radius, (true, false, true), target_type)
}

pub fn enemies_in_vision(game: &Game, u_id: UnitID) -> Vec<KDTUnit> {
    let sight_range = game.units.sight_range(u_id);
    let xy = game.units.xy(u_id);
    let team = game.units.team(u_id);
    get_range_matching(
        game,
        xy,
        team,
        sight_range,
        (false, false, true),
        TargetType::new_all_set(),
    )
}

pub fn weapon_targets_in_active_range(game: &Game, u_id: UnitID, w_id: WeaponID) -> Vec<KDTUnit> {
    let active_range = game.units.engagement_range(u_id);
    let target_type = game.weapons.target_type[w_id];
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

pub fn enemies_in_range_and_firing_arc(game: &Game, r: f64, u_id: UnitID, w_id: WeaponID) -> Vec<KDTUnit> {
    let (x, y) = game.units.xy(u_id);
    let team = game.units.team(u_id);
    let target_type = game.weapons.target_type[w_id];

    let is_matching = |b: &KDTUnit| {
        let tt = game.units.target_type(b.id);
        let in_arc = target_in_firing_arc(game, w_id, u_id, b.id);

        (b.team != team) && (game.teams.visible[team][b.id]) && (target_type.has_a_match(tt)) && in_arc &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
    };

    game.unit_kdt.in_range(&is_matching, &[(x, r), (y, r)])
}

#[inline]
fn target_in_firing_arc(game: &Game, w_id: WeaponID, u_id: UnitID, t_id: UnitID) -> bool {
    let (ux, uy) = game.units.xy(u_id);
    let unit_facing = game.units.facing(u_id);
    let coeff = f64::cos(mv::denormalize(unit_facing));
    let (x_off, y_off) = game.weapons.xy_offset[w_id];
    let wpn_x = ux + coeff * x_off;
    let wpn_y = uy + coeff * y_off;

    let (tx, ty) = game.units.xy(t_id);
    let dx = tx - wpn_x;
    let dy = ty - wpn_y;

    let angle_to_enemy = mv::new(dx, dy);
    let lock_angle = game.weapons.lock_offset[w_id] + game.units.facing(u_id);
    let firing_arc = game.weapons.firing_arc[w_id];

    mv::distance(angle_to_enemy, lock_angle) <= firing_arc
}

pub fn get_nearest_enemy(game: &Game, w_id: WeaponID, u_id: UnitID) -> Option<UnitID> {
    let range = game.weapons.range[w_id];
    let radius = game.units.radius(u_id);
    let enemies = enemies_in_range_and_firing_arc(game, range + radius, u_id, w_id);
    let xy = game.units.xy(u_id);

    nearest_in_group(xy, &enemies)
}

pub fn nearest_visible_enemy_in_active_range(game: &Game, u_id: UnitID) -> Option<UnitID> {
    let no_weapon = game.units.weapons(u_id).is_empty();
    let xy = game.units.xy(u_id);

    if no_weapon {
        None
    } else {
        let w_id = game.units.weapons(u_id)[0];
        let enemies = weapon_targets_in_active_range(game, u_id, w_id);

        nearest_in_group(xy, &enemies)
    }
}

fn nearest_in_group((xa, ya): (f64, f64), group: &[KDTUnit]) -> Option<UnitID> {
    if !group.is_empty() {
        let mut nearest_unit = None;
        let mut nearest_dist = f64::MAX;

        for unit in group {
            let xb = unit.x;
            let yb = unit.y;
            let dx = xb - xa;
            let dy = yb - ya;
            let unit_dist = dx * dx + dy * dy;

            if unit_dist < nearest_dist {
                nearest_unit = Some(unit.id);
                nearest_dist = unit_dist;
            }
        }

        nearest_unit
    } else {
        None
    }
}
