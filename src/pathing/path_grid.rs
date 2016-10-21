extern crate fnv;
extern crate rand;
extern crate time;

use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::HashMap;
use std::cmp::{Ordering};
use std::hash::BuildHasherDefault;
use self::fnv::FnvHasher;
use self::rand::Rng;
use self::time::{PreciseTime};

type Point = (isize,isize);

#[derive(Clone,Debug,Eq,PartialEq)]
struct Node {
    f:              isize,
    g:              isize,
    xy:             Point,
    parent:         Point,
    direction:      Direction,
}

#[derive(Clone,Debug)]
struct Jumps {
    nj: u16,
    ej: u16,
    sj: u16,
    wj: u16,
}

pub struct PathGrid {
    w: isize,
    h: isize,
    states: Vec<u8>,
    jumps: Vec<Jumps>,
    // Avoid allocations by using these pre-allocated collections
    open: BinaryHeap<Node>,
    closed: HashSet<Point>,
    expand: Vec<(Point, Direction)>,
    came_from: HashMap<Point,Point,BuildHasherDefault<FnvHasher>>,
}

impl Clone for PathGrid {
    fn clone(&self) -> PathGrid {
        let fnv = BuildHasherDefault::<FnvHasher>::default();
        let wth = (self.w * self.h) as usize;
        let wph = (self.w + self.h) as usize;
        let mut pg = PathGrid
                { w: self.w
                , h: self.h
                , states: Vec::with_capacity(wth)
                , jumps: Vec::with_capacity(wth)
                , open: BinaryHeap::with_capacity(wph)
                , closed: HashSet::with_capacity(wph)
                , expand: Vec::with_capacity(4)
                , came_from: HashMap::with_capacity_and_hasher(wph, fnv)
                };
        for _ in 0..wth {
            pg.states.push(0);
            pg.jumps.push(Jumps {nj: 0, ej: 0, sj: 0, wj: 0});
        }
        pg
    }
}

impl PathGrid {

    pub fn new(w: usize, h: usize) -> PathGrid {
        let fnv = BuildHasherDefault::<FnvHasher>::default();
        let wth = w * h;
        let wph = w + h;
        let mut pg = PathGrid
                { w: w as isize
                , h: h as isize
                , states: Vec::with_capacity(wth)
                , jumps: Vec::with_capacity(wth)
                , open: BinaryHeap::with_capacity(wph)
                , closed: HashSet::with_capacity(wph)
                , expand: Vec::with_capacity(4)
                , came_from: HashMap::with_capacity_and_hasher(wph, fnv)
                };
        for _ in 0..wth {
            pg.states.push(0);
            pg.jumps.push(Jumps {nj: 0, ej: 0, sj: 0, wj: 0});
        }
        pg
    }

    pub fn find_path(&mut self, start: (isize,isize), goal: (isize,isize)) -> Option<Vec<(isize,isize)>> {
        self.reset();

        if self.is_line_open(start,goal) {
            let mut vec = Vec::new();
            vec.push(goal);
            return Some(vec);
        }

        if !self.is_open(start) || !self.is_open(goal) || start == goal {
            return None;
        }

        self.init_open(start, goal);

        while let Some(current) = self.open.pop() {

            if self.closed.contains(&current.xy) {
                continue;
            }

            if self.lines_up(current.xy, goal) {
                let vec = reconstruct(goal, &self.came_from, current.xy);
                return Some(vec);
            }

            self.closed.insert(current.xy);
            self.expand_node(current.xy, goal, current.direction);

            for &(neighbor,dir) in self.expand.iter() {
                if !self.closed.contains(&neighbor) {
                    let g = current.g + dist_between(current.xy, neighbor);
                    let f = g + dist_between(goal, neighbor);
                    let node = Node {
                        f: f,
                        g: g,
                        xy: neighbor,
                        parent: current.xy,
                        direction: dir,
                    };
                    self.open.push(node);
                    self.came_from.insert(neighbor, current.xy);
                }
            }
        }

        return None;
    }

    pub fn is_open(&self, (x,y): (isize,isize)) -> bool {
        (x >= 0) &
        (y >= 0) &
        (x < self.w) &
        (y < self.h) &&
        self.states[(y * self.w + x) as usize] == 0
    }

