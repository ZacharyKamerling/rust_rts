#![allow(dead_code)]
extern crate rand;
extern crate time;

use std::collections::HashMap;
use self::time::{PreciseTime};
use self::rand::Rng;
use std::cmp::{min,max};
use std::f64;
use std::collections::VecDeque;

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Direction(isize);

#[derive(Clone,Copy,Debug)]
struct Degree(isize);

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Point(isize,isize);

#[derive(Clone)]
pub struct Node(f64,Point,Direction);

const DEG_45: Degree = Degree(1);
const DEG_90: Degree = Degree(2);
const DEG_135: Degree = Degree(3);
const DEG_180: Degree = Degree(4);
const NORTH: Direction = Direction(0);
const SOUTH: Direction = Direction(4);
const EAST: Direction = Direction(2);
const WEST: Direction = Direction(6);
const NORTHEAST: Direction = Direction(1);
const SOUTHEAST: Direction = Direction(3);
const SOUTHWEST: Direction = Direction(5);
const NORTHWEST: Direction = Direction(7);

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

fn translate(n: isize, Direction(dir): Direction, Point(x,y): Point) -> Point {
    match dir {
        0 => return Point(x, y + n),
        1 => return Point(x + n, y + n),
        2 => return Point(x + n, y),
        3 => return Point(x + n, y - n),
        4 => return Point(x, y - n),
        5 => return Point(x - n, y - n),
        6 => return Point(x - n, y),
        7 => return Point(x - n, y + n),
        _ => panic!("translate was given a bad Direction.")
    }
}

struct Jumps {
    nj: u16,
    ej: u16,
    sj: u16,
    wj: u16,
}

pub struct JumpGrid {
    w: isize,
    h: isize,
    open_vec: Vec<u8>,
    jump_vec: Vec<Jumps>,
}

impl JumpGrid
{
    pub fn make(w: usize, h: usize) -> JumpGrid {
        let mut jg = JumpGrid
                { w: w as isize
                , h: h as isize
                , open_vec: Vec::with_capacity(w * h)
                , jump_vec: Vec::with_capacity(w * h)
                };
        for _ in 0..(w * h) {
            jg.open_vec.push(0);
            jg.jump_vec.push(Jumps {nj: 0, ej: 0, sj: 0, wj: 0});
        }
        jg
    }

