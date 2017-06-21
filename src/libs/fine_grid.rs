extern crate rand;
extern crate time;

use self::time::PreciseTime;
use self::rand::Rng;

struct BitVec {
    vec: Vec<u8>,
}

impl BitVec {
    fn new(size: usize) -> BitVec {
        let size = {
            if size % 8 == 0 {
                size / 8
            }
            else {
                size / 8 + 1
            }
        };

        let mut vec = Vec::with_capacity(size);

        for _ in 0..size {
            vec.push(0);
        }

        BitVec { vec: vec }
    }

    fn get(&self, ix: usize) -> bool {
        let jx = ix >> 3;
        let kx = *unsafe { self.vec.get_unchecked(jx) };
        let lx = kx & (1 << (ix & 7));
        lx > 0
    }

    fn set(&mut self, ix: usize, v: bool) {
        let jx = ix >> 3;
        let kx = *unsafe { self.vec.get_unchecked(jx) };
        let lx = if v {
            kx | (1 << (ix & 7))
        }
        else {
            kx & !(1 << (ix & 7))
        };

        self.vec[jx] = lx;
    }
}

#[test]
fn test() {
    let mut bv = BitVec::new(1000);

    for i in 0..1000 {
        bv.set(i, true);
        assert!(bv.get(i));
    }

    bv.set(1, false);
    bv.set(3, false);
    bv.set(4, false);
    bv.set(6, false);

    assert!(   bv.get(0));
    assert!( ! bv.get(1));
    assert!(   bv.get(2));
    assert!( ! bv.get(3));
    assert!( ! bv.get(4));
    assert!(   bv.get(5));
    assert!( ! bv.get(6));
    assert!(   bv.get(7));
    assert!(   bv.get(8));
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FineGrid {
    w: i16,
    h: i16,
    trees: Vec<Tree>,
    tiles: Vec<Tile>,

    tn_w: usize,
    tn_h: usize,
    total_neighbors: usize,
    filter: usize,
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
struct Square {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

impl Square {

    fn new(x: i16, y: i16, w: i16, h: i16) -> Square {
        Square {
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }

    fn sw(self) -> Square {
        let hw = self.w / 2;
        let hh = self.h / 2;

        Square {
            x: self.x,
            y: self.y,
            w: hw,
            h: hh,
        }
    }

    fn se(self) -> Square {
        let hw = self.w / 2;
        let hh = self.h / 2;
        let mx = self.x + hw;

        Square {
            x: mx,
            y: self.y,
            w: hw,
            h: hh,
        }
    }

    fn nw(self) -> Square {
        let hw = self.w / 2;
        let hh = self.h / 2;
        let my = self.y + hh;

        Square {
            x: self.x,
            y: my,
            w: hw,
            h: hh,
        }
    }

    fn ne(self) -> Square {
        let hw = self.w / 2;
        let hh = self.h / 2;
        let mx = self.x + hw;
        let my = self.y + hh;

        Square {
            x: mx,
            y: my,
            w: hw,
            h: hh,
        }
    }

    fn quadrant(self, x: i16, y: i16) -> Quadrant {
        let cx = self.x + self.w / 2;
        let cy = self.y + self.h / 2;

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
        let w = 1 << 14; // Any higher and we'll eat up too much RAM.
        let h = 1 << 14;
        let trees = Vec::with_capacity((w * h * 4 - 1) / 3 - (w * h));
        let mut tiles = Vec::with_capacity(w * h);

        for _ in 0..w * h {
            tiles.push(Tile { parent: 0, status: true });
        }

        let mut fg = FineGrid {
            w: w as i16,
            h: h as i16,
            trees: trees,
            tiles: tiles,
            tn_w: 0,
            tn_h: 0,
            total_neighbors: 0,
            filter: 0,
        };

        fg.construct(Square::new(0, 0, w as i16, h as i16));

        fg
    }

    pub fn open(&mut self, x: i16, y: i16) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            let ix = y as usize * self.w as usize + x as usize;
            if self.tiles[ix].status {
                return;
            }
            self.tiles[ix].status = true;
            let w = self.w;
            let h = self.h;
            self.traverse_open(0, x, y, Square::new(0, 0, w, h));
        }
    }

    fn traverse_open(&mut self, tree_ix: usize, ox: i16, oy: i16, sqr: Square) {
        if sqr.w > 1 && sqr.h > 1 {
            let num_closed = self.trees[tree_ix].num_closed;

            // If num_closed is 1 then this trees closed count is about to be
            // reduced to 0 and we should make it the parent of all its tiles
            if num_closed == 1 {
                self.set_parents(tree_ix, sqr);
                self.traverse_reduce_num_closed(tree_ix, ox, oy, sqr);
            }
            else {
                self.trees[tree_ix].num_closed -= 1;

                match sqr.quadrant(ox, oy) {
                    Quadrant::NE => {
                        let ne = self.trees[tree_ix].ne;
                        self.traverse_open(ne as usize, ox, oy, sqr.ne());
                    }
                    Quadrant::SE => {
                        let se = self.trees[tree_ix].se;
                        self.traverse_open(se as usize, ox, oy, sqr.se());
                    }
                    Quadrant::NW => {
                        let nw = self.trees[tree_ix].nw;
                        self.traverse_open(nw as usize, ox, oy, sqr.nw());
                    }
                    Quadrant::SW => {
                        let sw = self.trees[tree_ix].sw;
                        self.traverse_open(sw as usize, ox, oy, sqr.sw());
                    }
                }
            }
        }
    }

    fn traverse_reduce_num_closed(&mut self, tree_ix: usize, ox: i16, oy: i16, sqr: Square) {
        if sqr.w > 1 && sqr.h > 1 {
            self.trees[tree_ix].num_closed -= 1;
            match sqr.quadrant(ox, oy) {
                Quadrant::NE => {
                    let ne = self.trees[tree_ix].ne;
                    self.traverse_reduce_num_closed(ne as usize, ox, oy, sqr.ne());
                }
                Quadrant::SE => {
                    let se = self.trees[tree_ix].se;
                    self.traverse_reduce_num_closed(se as usize, ox, oy, sqr.se());
                }
                Quadrant::NW => {
                    let nw = self.trees[tree_ix].nw;
                    self.traverse_reduce_num_closed(nw as usize, ox, oy, sqr.nw());
                }
                Quadrant::SW => {
                    let sw = self.trees[tree_ix].sw;
                    self.traverse_reduce_num_closed(sw as usize, ox, oy, sqr.sw());
                }
            }
        }
    }

    fn set_parents(&mut self, parent: usize, sqr: Square) {
        if sqr.w > 1 && sqr.h > 1 {
            for y in sqr.y .. sqr.y + sqr.h {
                for x in sqr.x .. sqr.x + sqr.w {
                    let ix = y as usize * self.w as usize + x as usize;
                    self.tiles[ix].parent = parent as u32;
                }
            }
        }
    }

    pub fn close(&mut self, x: i16, y: i16) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            let ix = y as usize * self.w as usize + x as usize;
            if !self.tiles[ix].status {
                return;
            }
            self.tiles[ix].status = false;
            let w = self.w;
            let h = self.h;
            self.traverse_closed(0, x, y, Square::new(0, 0, w, h));
        }
    }

    fn traverse_closed(&mut self, tree: usize, ox: i16, oy: i16, sqr: Square) {
        if sqr.w > 1 && sqr.h > 1 {
            let num_closed = self.trees[tree].num_closed;
            self.trees[tree].num_closed += 1;
            let nw = self.trees[tree].nw as usize;
            let ne = self.trees[tree].ne as usize;
            let sw = self.trees[tree].sw as usize;
            let se = self.trees[tree].se as usize;
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
        if sqr.w > 1 && sqr.h > 1 {
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
            self.trees[ix].nw = nw as u32;
            self.trees[ix].ne = ne as u32;
            self.trees[ix].sw = sw as u32;
            self.trees[ix].se = se as u32;
            ix
        }
        else {
            sqr.y as usize * self.w as usize + sqr.x as usize
        }
    }

    fn verify(&self, tree_ix: usize, sqr: Square) -> bool {
        if sqr.w > 1 && sqr.h > 1 {
            let tree = self.trees[tree_ix];
            let mut num_closed = 0;

            for y in sqr.y .. sqr.y + sqr.h {
                for x in sqr.x .. sqr.x + sqr.w {
                    let ix = y as usize * self.w as usize + x as usize;

                    if !self.tiles[ix].status {
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
        let ix = oy as usize * self.w as usize + ox as usize;
        let parent_ix = self.tiles[ix].parent as usize;
        self.trees[parent_ix]
    }

    fn neighbors(&mut self, ox: i16, oy: i16) -> Vec<Square> {
        let mut vec = Vec::new();

        if ox < 0 || oy < 0 || ox > self.w || oy > self.h {
            return vec;
        }

        let sqr = {
            let parent = self.get_parent(ox, oy);

            if parent.num_closed > 0 {
                Square::new(ox, oy, 1, 1)
            }
            else {
                parent.sqr
            }
        };

        let n = min_i16(self.h - 1, sqr.y + sqr.h);
        let e = min_i16(self.w - 1, sqr.x + sqr.w);
        let s = max_i16(0, sqr.y - 1);
        let w = max_i16(0, sqr.x - 1);

        let mut prev = Square::new(-1, -1, -1, -1);

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
            else {
                self.filter += 1;
            }
        }

        self.total_neighbors += vec.len();
        self.tn_w += sqr.w as usize;
        self.tn_h += sqr.h as usize;

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

#[test]
fn fine_grid_test() {
    println!("Creating Fine Grid");
    let mut fg = FineGrid::new();
    println!("Created Fine Grid");

    for y in 0..1 {
        for x in 0..1 {
            fg.close(x as i16, y as i16);
            fg.open(x as i16, y as i16);
            let ix = y * (fg.w as usize) + x;
            assert!(fg.tiles[ix].status);
            assert_eq!(fg.tiles[ix].parent, 0);
        }
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
    for _ in 0..1 << 20 {
        let x = rng.gen_range(0, 1 << 14);
        let y = rng.gen_range(0, 1 << 14);
        fg.close(x as i16, y as i16);
    }
    let end = PreciseTime::now();
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nClose Grid: {}ms", elapsed);

    fg.verify(0, Square::new(0, 0, 1 << 14, 1 << 14));

    let start = PreciseTime::now();
    for _ in 0..1 << 20 {
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

    let avg_w = fg.tn_w as f64 / (1 << 14) as f64;
    let avg_h = fg.tn_h as f64 / (1 << 14) as f64;
    println!("\nStats: {:?}, {:?}, {:?}, {:?}", avg_w, avg_h, fg.total_neighbors, fg.filter);

    fg.verify(0, Square::new(0, 0, 1 << 14, 1 << 14));
}