    pub fn is_line_open(&self, (x0,y0): (isize,isize), (x1,y1): (isize,isize)) -> bool {

        // Create local variables for moving start point
        let mut x0 = x0;
        let mut y0 = y0;

        // Get absolute x/y offset
        let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
        let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };

        // Get slopes
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        // Initialize error
        let mut err = if dx > dy { dx } else {-dy} / 2;
        let mut err2;

        loop {
            // Set pixel
            if !self.is_open((x0,y0)) {
                return false;
            }

            // Check end condition
            if x0 == x1 && y0 == y1 { return true; };

            // Store old error
            err2 = 2 * err;

            // Adjust error and start position
            if err2 > -dx && err2 < dy && !self.is_open((x0 + sx, y0)) && !self.is_open((x0, y0 + sy)) {
                return false;
            }

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

    fn init_open(&mut self, xy: Point, goal: Point) {
        let ne = translate(1, NORTHEAST, xy);
        let nw = translate(1, NORTHWEST, xy);
        let se = translate(1, SOUTHEAST, xy);
        let sw = translate(1, SOUTHWEST, xy);

        self.init_diag(ne, goal, NORTHEAST);
        self.init_diag(se, goal, SOUTHEAST);
        self.init_diag(sw, goal, SOUTHWEST);
        self.init_diag(nw, goal, NORTHWEST);

        self.init_axis(xy, goal, NORTH);
        self.init_axis(xy, goal, EAST);
        self.init_axis(xy, goal, SOUTH);
        self.init_axis(xy, goal, WEST);
    }

    fn init_axis(&mut self, xy: Point, goal: Point, dir: Direction) {
        match self.get_jump(dir, xy) {
            Some(jump) => {
                let node = Node {
                    f: 10 + dist_between(jump, goal),
                    g: 10,
                    xy: jump,
                    parent: xy,
                    direction: dir,
                };
                self.open.push(node);
            }
            _ => ()
        }
    }


    fn init_diag(&mut self, xy: Point, goal: Point, ne: Direction) {
        let w = rotate_cc(DEG_135, ne);
        let s = rotate_c(DEG_135, ne);
        let w_xy = translate(1, w, xy);
        let s_xy = translate(1, s, xy);

        if self.is_open(w_xy) || self.is_open(s_xy) {
            match self.search_diag(xy, goal, ne) {
                Some((jump,_)) => {
                    let node = Node {
                        f: 14 + dist_between(jump, goal),
                        g: 14,
                        xy: jump,
                        parent: xy,
                        direction: ne,
                    };
                    self.open.push(node);
                }
                _ => ()
            }
        }
    }

    fn expand_node(&mut self, xy: Point, goal: Point, dir: Direction) {
        self.expand.clear();
        match dir {
            NORTH | EAST | SOUTH | WEST => {
                Self::expand_axis(self, xy, dir);
            }
            NORTHEAST | SOUTHEAST | SOUTHWEST | NORTHWEST => {
                Self::expand_diag(self, xy, goal, dir);
            }
            _             => panic!("Expansion failed with a bad Direction.")
        }
    }

    fn expand_axis(&mut self, xy: Point, n: Direction) {
        let e  = rotate_c(DEG_90, n);
        let w  = rotate_cc(DEG_90, n);
        let nw = rotate_cc(DEG_45, n);
        let ne = rotate_c(DEG_45, n);
        let n_xy = translate(1, n, xy);
        let e_xy = translate(1, e, xy);
        let w_xy = translate(1, w, xy);
        let ne_xy = translate(1, ne, xy);
        let nw_xy = translate(1, nw, xy);

        if self.is_open(n_xy) {

            match self.get_jump(n, xy) {
                Some(n_jump) => {
                    self.expand.push((n_jump, n));
                }
                _            => (),
            }

            if self.is_open(nw_xy) && !self.is_open(w_xy) {
                self.expand.push((nw_xy, nw));
            }

            if self.is_open(ne_xy) && !self.is_open(e_xy) {
                self.expand.push((ne_xy, ne));
            }
        }
    }

    fn expand_diag(&mut self, xy: Point, goal: Point, ne: Direction) {
        let n = rotate_cc(DEG_45, ne);
        let e = rotate_c(DEG_45, ne);
        let w = rotate_cc(DEG_135, ne);
        let s = rotate_c(DEG_135, ne);
        let nw = rotate_cc(DEG_90, ne);
        let se = rotate_c(DEG_90, ne);
        let ne_xy = translate(1, ne, xy);
        let nw_xy = translate(1, nw, xy);
        let se_xy = translate(1, se, xy);
        let n_xy = translate(1, n, xy);
        let e_xy = translate(1, e, xy);
        let w_xy = translate(1, w, xy);
        let s_xy = translate(1, s, xy);
        let n_open = self.is_open(n_xy);
        let s_open = self.is_open(s_xy);
        let e_open = self.is_open(e_xy);
        let w_open = self.is_open(w_xy);
        let se_open = self.is_open(se_xy);
        let nw_open = self.is_open(nw_xy);

        if !w_open && n_open && nw_open {
            let opt = self.search_diag(nw_xy, goal, nw);
            self.push_option(opt);
        }
        else if !s_open && e_open && se_open {
            let opt = self.search_diag(se_xy, goal, se);
            self.push_option(opt);
        }

        match self.get_jump(n, xy) {
            Some(n_jump) => {
                self.expand.push((n_jump, n));
            }
            _ => ()
        }

        match self.get_jump(e, xy) {
            Some(e_jump) => {
                self.expand.push((e_jump, e));
            }
            _ => ()
        }

        let opt = self.search_diag(ne_xy, goal, ne);
        self.push_option(opt);
    }

    // Should be good
    fn search_diag(&self, mut xy: Point, goal: Point, ne: Direction) -> Option<(Point, Direction)> {
        let n = rotate_cc(DEG_45, ne);
        let e = rotate_c(DEG_45, ne);

        loop {
            let n_xy = translate(1, n, xy);
            let e_xy = translate(1, e, xy);

            if !self.is_open(xy) || (!self.is_open(n_xy) && !self.is_open(e_xy)) {
                return None;
            }

            if  self.get_jump(n, xy).is_some() ||
                self.get_jump(e, xy).is_some() ||
                self.is_diag_jump(ne, xy) ||
                self.lines_up(xy, goal)
            {
                return Some((xy, ne));
            }

            xy = translate(1, ne, xy);
        }
    }

    fn lines_up(&self, start: Point, goal: Point) -> bool {
        let (sx,sy) = start;
        let (gx,gy) = goal;

        if sx != gx && sy != gy {
            return false;
        }

        if sx == gx && sy == gy {
            return true;
        }

        let dir =
            if gy > sy {
                NORTH
            }
            else if gy < sy {
                SOUTH
            }
            else if gx > sx {
                EAST
            }
            else {
                WEST
            };

        let mut xy = start;
        loop {
            xy = translate(1, dir, xy);
            if xy == goal {
                return true;
            }
            if !self.is_open(xy) {
                return false;
            }
        }
    }

    fn push_option(&mut self, opt: Option<(Point,Direction)>) {
        match opt {
            Some(a) => self.expand.push(a),
            None => ()
        }
    }

    fn is_axis_jump(&self, dir: Direction, xy: Point) -> bool {
        let w  = translate(1, rotate_cc(DEG_90, dir), xy);
        let nw = translate(1, rotate_cc(DEG_45, dir), xy);
        let n  = translate(1, dir, xy);
        let s  = translate(1, rotate_c(DEG_180, dir), xy);
        let ne = translate(1, rotate_c(DEG_45, dir), xy);
        let e  = translate(1, rotate_c(DEG_90, dir), xy);

        self.is_open(xy) && self.is_open(n) && self.is_open(s) &&
        (!self.is_open(w) && self.is_open(nw) || !self.is_open(e) && self.is_open(ne))
    }

    fn is_diag_jump(&self, dir: Direction, xy: Point) -> bool {
        let n  = translate(1, rotate_cc(DEG_45, dir), xy);
        let s  = translate(1, rotate_c(DEG_135, dir), xy);
        let e  = translate(1, rotate_c(DEG_45, dir), xy);
        let w  = translate(1, rotate_cc(DEG_135, dir), xy);
        let nw = translate(1, rotate_cc(DEG_90, dir), xy);
        let se = translate(1, rotate_c(DEG_90, dir), xy);

        match (self.is_open(w), self.is_open(s)) {
            (false, true) => self.is_open(n) && self.is_open(nw),
            (true, false) => self.is_open(e) && self.is_open(se),
            _             => false
        }
    }

    fn get_jump(&self, dir: Direction, (x,y): Point) -> Option<Point> {
        let ix = (y * self.w + x) as usize;
        match dir {
            NORTH =>
                {
                    let jmp_offset = self.jumps[ix].nj;
                    if jmp_offset > 0 {
                        Some((x, y + jmp_offset as isize))
                    }
                    else {
                        None
                    }
                }
            EAST =>
                {
                    let jmp_offset = self.jumps[ix].ej;
                    if jmp_offset > 0 {
                        Some((x + jmp_offset as isize, y))
                    }
                    else {
                        None
                    }
                }
            SOUTH =>
                {
                    let jmp_offset = self.jumps[ix].sj;
                    if jmp_offset > 0 {
                        Some((x, y - jmp_offset as isize))
                    }
                    else {
                        None
                    }
                }
            WEST =>
                {
                    let jmp_offset = self.jumps[ix].wj;
                    if jmp_offset > 0 {
                        Some((x - jmp_offset as isize, y))
                    }
                    else {
                        None
                    }
                }
            _ => panic!("get_jump was given a diag Direction.")
        }
    }

    fn reset(&mut self) {
        self.open.clear();
        self.closed.clear();
        self.expand.clear();
        self.came_from.clear();
    }

    fn print_dir(&self, dir: Direction) {

        if self.w > 9 || self.h > 9 {
            println!("Cannot print PathGrid. Its w and h cannot exceed 9.");
            return;
        }

        let mut y = self.h - 1;
        match dir {
            NORTH => println!(" ======== N ========"),
            EAST => println!(" ======== E ========"),
            SOUTH => println!(" ======== S ========"),
            WEST => println!(" ======== W ========"),
            _ => println!("!BAD DIRECTION!"),
        }
        while y >= 0 {
            for x in 0..self.w {
                let xy = (x,y);
                if self.is_open(xy) {
                    if self.is_axis_jump(dir, xy) {
                        print!(" x");
                    }
                    else {
                        let jump_dist = self.get_jump_dist(dir, xy);
                        if jump_dist != 0 {
                            print!(" {}", jump_dist);
                        }
                        else {
                            print!("  ");
                        }
                    }
                }
                else {
                    print!(" #");
                }
            }
            println!("");
            y -= 1;
        }
    }

    pub fn open_point(&mut self, (x,y): (isize,isize)) {
        if self.is_open((x,y)) {
            return;
        }

        self.states[(y * self.w + x) as usize] = 0;

        for ux in 0..3 {
            for uy in 0..3 {
                let ix = x + (ux as isize) - 1;
                let iy = y + (uy as isize) - 1;
                let xy = (ix,iy);
                self.continue_jumps(NORTH, xy);
                self.continue_jumps(EAST, xy);
                self.continue_jumps(SOUTH, xy);
                self.continue_jumps(WEST, xy);
                self.set_jumps(NORTH, xy);
                self.set_jumps(EAST, xy);
                self.set_jumps(SOUTH, xy);
                self.set_jumps(WEST, xy);
            }
        }
    }

    fn correct_close_jumps(&mut self, n: Direction, start: Point) {
        let nw = rotate_cc(DEG_45, n);
        let ne = rotate_c(DEG_45, n);
        let w = rotate_cc(DEG_90, n);
        let e = rotate_c(DEG_90, n);
        let s = rotate_c(DEG_180, n);

        let nw_xy = translate(1, nw, start);
        let ne_xy = translate(1, ne, start);

        self.set_jumps(w, ne_xy);
        self.set_jumps(e, ne_xy);
        self.set_jumps(s, ne_xy);

        self.set_jumps(w, nw_xy);
        self.set_jumps(e, nw_xy);
        self.set_jumps(s, nw_xy);

        self.set_jumps(n, start);
    }

    /* Scans north looking for north jump then goes south
    ** setting jump distances until it encounters another north jump.
    ** If no north jump is encountered, then the jump distances going south
    ** are set to 0 until it encounters another north jump.
    */
    fn continue_jumps(&mut self, n: Direction, start: Point) {
        let s = rotate_c(DEG_180, n);
        let mut xy = start;
        let mut jump_dist = 1;

        while self.is_open(xy) {
            if self.is_axis_jump(n, xy) {
                xy = translate(1, s, xy);

                while self.is_open(xy) {
                    self.set_jump_dist(n, xy, jump_dist);

                    if self.is_axis_jump(n, xy) {
                        return;
                    }
                    jump_dist += 1;
                    xy = translate(1, s, xy);
                }
                return;
            }
            xy = translate(1, n, xy);
        }

        xy = translate(1, s, xy);

        while self.is_open(xy) {
            self.set_jump_dist(n, xy, 0);
            if self.is_axis_jump(n, xy) {
                return;
            }
            xy = translate(1, s, xy);
        }
    }

    pub fn close_point(&mut self, (x,y): (isize,isize)) {
        if !self.is_open((x,y)) {
            return;
        }
        let n = (x, y + 1);
        let e = (x + 1, y);
        let s = (x, y - 1);
        let w = (x - 1, y);
        let ix = (y * self.w + x) as usize;
        self.states[ix] = 1;

        self.set_jumps(NORTH, e);
        self.set_jumps(NORTH, w);
        self.set_jumps(EAST, n);
        self.set_jumps(EAST, s);
        self.set_jumps(SOUTH, e);
        self.set_jumps(SOUTH, w);
        self.set_jumps(WEST, n);
        self.set_jumps(WEST, s);
        self.clear_jumps(NORTH, s);
        self.clear_jumps(EAST, w);
        self.clear_jumps(SOUTH, n);
        self.clear_jumps(WEST, e);
    }

    fn set_jumps(&mut self, dir: Direction, mut xy: Point) {
        if self.is_open(xy) {
            if self.is_axis_jump(dir, xy) {
                let opp_dir = rotate_c(DEG_180, dir);
                xy = translate(1, opp_dir, xy);
                let mut jump_dist = 1;

                while self.is_open(xy) {
                    self.set_jump_dist(dir, xy, jump_dist);
                    if self.is_axis_jump(dir, xy) {
                        return;
                    }
                    jump_dist += 1;
                    xy = translate(1, opp_dir, xy);
                }
            }
        }
    }

    fn clear_jumps(&mut self, dir: Direction, start: Point) {
        let opp_dir = rotate_c(DEG_180, dir);
        let mut xy = start;

        while self.is_open(xy) {
            self.set_jump_dist(dir, xy, 0);
            if self.is_axis_jump(dir, xy) {
                return;
            }
            xy = translate(1, opp_dir, xy);
        }
    }

    fn set_jump_dist(&mut self, dir: Direction, (x,y): Point, dist: u16) {
        let ix = (y * self.w + x) as usize;
        match dir {
            NORTH => self.jumps[ix].nj = dist,
            EAST => self.jumps[ix].ej = dist,
            SOUTH => self.jumps[ix].sj = dist,
            WEST => self.jumps[ix].wj = dist,
            _ => panic!("set_jump_dist was given a diag Direction.")
        }
    }

    fn get_jump_dist(&self, dir: Direction, (x,y): Point) -> u16 {
        let ix = (y * self.w + x) as usize;
        match dir {
            NORTH => self.jumps[ix].nj,
            EAST => self.jumps[ix].ej,
            SOUTH => self.jumps[ix].sj,
            WEST => self.jumps[ix].wj,
            _ => panic!("set_jump_dist was given a diag Direction.")
        }
    }
}

fn dist_between((x0,y0): Point, (x1,y1): Point) -> isize {
    let x_dif = x0 - x1;
    let y_dif = y0 - y1;
    (f32::sqrt((x_dif * x_dif + y_dif * y_dif) as f32) * 10.0) as isize
}

pub fn bench() {
    let mut rng = rand::thread_rng();
    let w: isize = 1024;
    let h: isize = 1024;
    let mut jg = PathGrid::new(w as usize, h as usize);

    println!("Generating map.");

    for _ in 0..((w * h) / ((w + h) / 2)) {
        let x = rng.gen_range(0,w);
        let y = rng.gen_range(0,h);

        for ix in 0..10 {
            for iy in 0..10 {
                jg.close_point((x + ix, y + iy));
            }
        }
    }

    let mut total_len = 0;
    let start = PreciseTime::now();

    for _ in 0..10000 {
        let x0 = rng.gen_range(0, w / 2);
        let y0 = rng.gen_range(0, h / 2);
        let x1 = rng.gen_range(w / 2, w);
        let y1 = rng.gen_range(h / 2, h);

        if let Some(vec) = jg.find_path((x0,y0), (x1,y1)) {
            total_len += vec.len();
        }
    }

    let end = PreciseTime::now();
    let mili = 1000000.0;

    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nFind path time: {}ms", elapsed);
    println!("Avg Path Len: {}", total_len / 10000);
}

pub fn test() {
    let w: isize = 9;
    let h: isize = 9;
    let mut rng = rand::thread_rng();
    let mut jg = PathGrid::new(w as usize, h as usize);

    for _ in 0..100 {

        for _ in 0..1 {
            let x = rng.gen_range(0, w);
            let y = rng.gen_range(0, h);
            jg.close_point((x,y));
        }

        for _ in 0..9 {
            let x = rng.gen_range(0, w);
            let y = rng.gen_range(0, h);
            jg.open_point((x,y));
        }
    }

    jg.print_dir(NORTH);
    jg.print_dir(SOUTH);
    jg.print_dir(EAST);
    jg.print_dir(WEST);
}

fn reconstruct(goal: Point, closed: &HashMap<Point,Point,BuildHasherDefault<FnvHasher>>, mut xy: Point) -> Vec<Point> {
    let mut vec = Vec::with_capacity(512);
    vec.push(goal);

    loop {
        vec.push(xy);
        match closed.get(&xy) {
            Some(next) => {
                xy = *next;
            }
            None => break
        }
    }
    vec
}

impl Ord for Node {
    #[inline]
    fn cmp(&self, other: &Node) -> Ordering {
        // Notice that the we flip the ordering here
        other.f.cmp(&self.f)
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Node {
    #[inline]
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Direction(isize);

#[derive(Clone,Copy,Debug)]
struct Degree(isize);

const DEG_45:       Degree = Degree(1);
const DEG_90:       Degree = Degree(2);
const DEG_135:      Degree = Degree(3);
const DEG_180:      Degree = Degree(4);
const NORTH:        Direction = Direction(0);
const EAST:         Direction = Direction(2);
const SOUTH:        Direction = Direction(4);
const WEST:         Direction = Direction(6);
const NORTHEAST:    Direction = Direction(1);
const SOUTHEAST:    Direction = Direction(3);
const SOUTHWEST:    Direction = Direction(5);
const NORTHWEST:    Direction = Direction(7);

#[inline]
fn rotate_c(Degree(rot): Degree, Direction(d): Direction) -> Direction {
    let d2 = d + rot;
    if d2 >= 8 {
        return Direction(d2 - 8);
    }
    else {
        return Direction(d2);
    }
}

#[inline]
fn rotate_cc(Degree(rot): Degree, Direction(d): Direction) -> Direction {
    let d2 = d - rot;
    if d2 < 0 {
        return Direction(8 + d2);
    }
    else {
        return Direction(d2);
    }
}

#[inline]
fn translate(n: isize, Direction(dir): Direction, (x,y): (isize,isize)) -> (isize,isize) {
    match dir {
        0 => return (x, y + n),
        1 => return (x + n, y + n),
        2 => return (x + n, y),
        3 => return (x + n, y - n),
        4 => return (x, y - n),
        5 => return (x - n, y - n),
        6 => return (x - n, y),
        7 => return (x - n, y + n),
        _ => panic!("translate was given a bad Direction.")
    }
}