    fn is_open(&self, Point(x,y): Point) -> bool {
        x >= 0 &&
        y >= 0 &&
        x < self.w &&
        y < self.h &&
        self.open_vec[(y * self.w + x) as usize] == 0
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

    fn get_jump(&self, Direction(dir): Direction, Point(x,y): Point) -> Option<Point> {
        match dir {
            0 =>
                {
                    let jmp_offset = self.jump_vec[(y * self.w + x) as usize].nj;
                    if jmp_offset > 0 {
                        Some(Point(x, y + jmp_offset as isize))
                    }
                    else {
                        None
                    }
                }
            2 =>
                {
                    let jmp_offset = self.jump_vec[(y * self.w + x) as usize].ej;
                    if jmp_offset > 0 {
                        Some(Point(x + jmp_offset as isize, y))
                    }
                    else {
                        None
                    }
                }
            4 =>
                {
                    let jmp_offset = self.jump_vec[(y * self.w + x) as usize].sj;
                    if jmp_offset > 0 {
                        Some(Point(x, y - jmp_offset as isize))
                    }
                    else {
                        None
                    }
                }
            6 =>
                {
                    let jmp_offset = self.jump_vec[(y * self.w + x) as usize].wj;
                    if jmp_offset > 0 {
                        Some(Point(x - jmp_offset as isize, y))
                    }
                    else {
                        None
                    }
                }
            _ => panic!("get_jump was given a diag Direction.")
        }
    }

    pub fn open_or_close_point(&mut self, open_or_close: u8, Point(x0,y0): Point, Point(x1,y1): Point) {
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

        for y in max(0, min_y - 1)..min(self.h, max_y + 3) {
            for x in max(0, min_x - 1)..min(self.w, max_x + 3) {
                let xy = Point(x,y);
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
        for y in max(0, min_y_bound)..min(self.h, max_y + 2) {
            for x in max(0, min_x_bound)..min(self.w, max_x + 2) {
                let xy = Point(x,y);

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
                self.clear_jump_ray(SOUTH, Point(x, min_y));
                self.clear_jump_ray(NORTH, Point(x, max_y));
            }

            for y in min_y..max_y_bound {
                self.clear_jump_ray(WEST, Point(min_x, y));
                self.clear_jump_ray(EAST, Point(max_x, y));
            }
        }
        else {

            if max_x_bound < self.w {
                for y in min_y..max_y_bound {
                    self.continue_jump_ray(WEST, Point(max_x_bound, y));
                }
            }

            if min_x_bound >= 0 {
                for y in min_y..max_y_bound {
                    self.continue_jump_ray(EAST, Point(min_x_bound, y));
                }
            }

            if max_y_bound < self.h {
                for x in min_x..max_x_bound {
                    self.continue_jump_ray(SOUTH, Point(x, max_y_bound));
                }
            }

            if min_y_bound >= 0 {
                for x in min_x..max_x_bound {
                    self.continue_jump_ray(NORTH, Point(x, min_y_bound));
                }
            }
        }
    }

    fn set_jump_dist(&mut self, Direction(dir): Direction, Point(x,y): Point, dist: u16) {
        let ix = (y * self.w + x) as usize;
        match dir {
            0 => self.jump_vec[ix].nj = dist,
            2 => self.jump_vec[ix].ej = dist,
            4 => self.jump_vec[ix].sj = dist,
            6 => self.jump_vec[ix].wj = dist,
            _ => panic!("set_jump_dist was given a diag Direction.")
        }
    }

    fn get_jump_dist(&self, Direction(dir): Direction, Point(x,y): Point) -> u16 {
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
                let xy = Point(x,y);
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

    fn init_open(&self, xy: Point) -> PQ<Node> {
        let n  = translate(1, NORTH, xy);
        let s  = translate(1, SOUTH, xy);
        let e  = translate(1, EAST, xy);
        let w  = translate(1, WEST, xy);
        let ne = translate(1, NORTHEAST, xy);
        let nw = translate(1, NORTHWEST, xy);
        let se = translate(1, SOUTHEAST, xy);
        let sw = translate(1, SOUTHWEST, xy);
        let n_open  = self.is_open(n);
        let s_open  = self.is_open(s);
        let e_open  = self.is_open(e);
        let w_open  = self.is_open(w);
        let ne_open = self.is_open(ne);
        let nw_open = self.is_open(nw);
        let se_open = self.is_open(se);
        let sw_open = self.is_open(sw);

        let mut vec = Vec::with_capacity(8);
        vec.push((n, n_open, NORTH));
        vec.push((s, s_open, SOUTH));
        vec.push((e, e_open, EAST));
        vec.push((w, w_open, WEST));
        vec.push((ne, (n_open || e_open) && ne_open, NORTHEAST));
        vec.push((nw, (n_open || w_open) && nw_open, NORTHWEST));
        vec.push((se, (s_open || e_open) && se_open, SOUTHEAST));
        vec.push((sw, (s_open || w_open) && sw_open, SOUTHWEST));

        let mut pq = PQ::new();
        for e in vec.iter() {
            let (p,b,dir) = *e;
            if b {
                let dist = dist_between(xy,p);
                pq.push((dist, Node(dist, p, dir)));
            }
        }
        pq
    }

    fn expand_node(&self, dist: f64, xy: Point, goal: Point, dir: Direction) -> Vec<(f64, Point, Direction)> {
        let Direction(d) = dir;
        match d {
            0 | 2 | 4 | 6 => Self::expand_axis(self, dist, xy, dir),
            1 | 3 | 5 | 7 => Self::expand_diag(self, dist, xy, goal, dir),
            _             => panic!("Expansion failed with a bad Direction.")
        }
    }

    fn expand_axis(&self, dist: f64, xy: Point, n: Direction) -> Vec<(f64, Point, Direction)> {
        let mut vec = Vec::new();
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
                Some(n_jump) => vec.push((dist + dist_between(xy, n_jump), n_jump, n)),
                _            => (),
            }

            if self.is_open(nw_xy) && !self.is_open(w_xy) {
                vec.push((dist + dist_between(xy, nw_xy), nw_xy, nw));
            }

            if self.is_open(ne_xy) && !self.is_open(e_xy) {
                vec.push((dist + dist_between(xy, ne_xy), ne_xy, ne));
            }
        }
        vec
    }

    fn expand_diag(&self, dist: f64, mut xy: Point, goal: Point, ne: Direction) -> Vec<(f64, Point, Direction)> {
        let mut vec = Vec::new();

        loop {
            if self.lines_up(xy, goal) {
                vec.push((0.0, xy, ne));
                return vec;
            }
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

            if !self.is_open(w_xy) && self.is_open(n_xy) && self.is_open(nw_xy) {
                vec.push((dist + dist_between(xy, nw_xy), nw_xy, nw));
            }
            else if !self.is_open(s_xy) && self.is_open(e_xy) && self.is_open(se_xy) {
                vec.push((dist + dist_between(xy, se_xy), se_xy, se));
            }

            match (self.get_jump(n, xy), self.get_jump(e, xy)) {
                ( Some(n_jump)
                , Some(e_jump)
                ) => {
                    vec.push((dist + dist_between(xy, n_jump), n_jump, n));
                    vec.push((dist + dist_between(xy, e_jump), e_jump, e));
                    if (self.is_open(n_xy) || self.is_open(e_xy)) && self.is_open(ne_xy) {
                        vec.push((dist + dist_between(xy, ne_xy), ne_xy, ne));
                    }
                    break;
                }
                ( Some(n_jump)
                , None
                ) => {
                    vec.push((dist + dist_between(xy, n_jump), n_jump, n));
                    if (self.is_open(n_xy) || self.is_open(e_xy)) && self.is_open(ne_xy) {
                        vec.push((dist + dist_between(xy, ne_xy), ne_xy, ne));
                    }
                    break;
                }

                ( None
                , Some(e_jump)
                ) => {
                    vec.push((dist + dist_between(xy, e_jump), e_jump, e));
                    if (self.is_open(n_xy) || self.is_open(e_xy)) && self.is_open(ne_xy) {
                        vec.push((dist + dist_between(xy, ne_xy), ne_xy, ne));
                    }
                    break;
                }
                _ => {
                    if !self.is_open(n_xy) && !self.is_open(e_xy) || !self.is_open(ne_xy) {
                        break;
                    }
                }
            }
            xy = translate(1, ne, xy);
        }
        vec
    }

    fn lines_up(&self, start: Point, goal: Point) -> bool {
        let Point(sx,sy) = start;
        let Point(gx,gy) = goal;

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

    fn reconstruct(closed: HashMap<Point,Point>, mut xy: Point) -> Vec<Point> {
        let mut vec = VecDeque::new();

        loop {
            vec.push_front(xy);
            match closed.get(&xy) {
                Some(next) => {
                    xy = *next;
                }
                None => break
            }
        }
        vec.into_iter().collect()
    }

    pub fn find_path(&self, start: Point, goal: Point) -> Option<Vec<Point>> {
        if !self.is_open(start) || !self.is_open(goal) {
            None
        }
        else {
            let mut open: PQ<Node> = Self::init_open(self, start);
            let mut closed: HashMap<Point,Point> = HashMap::new();
            loop {
                match open.pop() {
                    Some(Node(dist, xy, dir)) => {
                        if self.lines_up(xy,goal) {
                            let mut vec = JumpGrid::reconstruct(closed, xy);
                            vec.push(goal);
                            return Some(vec);
                        }
                        for e in self.expand_node(dist, xy, goal, dir).iter() {
                            let (dist2, xy2, dir2) = *e;

                            if !closed.contains_key(&xy2) {
                                let node = Node(dist2 + dist_between(xy, xy2), xy2, dir2);
                                open.push((dist2 + dist_between(goal, xy2), node));
                                closed.insert(xy2,xy);
                            }
                        }
                    }
                    _ => {
                        return None;
                    }
                }
            }
        }
    }
}

fn dist_between(Point(x0,y0): Point, Point(x1,y1): Point) -> f64 {
    let x_dif = x0 - x1;
    let y_dif = y0 - y1;
    f64::sqrt((x_dif * x_dif + y_dif * y_dif) as f64)
}

pub fn test() {
    let mut rng = rand::thread_rng();
    let w: isize = 1024 * 2;
    let h: isize = 1024 * 2;
    let mut jg = JumpGrid::make(w as usize, h as usize);
    jg.open_or_close_point(1, Point(1, h / 2), Point(w - 2, h / 2));
    jg.open_or_close_point(1, Point(w / 2, 1), Point(w / 2, h - 2));

    for _ in 0..(w * h / 10) {
        let x0 = rng.gen_range(0,w);
        let y0 = rng.gen_range(0,h);
        jg.open_or_close_point(1, Point(x0,y0), Point(x0,y0));
    }

    let start = PreciseTime::now();
    for _ in 0..1000 {
        let x0 = rng.gen_range(0,w);
        let y0 = rng.gen_range(0,h);
        let x1 = rng.gen_range(0,w);
        let y1 = rng.gen_range(0,h);
        jg.find_path(Point(x0,y0), Point(x1,y1));
    }
    let end = PreciseTime::now();
    let mili = 1000000.0;
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("Find path time: {}ms", elapsed);
}

struct PQ<T> { vec: Vec<(f64,T)> }

impl<T> PQ<T> {
    fn new() -> PQ<T> {
        PQ{vec: Vec::new()}
    }

    fn push(&mut self, elem: (f64,T)) {
        let mut i = self.vec.len();
        let (k,_) = elem;

        if i == 0 {
            self.vec.push(elem);
            return;
        }

        while i > 0 {
            i -= 1;
            let (k2,_) = self.vec[i];
            if k <= k2 {
                self.vec.insert(i, elem);
                return;
            }
        }
    }

    fn pop(&mut self) -> Option<T> {
        match self.vec.pop() {
            Some((_,v)) => Some(v),
            None => None
        }
    }
}