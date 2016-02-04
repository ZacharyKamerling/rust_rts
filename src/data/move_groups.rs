/* As units approach the end of their path, they will bunch up and some of them will never reach their destination.
This is natural, but what is not natural is they will keep trying to reach the end forever.
To prevent this, we put units into a move group. As they reach their destination, the move group records how many
have reached the destination and adds up their total area (plus 25% extra). Units only have to move within the
radius of this circular area to complete their movement.
*/

use std::collections::HashMap;
use std::f32;

#[derive(Clone,Copy,Debug)]
pub struct MoveGroupID(usize);

pub struct MoveGroups {
    next_id: usize,
    map: HashMap<usize,(usize,usize,f32,f32)>
}

impl MoveGroups {

    pub fn new() -> MoveGroups {
        MoveGroups
        { next_id: 0
        , map: HashMap::new()
        }
    }

    pub fn make_group(&mut self, size: usize) -> MoveGroupID {
        let id = self.next_id;
        self.map.insert(id, (0, size, 0.0, 0.0));
        self.next_id += 1;
        MoveGroupID(id)
    }

    pub fn done_moving(&mut self, MoveGroupID(mg_id): MoveGroupID, radius: f32) {
        match self.map.get(&mg_id) {
            Some(&(done,size,area,_)) => {
                let new_area = area + radius * radius * 1.25;
                let new_done = done + 1;

                if new_done == size {
                    self.map.remove(&mg_id);
                    //println!("Move group {:?} was deleted.", mg_id);
                }
                else {
                    self.map.insert(mg_id, (new_done, size, new_area, f32::sqrt(new_area)));
                }
            }
            None => {
                println!("Move group doesn't exist.");
            }
        }
    }

    pub fn not_in_move_group_anymore(&mut self, MoveGroupID(mg_id): MoveGroupID) {
        match self.map.get(&mg_id) {
            Some(&(done,size,area,dist)) => {
                let new_size = size - 1;

                if done == new_size {
                    self.map.remove(&mg_id);
                    //println!("Move group {:?} was deleted.", mg_id);
                }
                else {
                    self.map.insert(mg_id, (done, new_size, area, dist));
                }
            }
            None => {
                println!("Move group doesn't exist.");
            }
        }
    }

    pub fn dist_to_group(&self, MoveGroupID(mg_id): MoveGroupID) -> f32 {
        match self.map.get(&mg_id) {
            Some(&(_,_,_,dist)) => {
                dist
            }
            None => {
                println!("Move group doesn't exist.");
                0.0
            }
        }
    }
}