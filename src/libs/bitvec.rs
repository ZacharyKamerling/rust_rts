extern crate test;
extern crate rand;

use std::usize;
use libs::bitvec::test::Bencher;
use self::rand::Rng;

#[derive(Clone)]
struct BitVec {
    vec: Vec<u64>,
}

impl BitVec {
    fn new(size: usize) -> BitVec {
        let size = {
            if size % 64 == 0 {
                size / 64
            }
            else {
                size / 64 + 1
            }
        };

        let mut vec = Vec::with_capacity(size);

        for _ in 0..size {
            vec.push(0);
        }

        BitVec { vec: vec }
    }

    fn get(&self, ix: usize) -> bool {
        unsafe {
            let jx = ix >> 6;
            let kx = *self.vec.get_unchecked(jx);
            let lx = kx & (1 << (ix & 63));
            lx > 0
        }
    }

    fn set(&mut self, ix: usize, v: bool) {
        unsafe {
            let jx = ix >> 6;
            let kx = *self.vec.get_unchecked(jx);
            let lx = if v {
                kx | (1 << (ix & 63))
            }
            else {
                kx & !(1 << (ix & 63))
            };

            *self.vec.get_unchecked_mut(jx) = lx;
        }
    }

    fn set_check(&self, other: &BitVec) -> bool {
        if self.vec.len() != other.vec.len() {
            false
        }
        else {
            for i in 0..self.vec.len() {
                let a = self.vec[i];
                let b = other.vec[i];

                if a & b != b {
                    return false;
                }
            }

            true
        }
    }

    fn bitwise_or(&mut self, other: &BitVec) {
        if self.vec.len() != other.vec.len() {
            panic!("bitwise_or: BitVec is different length.");
        }
        else {
            for i in 0..self.vec.len() {
                let a = self.vec[i];
                let b = other.vec[i];

                unsafe {
                    *self.vec.get_unchecked_mut(i) = a | b;
                }
            }
        }
    }

    fn bitwise_and(&mut self, other: &BitVec) {
        if self.vec.len() != other.vec.len() {
            panic!("bitwise_and: BitVec is different length.");
        }
        else {
            for i in 0..self.vec.len() {
                let a = self.vec[i];
                let b = other.vec[i];

                unsafe {
                    *self.vec.get_unchecked_mut(i) = a & b;
                }
            }
        }
    }
}

#[derive(Clone)]
struct BitGrid {
    w: isize,
    h: isize,
    vec: BitVec,
}

impl BitGrid {
    pub fn new(w: usize, h: usize) -> BitGrid {
        BitGrid {
            w: w as isize,
            h: h as isize,
            vec: BitVec::new(w * h),
        }
    }

    pub fn get(&self, (x,y): (isize,isize)) -> bool {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.vec.get((y * self.w + x) as usize)
        }
        else {
            false
        }
    }

    pub fn set(&mut self, (x,y): (isize,isize), v: bool) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.vec.set((y * self.w + x) as usize, v)
        }
        else {
            panic!("Trying to set a bit outside bounds (x, y): ({:?}, {:?})", x, y);
        }
    }

    pub fn bitwise_and(&mut self, other: &BitGrid) {
        if self.w != other.w || self.h != other.h {
            panic!("bitwise_and: BitGrid is different width or height.");
        }
        else {
            self.vec.bitwise_and(&other.vec);
        }
    }

    pub fn bitwise_or(&mut self, other: &BitGrid) {
        if self.w != other.w || self.h != other.h {
            panic!("bitwise_or: BitGrid is different width or height.");
        }
        else {
            self.vec.bitwise_or(&other.vec);
        }
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
    bv.set(62, false);
    bv.set(63, false);
    bv.set(64, false);
    bv.set(65, false);

    assert!(   bv.get(0));
    assert!( ! bv.get(1));
    assert!(   bv.get(2));
    assert!( ! bv.get(3));
    assert!( ! bv.get(4));
    assert!(   bv.get(5));
    assert!( ! bv.get(6));
    assert!(   bv.get(7));
    assert!(   bv.get(8));
    assert!(   bv.get(61));
    assert!( ! bv.get(62));
    assert!( ! bv.get(63));
    assert!( ! bv.get(64));
    assert!( ! bv.get(65));
    assert!(   bv.get(66));
}

