
#[derive(Clone)]
pub struct VisSet{vec: Vec<(bool,usize)>, num: usize}

impl VisSet {
    pub fn with_capacity(size: usize) -> VisSet {
        let mut vec = Vec::with_capacity(size);

        for _ in 0..size {
            vec.push((false,0));
        }

        VisSet{vec: vec, num: 0}
    }

    pub fn insert(&mut self, id: usize) {
        if self.num == self.vec.len() {
            let new_len = self.vec.len() * 2 + 1;
            let mut vec2 = Vec::with_capacity(new_len);

            for _ in 0..new_len {
                vec2.push((false,0));
            }

            for i in 0..self.vec.len() {
                let (b, v) = self.vec[i];
                if b {
                    add_to_vec_set(&mut vec2, v);
                }
            }
            if add_to_vec_set(&mut vec2, id) {
                self.num += 1;
            }
            self.vec = vec2;
        }
        else {
            if add_to_vec_set(&mut self.vec, id) {
                self.num += 1;
            }
        }
        println!("Resized set {}.", 0);
    }

    pub fn inner_vec(&mut self) -> &Vec<(bool,usize)> {
        &self.vec
    }

    pub fn clear(&mut self) {
        for i in 0..self.vec.len() {
            self.vec[i] = (false,0);
        }
    }
}

fn add_to_vec_set(vec: &mut Vec<(bool,usize)>, id: usize) -> bool {
    let mut ix = id % vec.len();

    while vec[ix].0 && vec[ix].1 != id {
        ix += 1;
    }

    if vec[ix].1 == id {
        false
    }
    else {
        vec[ix] = (true,id);
        true
    }
}