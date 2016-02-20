/* As units approach the end of their path, they will bunch up and some of them will never reach their destination.
This is natural, but what is not natural is they will keep trying to reach the end forever.
To prevent this, we put units into a move group. As they reach their destination, the move group records how many
have reached the destination and adds up their total area (plus 25% extra). Units then only have to move within the
circular area to complete their movement.
*/

use std::collections::HashMap;
use std::f32;

#[derive(Copy,Clone,Debug)]
pub struct MoveGroupID(usize);

pub struct MoveGroups {
    next_id: usize,
    map: HashMap<usize,MoveGroup>
}

#[derive(Copy,Clone,Debug)]
struct MoveGroup {
    num_done_moving: usize,
    size: usize,
    area: f32,
    dist: f32,
    x: f32,
    y: f32,
}

impl MoveGroups {

    pub fn new() -> MoveGroups {
        MoveGroups
        { next_id: 0
        , map: HashMap::new()
        }
    }

    pub fn make_group(&mut self, size: usize, x: f32, y: f32) -> MoveGroupID {
        let id = self.next_id;
        let move_group = MoveGroup {
            x: x,
            y: y,
            num_done_moving: 0,
            size: size,
            area: 0.0,
            dist: 0.0,
        };
        self.map.insert(id, move_group);
        self.next_id += 1;
        MoveGroupID(id)
    }

    pub fn done_moving(&mut self, MoveGroupID(mg_id): MoveGroupID, radius: f32) {
        let group_is_empty = match self.map.get_mut(&mg_id) {
            Some(mg) => {
                mg.area += radius * radius;
                mg.num_done_moving += 1;

                if mg.num_done_moving == mg.size {
                    true
                }
                else {
                    mg.dist = f32::sqrt(mg.area) * 1.25;
                    false
                }
            }
            None => {
                println!("done_moving: Move group doesn't exist.");
                false
            }
        };

        if group_is_empty {
            self.map.remove(&mg_id);
        }
    }

    pub fn not_in_move_group_anymore(&mut self, MoveGroupID(mg_id): MoveGroupID) {
        let group_is_empty = match self.map.get_mut(&mg_id) {
            Some(mg) => {
                mg.size = mg.size - 1;
                mg.num_done_moving == mg.size
            }
            None => {
                println!("not_in_move_group_anymore: Move group doesn't exist.");
                false
            }
        };

        if group_is_empty {
            self.map.remove(&mg_id);
        }
    }

    pub fn dist_to_group(&self, MoveGroupID(mg_id): MoveGroupID) -> f32 {
        match self.map.get(&mg_id) {
            Some(&mg) => {
                mg.dist
            }
            None => {
                println!("dist_to_group: Move group doesn't exist.");
                0.0
            }
        }
    }

    pub fn move_goal(&self, MoveGroupID(mg_id): MoveGroupID) -> (f32,f32) {
        match self.map.get(&mg_id) {
            Some(&mg) => {
                (mg.x,mg.y)
            }
            None => {
                println!("move_goal: Move group doesn't exist.");
                (0.0,0.0)
            }
        }
    }
}