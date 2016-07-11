use data::game::{Game};
use data::units::{Units};
use data::missiles::{Missiles};
use kdt::{KDTree,Dimensions};
use movement::{Collider};
use movement as mv;
use data::aliases::*;

#[derive(Clone,Copy,Debug)]
pub struct KDTUnit {
    pub id:                 UnitID,
    pub team:               TeamID,
    pub x:                  f32,
    pub y:                  f32,
    pub radius:             f32,
    pub collision_radius:   f32,
    pub weight:             f32,
    pub target_type:        TargetType,
    pub moving:             bool,
}

impl Dimensions for KDTUnit {
    fn bucket_size() -> usize {32}
    fn num_dims() -> usize {2}
    fn dimensions(&self, dim: usize) -> f32 {
        match dim {
            0 => { self.x }
            _ => { self.y }
        }
    }
    fn radii(&self, _: usize) -> f32 {
        self.collision_radius
    }
}

impl Collider for KDTUnit {
    fn x_y_radius_weight(&self) -> (f32,f32,f32,f32) {
        (self.x, self.y, self.collision_radius, self.weight)
    }
}

pub fn populate_with_kdtunits(units: &Units) -> KDTree<KDTUnit> {
    let mut vec = Vec::new();

    for id in units.iter() {
        let (x,y) = units.xy(id);
        let par = KDTUnit{ id: id
                          , team: units.team(id)
                          , x: x
                          , y: y
                          , radius: units.radius(id)
                          , collision_radius: units.collision_radius(id)
                          , weight: units.weight(id)
                          , target_type: units.target_type(id)
                          , moving: units.speed(id) > 0.0};
            vec.push(par);
    }

    KDTree::new(vec)
}

#[derive(Clone,Copy)]
pub struct KDTMissile {
    pub id:             MissileID,
    pub x:              f32,
    pub y:              f32,
}

impl Dimensions for KDTMissile {
    fn bucket_size() -> usize {256}
    fn num_dims() -> usize {2}
    fn dimensions(&self, dim: usize) -> f32 {
        match dim {
            0 => { self.x }
            _ => { self.y }
        }
    }
    fn radii(&self, _: usize) -> f32 {
        0.0
    }
}

pub fn populate_with_kdtmissiles(missiles: &Missiles) -> KDTree<KDTMissile> {
    let mut vec = Vec::new();

    for id in missiles.iter() {
        let (x,y) = missiles.xy[id];
        let par = KDTMissile
                { id: id
                , x: x
                , y: y
                };
        vec.push(par);
    }

    KDTree::new(vec)
}

#[inline]
fn get_range_matching(game: &Game, u_id: UnitID, r: f32, visible: bool, allies: bool, enemies: bool, flying: bool, ground: bool, structure: bool) -> Vec<KDTUnit> {
    let (x,y) = game.units.xy(u_id);
    let team = game.units.team(u_id);

    let is_matching = |b: &KDTUnit| {
            let tt = game.units.target_type(b.id);
            let tt_fly = TargetType::Flyer;
            let tt_ground = TargetType::Ground;
            let tt_struct = TargetType::Structure;

            (b.team != team && enemies || b.team == team && allies) &&
            (game.teams.visible[team][b.id] && visible || !visible) &&
            (flying && tt == tt_fly || ground && tt == tt_ground || structure && tt == tt_struct) &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
    };

    game.unit_kdt.in_range(&is_matching, &[(x,r),(y,r)])
}

pub fn enemies_in_vision(game: &Game, u_id: UnitID) -> Vec<KDTUnit> {
    let sight_range = game.units.sight_range(u_id);
    get_range_matching(game, u_id, sight_range, false, false, true, true, true, true)
}

pub fn weapon_targets_in_active_range(game: &Game, u_id: UnitID, w_id: WeaponID) -> Vec<KDTUnit> {
    let active_range = game.units.active_range(u_id);
    let flying = game.weapons.hits_air[w_id];
    let ground = game.weapons.hits_ground[w_id];
    let structures = game.weapons.hits_structure[w_id];

    get_range_matching(game, u_id, active_range, true, false, true, flying, ground, structures)
}

pub fn enemies_in_range_and_firing_arc(game: &Game, r: f32, u_id: UnitID, w_id: WeaponID) -> Vec<KDTUnit> {
    let (x,y) = game.units.xy(u_id);
    let team = game.units.team(u_id);
    let flying = game.weapons.hits_air[w_id];
    let ground = game.weapons.hits_ground[w_id];
    let structure = game.weapons.hits_structure[w_id];

    let is_matching = |b: &KDTUnit| {
            let tt = game.units.target_type(b.id);
            let tt_fly = TargetType::Flyer;
            let tt_ground = TargetType::Ground;
            let tt_struct = TargetType::Structure;
            let in_arc = target_in_firing_arc(game, w_id, u_id, b.id);

            (b.team != team) &&
            (game.teams.visible[team][b.id]) &&
            (flying && tt == tt_fly || ground && tt == tt_ground || structure && tt == tt_struct) &&
            in_arc &&
            {
                let dx = b.x - x;
                let dy = b.y - y;
                let dr = b.radius + r;
                (dx * dx) + (dy * dy) <= dr * dr
            }
    };

    game.unit_kdt.in_range(&is_matching, &[(x,r),(y,r)])
}

#[inline]
fn target_in_firing_arc(game: &Game, w_id: WeaponID, u_id: UnitID, t_id: UnitID) -> bool {
    let (ux,uy) = game.units.xy(u_id);
    let unit_facing = game.units.facing(u_id);
    let coeff = f32::cos(mv::denormalize(unit_facing));
    let (x_off, y_off) = game.weapons.xy_offset[w_id];
    let wpn_x = ux + coeff * x_off;
    let wpn_y = uy + coeff * y_off;

    let (tx,ty) = game.units.xy(t_id);
    let dx = tx - wpn_x;
    let dy = ty - wpn_y;

    let angle_to_enemy = mv::new(dx, dy);
    let lock_angle = game.weapons.lock_offset[w_id] + game.units.facing(u_id);
    let firing_arc = game.weapons.firing_arc[w_id];

    mv::distance(angle_to_enemy, lock_angle) <= firing_arc
}