#[bench()]
fn create_bitvec(b: &mut Bencher) {
    b.iter(|| {
        BitVec::new(65536);
    });
}

#[bench()]
fn search_bitvec(b: &mut Bencher) {
    let bitvec = BitVec::new(65536);
    let mut a = 0;
    b.iter(|| {
        for i in 0..65536 {
            if bitvec.get(i) {
                a += 1;
            }
        }
    });
}

#[bench()]
fn set_bitvec(b: &mut Bencher) {
    let mut bitvec = BitVec::new(65536);
    b.iter(|| {
        for i in 0..65536 {
            bitvec.set(i, true);
        }
    });
}

#[derive(Clone)]
struct LOS {
    w: u8,
    h: u8,
    shading: Vec<BitGrid>,
}

impl LOS {
    pub fn new(w: u8, h: u8) -> LOS {
        let (w,h) = (w as usize, h as usize);
        let mut shading = Vec::with_capacity(w*h);

        for y in 0..w {
            for x in 0..h {
                shading.push(BitGrid::new(w, h));

                if x == (w - 1) || y == (h - 1) {
                    let line = trace((x as isize, y as isize), (0,0));

                    for i in 0..line.len() {
                        let ix = line[i].1 as usize * w + line[i].0 as usize;

                        for j in 0..i {
                            let xy = line[j];
                            shading[ix].set(xy, true);
                        }
                    }
                }
            }
        }

        LOS {
            w: w as u8,
            h: h as u8,
            shading: shading,
        }
    }

    // Given a BitGrid where 'true' values are los blockers,
    // return a BitGrid where 'true' values are not visible.
    pub fn los_grid(&self, state: &BitGrid) -> BitGrid {
        if state.w != self.w as isize || state.h != self.h as isize {
            panic!("los_grid: BitGrid is different width or height.");
        }

        let mut output = BitGrid::new(state.w as usize, state.h as usize);
        for y in 0..state.h {
            for x in 0..state.w {
                if !output.get((x,y)) && state.get((x,y)) {
                    output.set((x,y), true);

                    let ix = (y * state.w + x) as usize;
                    output.bitwise_or(&self.shading[ix]);
                }
            }
        }

        output
    }
}

fn trace((mut x0, mut y0): (isize, isize), (x1, y1): (isize, isize)) -> Vec<(isize,isize)> {
    let mut vec = Vec::new();
    // Create local variables for moving start point
    //let mut x0 = x0;
    //let mut y0 = y0;

    // Get absolute x/y offset
    let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
    let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };

    // Get slopes
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    // Initialize error
    let mut err = if dx > dy { dx } else { -dy } / 2;
    let mut err2;

    loop {
        vec.push((x0,y0));
        // Check end condition
        if x0 == x1 && y0 == y1 {
            return vec;
        };

        // Store old error
        err2 = 2 * err;

        if err2 > -dx {
            err -= dy;
            x0 += sx;
        }

        if err2 < dy {
            err += dx;
            y0 += sy;
        }
    }
}

#[bench()]
fn create_los(b: &mut Bencher) {
    b.iter(|| {
        LOS::new(100,100);
    });
}

#[bench()]
fn use_los(b: &mut Bencher) {
    let (w,h) = (100,100);
    let mut state = BitGrid::new(w as usize, h as usize);
    let mut rng = rand::thread_rng();
    let los = LOS::new(w as u8, h as u8);

    for y in 0..w {
        for x in 0..h {
            state.set((x,y), false);
        }
    }

    b.iter(|| {
        for _ in 0..w {
            let x = rng.gen_range(0, w);
            let y = rng.gen_range(0, h);
            state.set((x,y), true);
        }
        los.los_grid(&state);
    });
}

pub fn los_visual() {
    let (w,h) = (32,32);
    let mut state = BitGrid::new(w as usize, h as usize);
    let mut rng = rand::thread_rng();
    let los = LOS::new(w as u8, h as u8);

    for _ in 0..w {
        let x = rng.gen_range(0, w);
        let y = rng.gen_range(0, h);
        state.set((x,y), true);
    }

    let grid = los.los_grid(&state);

    print!("---------------------------------------------------------------");
    for y in 0..w {
        println!("");
        for x in 0..h {
            if state.get((x,y)) {
                print!("X ");
            }
            else if grid.get((x,y)) {
                print!("x ");
            }
            else {
                print!("  ");
            }
        }
    }
}