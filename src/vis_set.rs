
#[derive(Clone)]
pub struct VisSet{vec: Vec<(bool,usize)>}

impl VisSet {
    pub fn with_capacity(size: usize) -> VisSet {
        let mut vec = Vec::with_capacity(size);

        for _ in 0..size {
            vec.push((false,0));
        }

        VisSet{vec: vec}
    }

    pub fn insert(&mut self, id: usize) {
        if self.vec.capacity() == self.vec.len() {
            let new_len = self.vec.len() * 2 + 1;
            let mut vec2 = Vec::with_capacity(new_len);

            for _ in 0..new_len {
                vec2.push((false,0));
            }

            self.vec = vec2;

            for i in 0..self.vec.len() {
                let (b,v) = self.vec[i];
                if b {
                    self.add(v);
                }
            }
        }
        else {
            self.add(id);
        }
    }

    fn add(&mut self, id: usize) {
        let mut ix = id % self.vec.len();

        while self.vec[ix].0 {
            ix += 1;
        }

        self.vec[ix] = (true,id);
    }

    pub fn inner_vec(&mut self) -> &Vec<(bool,usize)> {
        &self.vec
    }
}