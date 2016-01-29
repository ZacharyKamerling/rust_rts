#![allow(dead_code)]
extern crate rand;
extern crate time;

use std::collections::HashMap;
use std::collections::HashSet;
use self::time::{PreciseTime};
use self::rand::Rng;
use std::cmp::{min,max};
use std::f32;

type Point = (isize,isize);
type DistSearched = f32;

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Direction(isize);

#[derive(Clone,Copy,Debug)]
struct Degree(isize);

#[derive(Clone,Debug)]
struct Node(DistSearched,Point,Direction);

#[derive(Clone,Debug)]
struct Jumps {
    nj: u16,
    ej: u16,
    sj: u16,
    wj: u16,
}

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
const SQRT2:        f32 = 1.4142135623730950488016887242097;

#[derive(Clone)]
pub struct JumpGrid {
    pub w: isize,
    pub h: isize,
    open_vec: Vec<u8>,
    jump_vec: Vec<Jumps>,
    // Avoid allocations by using these pre-allocated collections
    open: PQ<Node>,
    expand: Vec<(Point, Direction)>,
    closed: HashSet<Point>,
    came_from: HashMap<Point,Point>,
}

impl JumpGrid
{
    pub fn new(w: usize, h: usize) -> JumpGrid {
        let mut jg = JumpGrid
                { w: w as isize
                , h: h as isize
                , open_vec: Vec::with_capacity(w * h)
                , jump_vec: Vec::with_capacity(w * h)
                , open: PQ::with_capacity(w + h)
                , expand: Vec::with_capacity(4)
                , closed: HashSet::with_capacity(w + h)
                , came_from: HashMap::with_capacity(w + h)
                };
        for _ in 0..(w * h) {
            jg.open_vec.push(0);
            jg.jump_vec.push(Jumps {nj: 0, ej: 0, sj: 0, wj: 0});
        }
        jg
    }

    pub fn is_open(&self, (x,y): Point) -> bool {
        x >= 0 &&
        y >= 0 &&
        x < self.w &&
        y < self.h &&
        self.open_vec[(y * self.w + x) as usize] == 0
    }

    pub fn find_path(&mut self, start: Point, goal: Point) -> Option<Vec<Point>> {
        let (x0,y0) = start;
        let (x1,y1) = goal;

        if self.is_line_open((x0,y0),(x1,y1)) {
            let mut vec = Vec::new();
            vec.push((x1,y1));
            return Some(vec);
        }

        if !self.is_open(start) || !self.is_open(goal) {
            None
        }
        else {
            self.init_open(start, goal);

            loop {
                match self.open.pop() {
                    Some(Node(dist, xy, dir)) => {

                        if self.lines_up(xy, goal) {
                            let vec = reconstruct(goal, &self.came_from, xy);
                            self.open.vec.clear();
                            self.expand.clear();
                            self.closed.clear();
                            self.came_from.clear();
                            return Some(vec);
                        }
                        self.closed.insert(xy);
                        self.expand_node(xy, goal, dir);

                        for e in self.expand.iter() {
                            let (xy2, dir2) = *e;

                            if !self.closed.contains(&xy2) {
                                let g = dist + dist_between(xy, xy2);
                                let node = Node(g, xy2, dir2);
                                let f = g + dist_between(xy2, goal);

                                self.closed.insert(xy2);
                                self.open.insert((f, node));
                                self.came_from.insert(xy2, xy);
                            }
                        }
                        self.expand.clear();
                    }
                    _ => {
                        self.open.vec.clear();
                        self.expand.clear();
                        self.closed.clear();
                        self.came_from.clear();
                        return None;
                    }
                }
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
                self.closed.insert(jump);
                self.open.insert((1.0 + dist_between(jump, goal), Node(1.0, jump, dir)));
            }
            _ => ()
        }
    }


    fn init_diag(&mut self, xy: Point, goal: Point, dir: Direction) {
        match self.search_diag(xy, goal, dir) {
            Some((jump,_)) => {
                self.closed.insert(jump);
                self.open.insert((SQRT2 + dist_between(jump, goal), Node(SQRT2, jump, dir)));
            }
            _ => ()
        }
    }

