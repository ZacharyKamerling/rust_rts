use pathing::path_grid::{PathGrid};
use data::aliases::*;

pub struct Teams {
    available_ids:                  UIDPool<TeamID>,
    pub prime:                      VecUID<TeamID, f64>,
    pub energy:                     VecUID<TeamID, f64>,
    pub jps_grid:                   VecUID<TeamID, PathGrid>,
    pub visible:                    VecUID<TeamID, VecUID<UnitID, bool>>,
    pub visible_missiles:           VecUID<TeamID, VecUID<MissileID, bool>>,
    build_power_distribution:       VecUID<TeamID, VecUID<UnitID, f64>>,
    total_build_power_applied:      VecUID<TeamID, f64>,
}

impl Teams {
    pub fn new(max_units: usize, max_teams: usize, width: usize, height: usize) -> Teams {
        Teams {
            available_ids:              UIDPool::new(max_teams),
            prime:                      VecUID::full_vec(max_teams, 0.0),
            energy:                     VecUID::full_vec(max_teams, 0.0),
            total_build_power_applied:  VecUID::full_vec(max_teams, 0.0),
            jps_grid:                   VecUID::full_vec(max_teams, PathGrid::new(width, height)),
            visible:                    VecUID::full_vec(max_teams, VecUID::full_vec(max_units, false)),
            visible_missiles:           VecUID::full_vec(max_teams, VecUID::full_vec(max_units * 4, false)),
            build_power_distribution:   VecUID::full_vec(max_teams, VecUID::full_vec(max_units, 0.0)),
        }
    }

    pub fn make_team(&mut self) -> Option<TeamID> {
        self.available_ids.get_id()
    }

    pub fn iter(&self) -> Vec<TeamID>
    {
        self.available_ids.iter()
    }

    pub fn distribute_build_power(&mut self, team: TeamID, id: UnitID, build_power: f64) {
        self.build_power_distribution[team][id] += build_power;
        self.total_build_power_applied[team] += build_power;
    }

    pub fn get_build_power_distribution(&mut self, team: TeamID) -> (Vec<(UnitID,f64)>, f64) {
        let tbpa = self.total_build_power_applied[team];
        let mut vec = Vec::new();

        for ix in 0..self.build_power_distribution[team].len() {
            let uid = unsafe {
                UnitID::usize_wrap(ix)
            };
            let build_power = self.build_power_distribution[team][uid];
            if build_power > 0.0 {
                vec.push((uid,build_power))
            }

            self.build_power_distribution[team][uid] = 0.0;
        }

        self.total_build_power_applied[team] = 0.0;

        (vec, tbpa)
    }
}