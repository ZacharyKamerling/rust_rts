use pathing::path_grid::{PathGrid};
use data::aliases::*;

pub struct Teams {
    available_ids:                  UIDPool<TeamID>,
    pub metal:                      VecUID<TeamID, f64>,
    pub energy:                     VecUID<TeamID, f64>,
    pub jps_grid:                   VecUID<TeamID, PathGrid>,
    pub visible:                    VecUID<TeamID, VecUID<UnitID, bool>>,
    pub visible_missiles:           VecUID<TeamID, VecUID<MissileID, bool>>,
}

impl Teams {
    pub fn new(max_units: usize, max_teams: usize, width: usize, height: usize) -> Teams {
        Teams {
            available_ids:      UIDPool::new(max_teams),
            metal:              VecUID::full_vec(max_teams, 0.0),
            energy:             VecUID::full_vec(max_teams, 0.0),
            jps_grid:           VecUID::full_vec(max_teams, PathGrid::new(width, height)),
            visible:            VecUID::full_vec(max_teams, VecUID::full_vec(max_units, false)),
            visible_missiles:   VecUID::full_vec(max_teams, VecUID::full_vec(max_units * 4, false)),
        }
    }

    pub fn make_team(&mut self) -> Option<TeamID> {
        self.available_ids.get_id()
    }

    pub fn iter(&self) -> Vec<TeamID>
    {
        self.available_ids.iter()
    }
}