    fn expand_node(&mut self, xy: Point, goal: Point, dir: Direction) {
        let Direction(d) = dir;
        match d {
            0 | 2 | 4 | 6 => Self::expand_axis(self, xy, dir),
            1 | 3 | 5 | 7 => Self::expand_diag(self, xy, goal, dir),
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

    pub fn is_line_open(&self, (x0,y0): Point, (x1,y1): Point) -> bool {

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

    pub fn open_or_close_points(&mut self, open_or_close: u8, (x0,y0): Point, (x1,y1): Point) {
        let min_x = min(x0, x1);
        let min_y = min(y0, y1);
        let max_x = max(x0, x1);
        let max_y = max(y0, y1);
        let max_x_bound = max_x + 1;
        let min_x_bound = min_x - 1;
        let max_y_bound = max_y + 1;
        let min_y_bound = min_y - 1;

        let mut n_jumps = Vec::new();
        let mut e_jumps = Vec::new();
        let mut s_jumps = Vec::new();
        let mut w_jumps = Vec::new();

        for y in max(0, min_y_bound)..min(self.h, max_y_bound + 1) {
            for x in max(0, min_x_bound)..min(self.w, max_x_bound + 1) {
                let xy = (x,y);
                n_jumps.push(self.is_axis_jump(Direction(0), xy));
                e_jumps.push(self.is_axis_jump(Direction(2), xy));
                s_jumps.push(self.is_axis_jump(Direction(4), xy));
                w_jumps.push(self.is_axis_jump(Direction(6), xy));
            }
        }

        for y in min_y..max_y_bound {
            for x in min_x..max_x_bound {
                let ix = (y * self.w + x) as usize;
                self.open_vec[ix] = open_or_close;
                if open_or_close > 0 {
                    self.jump_vec[ix] = Jumps {nj: 0, ej: 0, sj: 0, wj: 0};
                }
            }
        }

        let mut jump_c = 0;
        for y in max(0, min_y_bound)..min(self.h, max_y_bound + 1) {
            for x in max(0, min_x_bound)..min(self.w, max_x_bound + 1) {
                let xy = (x,y);

                // Does the following for each direction (n,s,e,w).
                // Check if the point changed from not being a jump point to a jump point.
                // If a node is a new jump point, then set jump values accordingly.
                if !n_jumps[jump_c] && self.is_axis_jump(NORTH, xy) {
                    self.set_jump_ray(SOUTH, xy);
                }
                else if n_jumps[jump_c] && !self.is_axis_jump(NORTH, xy) {
                    self.clear_jump_ray(SOUTH, xy);
                }

                if !e_jumps[jump_c] && self.is_axis_jump(EAST, xy){
                    self.set_jump_ray(WEST, xy);
                }
                else if e_jumps[jump_c] && !self.is_axis_jump(EAST, xy) {
                    self.clear_jump_ray(WEST, xy);
                }

                if !s_jumps[jump_c] && self.is_axis_jump(SOUTH, xy) {
                    self.set_jump_ray(NORTH, xy);
                }
                else if s_jumps[jump_c] && !self.is_axis_jump(SOUTH, xy) {
                    self.clear_jump_ray(NORTH, xy);
                }

                if !w_jumps[jump_c] && self.is_axis_jump(WEST, xy) {
                    self.set_jump_ray(EAST, xy);
                }
                else if w_jumps[jump_c] && !self.is_axis_jump(WEST, xy) {
                    self.clear_jump_ray(EAST, xy);
                }

                jump_c += 1;
            }
        }

        // Only perform these if the area was closed.
        if open_or_close > 0 {

            for x in min_x..max_x_bound {
                self.clear_jump_ray(SOUTH, (x, min_y));
                self.clear_jump_ray(NORTH, (x, max_y));
            }

            for y in min_y..max_y_bound {
                self.clear_jump_ray(WEST, (min_x, y));
                self.clear_jump_ray(EAST, (max_x, y));
            }
        }
        else {

            if max_x_bound < self.w {
                for y in min_y..max_y_bound {
                    self.continue_jump_ray(WEST, (max_x_bound, y));
                }
            }

            if min_x_bound >= 0 {
                for y in min_y..max_y_bound {
                    self.continue_jump_ray(EAST, (min_x_bound, y));
                }
            }

            if max_y_bound < self.h {
                for x in min_x..max_x_bound {
                    self.continue_jump_ray(SOUTH, (x, max_y_bound));
                }
            }

            if min_y_bound >= 0 {
                for x in min_x..max_x_bound {
                    self.continue_jump_ray(NORTH, (x, min_y_bound));
                }
            }
        }
    }

    fn is_axis_jump(&self, dir: Direction, xy: Point) -> bool {
        let w  = translate(1, rotate_cc(DEG_90, dir), xy);
        let nw = translate(1, rotate_cc(DEG_45, dir), xy);
        let n  = translate(1, dir, xy);
        let ne = translate(1, rotate_c(DEG_45, dir), xy);
        let e  = translate(1, rotate_c(DEG_90, dir), xy);

        self.is_open(xy) && self.is_open(n) &&
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

    fn get_jump(&self, Direction(dir): Direction, (x,y): Point) -> Option<Point> {
        let ix = (y * self.w + x) as usize;
        match dir {
            0 =>
                {
                    let jmp_offset = self.jump_vec[ix].nj;
                    if jmp_offset > 0 {
                        Some((x, y + jmp_offset as isize))
                    }
                    else {
                        None
                    }
                }
            2 =>
                {
                    let jmp_offset = self.jump_vec[ix].ej;
                    if jmp_offset > 0 {
                        Some((x + jmp_offset as isize, y))
                    }
                    else {
                        None
                    }
                }
            4 =>
                {
                    let jmp_offset = self.jump_vec[ix].sj;
                    if jmp_offset > 0 {
                        Some((x, y - jmp_offset as isize))
                    }
                    else {
                        None
                    }
                }
            6 =>
                {
                    let jmp_offset = self.jump_vec[ix].wj;
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

    fn set_jump_dist(&mut self, Direction(dir): Direction, (x,y): Point, dist: u16) {
        let ix = (y * self.w + x) as usize;
        match dir {
            0 => self.jump_vec[ix].nj = dist,
            2 => self.jump_vec[ix].ej = dist,
            4 => self.jump_vec[ix].sj = dist,
            6 => self.jump_vec[ix].wj = dist,
            _ => panic!("set_jump_dist was given a diag Direction.")
        }
    }

    fn get_jump_dist(&self, Direction(dir): Direction, (x,y): Point) -> u16 {
        let ix = (y * self.w + x) as usize;
        match dir {
            0 => self.jump_vec[ix].nj,
            2 => self.jump_vec[ix].ej,
            4 => self.jump_vec[ix].sj,
            6 => self.jump_vec[ix].wj,
            _ => panic!("set_jump_dist was given a diag Direction.")
        }
    }

    fn set_jump_ray(&mut self, dir: Direction, start: Point) {
        let opp_dir = rotate_c(DEG_180, dir);
        let mut xy = translate(1, dir, start);
        let mut jump_dist = 1;

        while self.is_open(xy) {
            self.set_jump_dist(opp_dir, xy, jump_dist);
            if self.is_axis_jump(opp_dir, xy) {
                return;
            }
            jump_dist += 1;
            xy = translate(1, dir, xy);
        }
    }

    fn clear_jump_ray(&mut self, dir: Direction, start: Point) {
        let opp_dir = rotate_c(DEG_180, dir);
        let mut xy = translate(1, dir, start);

        while self.is_open(xy) && self.get_jump_dist(opp_dir, xy) > 0 {
            self.set_jump_dist(opp_dir, xy, 0);
            if self.is_axis_jump(opp_dir, xy) {
                return;
            }
            xy = translate(1, dir, xy);
        }
    }

    fn continue_jump_ray(&mut self, dir: Direction, start: Point) {
        let opp_dir = rotate_c(DEG_180, dir);
        let mut xy = start;
        let mut jump_dist = self.get_jump_dist(opp_dir,start);

        if jump_dist > 0 {
            while self.is_open(xy) {
                self.set_jump_dist(opp_dir, xy, jump_dist);
                if self.is_axis_jump(opp_dir, xy) {
                    return;
                }
                jump_dist += 1;
                xy = translate(1, dir, xy);
            }
        }
    }

    fn print(&self, Direction(dir): Direction) {

        if self.w > 9 || self.h > 9 {
            println!("Cannot print JumpGrid. Its width and height cannot exceed 9.");
            return;
        }

        let mut y = self.h - 1;
        match dir {
            0 => println!(" ======== N ========"),
            2 => println!(" ======== E ========"),
            4 => println!(" ======== S ========"),
            6 => println!(" ======== W ========"),
            _ => println!("!BAD DIRECTION!"),
        }
        while y >= 0 {
            for x in 0..self.w {
                let xy = (x,y);
                if self.is_open(xy) {
                    if self.is_axis_jump(Direction(dir), xy) {
                        print!(" J");
                    }
                    else {
                        let jump_dist = self.get_jump_dist(Direction(dir), xy);
                        print!(" {}", jump_dist);
                    }
                }
                else {
                    print!(" X");
                }
            }
            println!("");
            y -= 1;
        }
    }
}

fn dist_between((x0,y0): Point, (x1,y1): Point) -> f32 {
    let x_dif = x0 - x1;
    let y_dif = y0 - y1;
    f32::sqrt((x_dif * x_dif + y_dif * y_dif) as f32)
}

pub fn bench() {
    let mut rng = rand::thread_rng();
    let w: isize = 512;
    let h: isize = 512;
    let mut jg = JumpGrid::new(w as usize, h as usize);

    println!("Generating map.");
    for _ in 0..((w * h) / 100) {
        let x0 = rng.gen_range(0,w);
        let y0 = rng.gen_range(0,h);
        jg.open_or_close_points(1, (x0,y0), (x0,y0));
    }

    println!("Finding path.");
    let x0 = rng.gen_range(0,w);
    let y0 = rng.gen_range(0,h);
    let x1 = rng.gen_range(0,w);
    let y1 = rng.gen_range(0,h);

    let start = PreciseTime::now();

    match jg.find_path((x0,y0), (x1,y1)) {
        Some(path) => {
            let end = PreciseTime::now();
            let mili = 1000000.0;
            let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
            println!("\nFind path time: {}ms", elapsed);
            println!("\n===== {} =====", path.len());
            println!("\n{:?}", path);
        }
        _ => {
            let end = PreciseTime::now();
            let mili = 1000000.0;
            let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
            println!("\nFind path time: {}ms", elapsed);
        }
    }
}

pub fn test() {
    let w: isize = 9;
    let h: isize = 9;
    let mut jg = JumpGrid::new(w as usize, h as usize);

    for x in 1..8 {
        jg.open_or_close_points(1, (x,1), (x,1));
        jg.print(WEST);
    }
}


#[derive(Clone,Debug)]
struct PQ<T> { vec: Vec<(f32,T)> }

impl<T> PQ<T> {
    fn new() -> PQ<T> {
        PQ{vec: Vec::new()}
    }

    fn with_capacity(size: usize) -> PQ<T> {
        PQ{vec: Vec::with_capacity(size)}
    }

    fn insert(&mut self, elem: (f32,T)) {
        let mut i = self.vec.len();

        if self.vec.is_empty() {
            self.vec.push(elem);
        }
        else {
            let (k,_) = elem;

            if k < self.vec[i - 1].0 {
                self.vec.push(elem);
                return;
            }

            while i > 0 {
                i -= 1;

                if k < self.vec[i].0 {
                    self.vec.insert(i + 1, elem);
                    return;
                }
            }
            self.vec.insert(0, elem);
        }
    }

    fn pop(&mut self) -> Option<T> {
        match self.vec.pop() {
            Some((_,v)) => Some(v),
            None => None
        }
    }
}

fn rotate_c(Degree(rot): Degree, Direction(d): Direction) -> Direction {
    let d2 = d + rot;
    if d2 >= 8 {
        return Direction(d2 - 8);
    }
    else {
        return Direction(d2);
    }
}

fn rotate_cc(Degree(rot): Degree, Direction(d): Direction) -> Direction {
    let d2 = d - rot;
    if d2 < 0 {
        return Direction(8 + d2);
    }
    else {
        return Direction(d2);
    }
}

fn translate(n: isize, Direction(dir): Direction, (x,y): Point) -> Point {
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

fn reconstruct(goal: Point, closed: &HashMap<Point,Point>, mut xy: Point) -> Vec<Point> {
    let mut vec = Vec::new();
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