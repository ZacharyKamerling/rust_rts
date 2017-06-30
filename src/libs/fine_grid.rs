extern crate rand;
extern crate time;

use self::time::PreciseTime;
use self::rand::Rng;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FineGrid {
    r: i16,
    trees: Vec<Tree>,
    tiles: Vec<Tile>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Tile {
    parent: u32,
    status: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Tree {
    num_closed: u32,
    nw: u32,
    ne: u32,
    sw: u32,
    se: u32,
    sqr: Square,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square {
    x: i16,
    y: i16,
    r: i16,
}

impl Square {

    fn new(x: i16, y: i16, r: i16) -> Square {
        Square {
            x: x,
            y: y,
            r: r,
        }
    }

    fn sw(self) -> Square {
        let hr = self.r / 2;

        Square {
            x: self.x,
            y: self.y,
            r: hr,
        }
    }

    fn se(self) -> Square {
        let hr = self.r / 2;
        let mx = self.x + hr;

        Square {
            x: mx,
            y: self.y,
            r: hr,
        }
    }

    fn nw(self) -> Square {
        let hr = self.r / 2;
        let my = self.y + hr;

        Square {
            x: self.x,
            y: my,
            r: hr,
        }
    }

    fn ne(self) -> Square {
        let hr = self.r / 2;
        let mx = self.x + hr;
        let my = self.y + hr;

        Square {
            x: mx,
            y: my,
            r: hr,
        }
    }

    fn quadrant(self, x: i16, y: i16) -> Quadrant {
        let cx = self.x + self.r / 2;
        let cy = self.y + self.r / 2;

        if x >= cx {
            if y >= cy {
                Quadrant::NE
            }
            else {
                Quadrant::SE
            }
        }
        else if y >= cy {
            Quadrant::NW
        }
        else {
            Quadrant::SW
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Quadrant {
    NW,
    NE,
    SW,
    SE,
}

impl FineGrid {
    pub fn new() -> FineGrid {
        let r = 1 << 14; // Any higher and we'll eat up too much RAM.
        let trees = Vec::with_capacity((r * r * 4 - 1) / 3 - (r * r));
        let mut tiles = Vec::with_capacity(r * r);

        for _ in 0..r * r {
            tiles.push(Tile { parent: 0, status: true });
        }

        let mut fg = FineGrid {
            r: r as i16,
            trees: trees,
            tiles: tiles,
        };

        fg.construct(Square::new(0, 0, r as i16));

        fg
    }

    pub fn open(&mut self, x: i16, y: i16) {
        let r = self.r;

        if x >= 0 && y >= 0 && x < r && y < r {
            let ix = y as usize * r as usize + x as usize;
            unsafe {
                if self.tiles.get_unchecked(ix).status {
                    return;
                }
                self.tiles.get_unchecked_mut(ix).status = true;
            }
            self.traverse_open(0, x, y, Square::new(0, 0, r));
        }
    }

    fn traverse_open(&mut self, tree_ix: usize, ox: i16, oy: i16, sqr: Square) {
        if sqr.r > 1 {
            let num_closed = unsafe {
                self.trees.get_unchecked(tree_ix).num_closed
            };

            // If num_closed is 1 then this trees closed count is about to be
            // reduced to 0 and we should make it the parent of all its tiles
            if num_closed == 1 {
                self.set_parents(tree_ix, sqr);
                self.traverse_reduce_num_closed(tree_ix, ox, oy, sqr);
            }
            else {
                unsafe {
                    self.trees.get_unchecked_mut(tree_ix).num_closed -= 1;
                }

                match sqr.quadrant(ox, oy) {
                    Quadrant::NE => {
                        let ne = unsafe {
                            self.trees.get_unchecked(tree_ix).ne
                        };
                        self.traverse_open(ne as usize, ox, oy, sqr.ne());
                    }
                    Quadrant::SE => {
                        let se = unsafe {
                            self.trees.get_unchecked(tree_ix).se
                        };
                        self.traverse_open(se as usize, ox, oy, sqr.se());
                    }
                    Quadrant::NW => {
                        let nw = unsafe {
                            self.trees.get_unchecked(tree_ix).nw
                        };
                        self.traverse_open(nw as usize, ox, oy, sqr.nw());
                    }
                    Quadrant::SW => {
                        let sw = unsafe {
                            self.trees.get_unchecked(tree_ix).sw
                        };
                        self.traverse_open(sw as usize, ox, oy, sqr.sw());
                    }
                }
            }
        }
    }

    fn traverse_reduce_num_closed(&mut self, tree_ix: usize, ox: i16, oy: i16, sqr: Square) {
        if sqr.r > 1 {
            unsafe {
                self.trees.get_unchecked_mut(tree_ix).num_closed -= 1;
            }
            match sqr.quadrant(ox, oy) {
                Quadrant::NE => {
                    let ne = unsafe {
                        self.trees.get_unchecked(tree_ix).ne
                    };
                    self.traverse_reduce_num_closed(ne as usize, ox, oy, sqr.ne());
                }
                Quadrant::SE => {
                    let se = unsafe {
                        self.trees.get_unchecked(tree_ix).se
                    };
                    self.traverse_reduce_num_closed(se as usize, ox, oy, sqr.se());
                }
                Quadrant::NW => {
                    let nw = unsafe {
                        self.trees.get_unchecked(tree_ix).nw
                    };
                    self.traverse_reduce_num_closed(nw as usize, ox, oy, sqr.nw());
                }
                Quadrant::SW => {
                    let sw = unsafe {
                        self.trees.get_unchecked(tree_ix).sw
                    };
                    self.traverse_reduce_num_closed(sw as usize, ox, oy, sqr.sw());
                }
            }
        }
    }

    fn set_parents(&mut self, parent: usize, sqr: Square) {
        if sqr.r > 1 {
            for y in sqr.y .. sqr.y + sqr.r {
                for x in sqr.x .. sqr.x + sqr.r {
                    let ix = y as usize * self.r as usize + x as usize;
                    unsafe {
                        self.tiles.get_unchecked_mut(ix).parent = parent as u32;
                    }
                }
            }
        }
    }

    pub fn close(&mut self, x: i16, y: i16) {
        let r = self.r;

        if x >= 0 && y >= 0 && x < r && y < r {
            let ix = y as usize * r as usize + x as usize;
            let status = unsafe {
                self.tiles.get_unchecked(ix).status
            };
            if !status {
                return;
            }
            unsafe {
                self.tiles.get_unchecked_mut(ix).status = false;
            }
            self.traverse_closed(0, x, y, Square::new(0, 0, r));
        }
    }

    fn traverse_closed(&mut self, tree: usize, ox: i16, oy: i16, sqr: Square) {
        if sqr.r > 1 {
            let num_closed = unsafe {
                self.trees.get_unchecked(tree).num_closed
            };
            unsafe {
                self.trees.get_unchecked_mut(tree).num_closed += 1;
            }
            let nw = unsafe {
                self.trees.get_unchecked(tree).nw as usize
            };
            let ne = unsafe {
                self.trees.get_unchecked(tree).ne as usize
            };
            let sw = unsafe {
                self.trees.get_unchecked(tree).sw as usize
            };
            let se = unsafe {
                self.trees.get_unchecked(tree).se as usize
            };
            let set_nw = |a: &mut FineGrid| a.set_parents(nw, sqr.nw());
            let set_ne = |a: &mut FineGrid| a.set_parents(ne, sqr.ne());
            let set_sw = |a: &mut FineGrid| a.set_parents(sw, sqr.sw());
            let set_se = |a: &mut FineGrid| a.set_parents(se, sqr.se());

            if num_closed == 0 {
                match sqr.quadrant(ox, oy) {
                    Quadrant::NE => {
                        set_nw(self);
                        set_sw(self);
                        set_se(self);
                        self.traverse_closed(ne, ox, oy, sqr.ne());
                    }
                    Quadrant::SE => {
                        set_nw(self);
                        set_ne(self);
                        set_sw(self);
                        self.traverse_closed(se, ox, oy, sqr.se());
                    }
                    Quadrant::NW => {
                        set_ne(self);
                        set_sw(self);
                        set_se(self);
                        self.traverse_closed(nw, ox, oy, sqr.nw());
                    }
                    Quadrant::SW => {
                        set_nw(self);
                        set_ne(self);
                        set_se(self);
                        self.traverse_closed(sw, ox, oy, sqr.sw());
                    }
                }
            }
            else {
                match sqr.quadrant(ox, oy) {
                    Quadrant::NE => {
                        self.traverse_closed(ne, ox, oy, sqr.ne());
                    }
                    Quadrant::SE => {
                        self.traverse_closed(se, ox, oy, sqr.se());
                    }
                    Quadrant::NW => {
                        self.traverse_closed(nw, ox, oy, sqr.nw());
                    }
                    Quadrant::SW => {
                        self.traverse_closed(sw, ox, oy, sqr.sw());
                    }
                }
            }
        }
    }

    fn construct(&mut self, sqr: Square) -> usize {
        if sqr.r > 1 {
            let ix = self.trees.len();
            self.trees.push(
                Tree {
                    num_closed: 0,
                    nw: 0, ne: 0,
                    sw: 0,
                    se: 0,
                    sqr: sqr,
                });
            let nw = self.construct(sqr.nw());
            let ne = self.construct(sqr.ne());
            let sw = self.construct(sqr.sw());
            let se = self.construct(sqr.se());
            unsafe {
                self.trees.get_unchecked_mut(ix).nw = nw as u32;
                self.trees.get_unchecked_mut(ix).ne = ne as u32;
                self.trees.get_unchecked_mut(ix).sw = sw as u32;
                self.trees.get_unchecked_mut(ix).se = se as u32;
            }
            ix
        }
        else {
            sqr.y as usize * self.r as usize + sqr.x as usize
        }
    }

    fn verify(&self, tree_ix: usize, sqr: Square) -> bool {
        if sqr.r > 1 {
            let tree = unsafe {
                self.trees.get_unchecked(tree_ix)
            };
            let mut num_closed = 0;

            for y in sqr.y .. sqr.y + sqr.r {
                for x in sqr.x .. sqr.x + sqr.r {
                    let ix = y as usize * self.r as usize + x as usize;

                    let status = unsafe {
                        self.tiles.get_unchecked(ix).status
                    };
                    if !status {
                        num_closed += 1;
                    }
                }
            }

            if num_closed != tree.num_closed {
                println!("Claimed {:?}, Verified {:?}, Sqr: {:?}, Ix: {:?}", tree.num_closed, num_closed, tree.sqr, tree_ix);
            }

            let nw = tree.nw as usize;
            let ne = tree.ne as usize;
            let sw = tree.sw as usize;
            let se = tree.se as usize;
            let a = self.verify(nw, sqr.nw());
            let b = self.verify(ne, sqr.ne());
            let c = self.verify(sw, sqr.sw());
            let d = self.verify(se, sqr.se());

            a && b && c && d && num_closed == tree.num_closed
        }
        else {
            true
        }
    }

    fn get_parent(&self, ox: i16, oy: i16) -> Tree {
        let ix = oy as usize * self.r as usize + ox as usize;
        unsafe {
            let parent_ix = self.tiles.get_unchecked(ix).parent as usize;
            self.trees[parent_ix]
        }
    }

    pub fn neighbors(&mut self, ox: i16, oy: i16) -> Vec<Square> {
        let mut vec = Vec::new();

        if ox < 0 || oy < 0 || ox > self.r || oy > self.r {
            return vec;
        }

        let sqr = {
            let parent = self.get_parent(ox, oy);

            if parent.num_closed > 0 {
                Square::new(ox, oy, 1)
            }
            else {
                parent.sqr
            }
        };

        let n = min_i16(self.r - 1, sqr.y + sqr.r);
        let e = min_i16(self.r - 1, sqr.x + sqr.r);
        let s = max_i16(0, sqr.y - 1);
        let w = max_i16(0, sqr.x - 1);

        let mut prev = Square::new(-1, -1, -1);

        // North row (west to eat)
        for x in w..e {
            let parent = self.get_parent(x, n);

            if parent.sqr != prev {
                vec.push(parent.sqr);
                prev = parent.sqr;
            }
        }

        // East column (north to south)
        for y in n..s {
            let parent = self.get_parent(e, y);

            if parent.sqr != prev {
                vec.push(parent.sqr);
                prev = parent.sqr;
            }
        }

        // South row (east to west)
        for x in e..w {
            let parent = self.get_parent(x, s);

            if parent.sqr != prev {
                vec.push(parent.sqr);
                prev = parent.sqr;
            }
        }

        // West column (south to north)
        for y in s..n {
            let parent = self.get_parent(w, y);

            if parent.sqr != prev {
                vec.push(parent.sqr);
                prev = parent.sqr;
            }
        }

        vec
    }
}

fn min_i16(a: i16, b: i16) -> i16 {
    if a < b {
        a
    }
    else {
        b
    }
}

fn max_i16(a: i16, b: i16) -> i16 {
    if a > b {
        a
    }
    else {
        b
    }
}

pub fn bench_fine_grid() {
    let mili = 1000000.0;
    let mut rng = rand::thread_rng();

    let start = PreciseTime::now();
    let mut fg = FineGrid::new();
    let end = PreciseTime::now();

    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nCreate Fine Grid: {}ms", elapsed);

    let start = PreciseTime::now();
    for _ in 0..1 << 24 {
        let x = rng.gen_range(0, 1 << 14);
        let y = rng.gen_range(0, 1 << 14);
        fg.close(x as i16, y as i16);
    }
    let end = PreciseTime::now();
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nClose Grid: {}ms", elapsed);

    fg.verify(0, Square::new(0, 0, 1 << 14));

    let start = PreciseTime::now();
    for _ in 0..1 << 24 {
        let x = rng.gen_range(0, 1 << 14);
        let y = rng.gen_range(0, 1 << 14);
        fg.open(x as i16, y as i16);
    }
    let end = PreciseTime::now();
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nOpen Grid: {}ms", elapsed);

    let start = PreciseTime::now();
    for _ in 0..1 << 14 {
        let x = rng.gen_range(0, 1 << 14);
        let y = rng.gen_range(0, 1 << 14);
        fg.neighbors(x as i16, y as i16);
    }
    let end = PreciseTime::now();
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nNeighbors: {}ms", elapsed);

    fg.verify(0, Square::new(0, 0, 1 << 14));
}