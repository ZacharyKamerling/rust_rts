extern crate fnv;
extern crate rand;
extern crate time;
extern crate bit_vec;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use self::bit_vec::BitVec;
use std::cmp::Ordering;
use self::fnv::FnvHashMap;
use self::rand::Rng;
use self::time::PreciseTime;

type Point = (isize, isize);

#[derive(Clone, Debug, PartialEq)]
struct Node {
    f: f64,
    g: f64,
    xy: Point,
    direction: Ordinal,
}

#[derive(Clone, Debug)]
struct Jumps {
    nj: u16,
    ej: u16,
    sj: u16,
    wj: u16,
}

#[derive(Clone, Debug)]
pub struct PathGrid {
    w: isize,
    h: isize,
    counter: usize,
    states: BitVec,
    jumps: Vec<Jumps>,
    // Avoid allocations by using these pre-allocated collections
    open: BinaryHeap<Node>,
    closed: Vec<usize>,
    expand: Vec<(Point, Ordinal)>,
    came_from: FnvHashMap<Point, Point>,
}

impl PathGrid {

    fn reset(&mut self) {
        self.counter += 1;
        self.open.clear();
        self.expand.clear();
        self.came_from.clear();
    }

    pub fn new(w: usize, h: usize) -> PathGrid {
        let wth = w * h;
        let wph = w + h;
        let mut pg = PathGrid {
            w: w as isize,
            h: h as isize,
            counter: 1,
            states: BitVec::with_capacity(wth),
            jumps: Vec::with_capacity(wth),
            open: BinaryHeap::with_capacity(wph),
            closed: Vec::with_capacity(wph),
            expand: Vec::with_capacity(4),
            came_from: HashMap::with_capacity_and_hasher(wph, Default::default()),
        };
        for _ in 0..wth {
            pg.states.push(true);
            pg.jumps.push(Jumps {
                nj: 0,
                ej: 0,
                sj: 0,
                wj: 0,
            });
            pg.closed.push(0);
        }
        pg
    }

    pub fn width_and_height(&self) -> (isize, isize) {
        (self.w, self.h)
    }

    pub fn is_open(&self, (x, y): (isize, isize)) -> bool {
        (x >= 0) && (y >= 0) && (x < self.w) && (y < self.h) && self.states[(y * self.w + x) as usize]
    }

    pub fn find_path(&mut self, start: (isize, isize), goal: (isize, isize)) -> Option<Vec<(isize, isize)>> {
        self.reset();

        if self.is_line_open(start, goal) {
            let mut vec = Vec::new();
            vec.push(goal);
            return Some(vec);
        }

        if !self.is_open(start) || !self.is_open(goal) || start == goal {
            return None;
        }

        self.init_open(start, goal);

        while let Some(current) = self.open.pop() {

            if self.is_in_closed_list(current.xy) {
                continue;
            }

            if self.lines_up(current.xy, goal) {
                let vec = reconstruct(goal, &self.came_from, current.xy);
                return Some(vec);
            }

            self.add_to_closed_list(current.xy);
            self.expand_node(current.xy, goal, current.direction);

            for &(neighbor, dir) in &self.expand {
                if !self.is_in_closed_list(neighbor) {
                    let g = current.g + dist_between(current.xy, neighbor);
                    let f = g + dist_between(goal, neighbor);
                    let node = Node {
                        f: f,
                        g: g,
                        xy: neighbor,
                        direction: dir,
                    };
                    self.open.push(node);
                    self.came_from.insert(neighbor, current.xy);
                }
            }
        }

        None
    }

    fn set_state(&mut self, (x, y): (isize, isize), state: bool) {
        if !self.inside_bounds((x, y)) {
            if state {
                panic!("Tried to open a node outside of bounds: {:?}", (x, y));
            }
            return;
        }

        self.states.set((y * self.w + x) as usize, state);
    }

    pub fn open_area(&mut self, (x, y, w, h): (isize, isize, isize, isize)) {
        for ux in x .. x + w {
            for uy in y .. y + h {
                let xy = (ux, uy);
                self.set_state(xy, true);
            }
        }

        for ux in -1..=w {
            for uy in -1..=h {
                let ix = x + (ux as isize) - 1;
                let iy = y + (uy as isize) - 1;
                let xy = (ix, iy);

                if self.inside_bounds(xy) {
                    self.continue_jumps(Ordinal(0), xy);
                    self.continue_jumps(Ordinal(2), xy);
                    self.continue_jumps(Ordinal(4), xy);
                    self.continue_jumps(Ordinal(6), xy);
                    self.set_jumps(Ordinal(0), xy);
                    self.set_jumps(Ordinal(2), xy);
                    self.set_jumps(Ordinal(4), xy);
                    self.set_jumps(Ordinal(6), xy);
                }
            }
        }
    }

    pub fn open_point(&mut self, xy: (isize, isize)) {
        if !self.inside_bounds(xy) {
            return;
        }

        self.set_state(xy, true);

        for ux in -1..=1 {
            for uy in -1..=1 {
                let xy2 = translate(uy, Ordinal(4), translate(ux, Ordinal(6), xy));
                self.continue_jumps(Ordinal(0), xy2);
                self.continue_jumps(Ordinal(2), xy2);
                self.continue_jumps(Ordinal(4), xy2);
                self.continue_jumps(Ordinal(6), xy2);
                self.set_jumps(Ordinal(0), xy2);
                self.set_jumps(Ordinal(2), xy2);
                self.set_jumps(Ordinal(4), xy2);
                self.set_jumps(Ordinal(6), xy2);
            }
        }
    }

    pub fn close_area(&mut self, (x, y, w, h): (isize, isize, isize, isize)) {
        for ux in x .. x + w {
            for uy in y .. y + h {
                let xy = (ux, uy);
                self.set_state(xy, false);
            }
        }

        for ux in x - 1 ..= x + w {
            let uy = y - 1;
            let s = (ux, uy);

            self.set_jumps(Ordinal(2), s);
            self.set_jumps(Ordinal(6), s);
        }

        for ux in x - 1 ..= x + w {
            let uy = y + h;
            let n = (ux, uy);

            self.set_jumps(Ordinal(2), n);
            self.set_jumps(Ordinal(6), n);
        }

        for uy in y - 1 ..= y + h {
            let ux = x - 1;
            let w = (ux, uy);

            self.set_jumps(Ordinal(0), w);
            self.set_jumps(Ordinal(4), w);
        }

        for uy in y - 1 ..= y + h {
            let ux = x + w;
            let e = (ux, uy);

            self.set_jumps(Ordinal(0), e);
            self.set_jumps(Ordinal(4), e);
        }

        for ux in x .. x + w {
            let uy = y;
            let s = (ux, uy);
            self.clear_jumps(Ordinal(0), s);
        }

        for ux in x .. x + w {
            let uy = y + h;
            let n = (ux, uy);
            self.clear_jumps(Ordinal(4), n);
        }

        for uy in y .. y + h {
            let ux = x;
            let w = (ux, uy);
            self.clear_jumps(Ordinal(2), w);
        }

        for uy in y .. y + h {
            let ux = x + w;
            let e = (ux, uy);
            self.clear_jumps(Ordinal(6), e);
        }
    }

    pub fn close_point(&mut self, xy: (isize, isize)) {
        let n = translate(1, Ordinal(0), xy);
        let e = translate(1, Ordinal(2), xy);
        let s = translate(1, Ordinal(4), xy);
        let w = translate(1, Ordinal(6), xy);

        self.set_state(xy, false);
        self.set_jumps(Ordinal(0), e);
        self.set_jumps(Ordinal(0), w);
        self.set_jumps(Ordinal(2), n);
        self.set_jumps(Ordinal(2), s);
        self.set_jumps(Ordinal(4), e);
        self.set_jumps(Ordinal(4), w);
        self.set_jumps(Ordinal(6), n);
        self.set_jumps(Ordinal(6), s);
        self.clear_jumps(Ordinal(0), s);
        self.clear_jumps(Ordinal(2), w);
        self.clear_jumps(Ordinal(4), n);
        self.clear_jumps(Ordinal(6), e);
    }

    pub fn is_line_open(&self, (x0, y0): (isize, isize), (x1, y1): (isize, isize)) -> bool {
        let mut x0 = x0;
        let mut y0 = y0;

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
            if !self.is_open((x0, y0)) {
                return false;
            }

            // Check end condition
            if x0 == x1 && y0 == y1 {
                return true;
            };

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

    pub fn nearest_open(&self, start: (isize, isize)) -> Option<(isize, isize)> {
        if self.is_open(start) {
            return Some(start);
        }

        let mut bh = BinaryHeap::with_capacity(25);
        let ne = translate(1, Ordinal(1), start);
        let nw = translate(1, Ordinal(7), start);
        let se = translate(1, Ordinal(3), start);
        let sw = translate(1, Ordinal(5), start);
        let n = translate(1, Ordinal(0), start);
        let e = translate(1, Ordinal(2), start);
        let s = translate(1, Ordinal(4), start);
        let w = translate(1, Ordinal(6), start);

        let init_nodes = vec![
            (n, Ordinal(0)),
            (e, Ordinal(2)),
            (s, Ordinal(4)),
            (w, Ordinal(6)),
            (ne, Ordinal(1)),
            (nw, Ordinal(7)),
            (se, Ordinal(3)),
            (sw, Ordinal(5)),
        ];

        for &(xy, dir) in &init_nodes {
            if self.inside_bounds(xy) {
                let f = dist_between(xy, start);
                bh.push(Node {
                    f: f,
                    g: 0.0,
                    xy: xy,
                    direction: dir,
                });
            }
        }

        while let Some(node) = bh.pop() {
            if self.is_open(node.xy) {
                return Some(node.xy);
            }

            match node.direction {
                Ordinal(0) | Ordinal(2) | Ordinal(4) | Ordinal(6) => {
                    let mut tmp_xy = translate(1, node.direction, node.xy);
                    while self.is_closed_and_inside_bounds(tmp_xy) {
                        tmp_xy = translate(1, node.direction, tmp_xy);
                    }

                    if self.is_open(tmp_xy) {
                        let f = dist_between(tmp_xy, start);
                        bh.push(Node {
                            f: f,
                            g: 0.0,
                            xy: tmp_xy,
                            direction: node.direction,
                        });
                    }
                }
                Ordinal(1) | Ordinal(3) | Ordinal(5) | Ordinal(7) => {
                    let dir_c = rotate_c(DEG_45, node.direction);
                    let dir_cc = rotate_cc(DEG_45, node.direction);
                    let xy_c = translate(1, dir_c, node.xy);
                    let xy_cc = translate(1, dir_cc, node.xy);
                    let xy_next = translate(1, node.direction, node.xy);

                    if self.inside_bounds(xy_next) {
                        let f = dist_between(xy_next, start);
                        bh.push(Node {
                            f: f,
                            g: 0.0,
                            xy: xy_next,
                            direction: node.direction,
                        });
                    }

                    if self.inside_bounds(xy_c) {
                        let f = dist_between(xy_c, start);
                        bh.push(Node {
                            f: f,
                            g: 0.0,
                            xy: xy_c,
                            direction: dir_c,
                        });
                    }

                    if self.inside_bounds(xy_cc) {
                        let f = dist_between(xy_cc, start);
                        bh.push(Node {
                            f: f,
                            g: 0.0,
                            xy: xy_cc,
                            direction: dir_cc,
                        });
                    }
                }
                _ => panic!("nearest_open was given a bad Ordinal.")
            }
        }

        None
    }

    fn is_in_closed_list(&self, (x,y): (isize, isize)) -> bool {
        self.closed[(y * self.w + x) as usize] == self.counter
    }

    fn add_to_closed_list(&mut self, (x,y): (isize,isize)) {
        self.closed[(y * self.w + x) as usize] = self.counter;
    }

    fn inside_bounds(&self, (x, y): (isize, isize)) -> bool {
        (x >= 0) & (y >= 0) & (x < self.w) & (y < self.h)
    }

    fn is_closed_and_inside_bounds(&self, (x, y): (isize, isize)) -> bool {
        (x >= 0) & (y >= 0) & (x < self.w) & (y < self.h) && !self.states[(y * self.w + x) as usize]
    }

    fn init_open(&mut self, xy: Point, goal: Point) {
        let ne = translate(1, Ordinal(1), xy);
        let nw = translate(1, Ordinal(7), xy);
        let se = translate(1, Ordinal(3), xy);
        let sw = translate(1, Ordinal(5), xy);

        self.init_diag(ne, goal, Ordinal(1));
        self.init_diag(se, goal, Ordinal(3));
        self.init_diag(sw, goal, Ordinal(5));
        self.init_diag(nw, goal, Ordinal(7));

        self.init_axis(xy, goal, Ordinal(0));
        self.init_axis(xy, goal, Ordinal(2));
        self.init_axis(xy, goal, Ordinal(4));
        self.init_axis(xy, goal, Ordinal(6));
    }

    fn init_axis(&mut self, xy: Point, goal: Point, dir: Ordinal) {
        if let Some(jump) = self.get_jump(dir, xy) {
            let node = Node {
                f: dist_between(xy, jump) + dist_between(jump, goal),
                g: dist_between(xy, jump),
                xy: jump,
                direction: dir,
            };
            self.open.push(node);
        }
    }


    fn init_diag(&mut self, xy: Point, goal: Point, ne: Ordinal) {
        let w = rotate_cc(DEG_135, ne);
        let s = rotate_c(DEG_135, ne);
        let w_xy = translate(1, w, xy);
        let s_xy = translate(1, s, xy);

        if self.is_open(w_xy) || self.is_open(s_xy) {
            if let Some((jump, _)) = self.search_diag(xy, goal, ne) {
                let node = Node {
                    f: dist_between(xy, jump) + dist_between(jump, goal),
                    g: dist_between(xy, jump),
                    xy: jump,
                    direction: ne,
                };
                self.open.push(node);
            }
        }
    }

    fn expand_node(&mut self, xy: Point, goal: Point, dir: Ordinal) {
        self.expand.clear();
        match dir {
            Ordinal(0) | Ordinal(2) | Ordinal(4) | Ordinal(6) => {
                Self::expand_axis(self, xy, dir);
            }
            Ordinal(1) | Ordinal(3) | Ordinal(5) | Ordinal(7) => {
                Self::expand_diag(self, xy, goal, dir);
            }
            _ => {
                panic!("expand_node was given a bad Ordinal.")
            }
        }
    }

    fn expand_axis(&mut self, xy: Point, n: Ordinal) {
        let e = rotate_c(DEG_90, n);
        let w = rotate_cc(DEG_90, n);
        let nw = rotate_cc(DEG_45, n);
        let ne = rotate_c(DEG_45, n);
        let n_xy = translate(1, n, xy);
        let e_xy = translate(1, e, xy);
        let w_xy = translate(1, w, xy);
        let ne_xy = translate(1, ne, xy);
        let nw_xy = translate(1, nw, xy);

        if self.is_open(n_xy) {

            if let Some(n_jump) = self.get_jump(n, xy) {
                self.expand.push((n_jump, n));
            }

            if self.is_open(nw_xy) && !self.is_open(w_xy) {
                self.expand.push((nw_xy, nw));
            }

            if self.is_open(ne_xy) && !self.is_open(e_xy) {
                self.expand.push((ne_xy, ne));
            }
        }
    }

    fn expand_diag(&mut self, xy: Point, goal: Point, ne: Ordinal) {
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
        } else if !s_open && e_open && se_open {
            let opt = self.search_diag(se_xy, goal, se);
            self.push_option(opt);
        }

        if let Some(n_jump) = self.get_jump(n, xy) {
            self.expand.push((n_jump, n));
        }

        if let Some(e_jump) = self.get_jump(e, xy) {
            self.expand.push((e_jump, e));
        }

        let opt = self.search_diag(ne_xy, goal, ne);
        self.push_option(opt);
    }

    fn search_diag(&self, mut xy: Point, goal: Point, ne: Ordinal) -> Option<(Point, Ordinal)> {
        let n = rotate_cc(DEG_45, ne);
        let e = rotate_c(DEG_45, ne);
        loop {
            let n_xy = translate(1, n, xy);
            let e_xy = translate(1, e, xy);

            if !self.is_open(xy) || (!self.is_open(n_xy) && !self.is_open(e_xy)) {
                return None;
            }

            if self.get_jump(n, xy).is_some() || self.get_jump(e, xy).is_some() || self.is_diag_jump(ne, xy) || self.lines_up(xy, goal) {
                return Some((xy, ne));
            }

            xy = translate(1, ne, xy);
        }
    }

    fn lines_up(&self, start: Point, goal: Point) -> bool {
        let (sx, sy) = start;
        let (gx, gy) = goal;

        if sx != gx && sy != gy {
            return false;
        }

        if sx == gx && sy == gy {
            return true;
        }

        let dir = if gy > sy {
            Ordinal(0)
        } else if gy < sy {
            Ordinal(4)
        } else if gx > sx {
            Ordinal(2)
        } else {
            Ordinal(6)
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

    fn push_option(&mut self, opt: Option<(Point, Ordinal)>) {
        if let Some(a) = opt {
            self.expand.push(a);
        }
    }

    fn is_axis_jump(&self, dir: Ordinal, xy: Point) -> bool {
        let w  = translate(1, rotate_cc(DEG_90, dir), xy);
        let nw = translate(1, rotate_cc(DEG_45, dir), xy);
        let n  = translate(1, dir, xy);
        let s  = translate(1, rotate_c(DEG_180, dir), xy);
        let ne = translate(1, rotate_c(DEG_45, dir), xy);
        let e  = translate(1, rotate_c(DEG_90, dir), xy);

        self.is_open(xy) && self.is_open(n) && self.is_open(s) &&
            (!self.is_open(w) && self.is_open(nw) || !self.is_open(e) && self.is_open(ne))
    }

    fn is_diag_jump(&self, dir: Ordinal, xy: Point) -> bool {
        let n = translate(1, rotate_cc(DEG_45, dir), xy);
        let s = translate(1, rotate_c(DEG_135, dir), xy);
        let e = translate(1, rotate_c(DEG_45, dir), xy);
        let w = translate(1, rotate_cc(DEG_135, dir), xy);
        let nw = translate(1, rotate_cc(DEG_90, dir), xy);
        let se = translate(1, rotate_c(DEG_90, dir), xy);

        match (self.is_open(w), self.is_open(s)) {
            (false, true) => self.is_open(n) && self.is_open(nw),
            (true, false) => self.is_open(e) && self.is_open(se),
            _ => false,
        }
    }

    fn get_jump(&self, dir: Ordinal, (x, y): Point) -> Option<Point> {
        let ix = (y * self.w + x) as usize;
        match dir {
            Ordinal(0) => {
                let jmp_offset = self.jumps[ix].nj;
                if jmp_offset > 0 {
                    Some((x, y + jmp_offset as isize))
                } else {
                    None
                }
            }
            Ordinal(2) => {
                let jmp_offset = self.jumps[ix].ej;
                if jmp_offset > 0 {
                    Some((x + jmp_offset as isize, y))
                } else {
                    None
                }
            }
            Ordinal(4) => {
                let jmp_offset = self.jumps[ix].sj;
                if jmp_offset > 0 {
                    Some((x, y - jmp_offset as isize))
                } else {
                    None
                }
            }
            Ordinal(6) => {
                let jmp_offset = self.jumps[ix].wj;
                if jmp_offset > 0 {
                    Some((x - jmp_offset as isize, y))
                } else {
                    None
                }
            }
            _ => panic!("get_jump was given a diag Ordinal."),
        }
    }

    fn print_dir(&self, dir: Ordinal) {
        let mut y = self.h - 1;
        match dir {
            Ordinal(0) => println!(" ======== N ========"),
            Ordinal(2) => println!(" ======== E ========"),
            Ordinal(4) => println!(" ======== S ========"),
            Ordinal(6) => println!(" ======== W ========"),
            _ => println!("!BAD DIRECTION!"),
        }
        while y >= 0 {
            for x in 0..self.w {
                let xy = (x, y);
                if self.is_open(xy) {
                    if self.is_axis_jump(dir, xy) {
                        print!(" O");
                    } else {
                        let jump_dist = self.get_jump_dist(dir, xy);
                        if jump_dist > 0 {
                            print!(" *");
                        }
                        else {
                            print!("  ");
                        }
                    }
                } else {
                    print!(" #");
                }
            }
            println!("");
            y -= 1;
        }
    }

    fn correct_close_jumps(&mut self, n: Ordinal, start: Point) {
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

    /* Scans north looking for a north jump then goes south
    ** setting jump distances until it encounters another north jump.
    ** If no north jump is encountered, then the jump distances going south
    ** are set to 0 until it encounters another north jump.
    */
    fn continue_jumps(&mut self, n: Ordinal, start: Point) {
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

    fn set_jumps(&mut self, dir: Ordinal, mut xy: Point) {
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

    fn clear_jumps(&mut self, dir: Ordinal, start: Point) {
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

    fn set_jump_dist(&mut self, dir: Ordinal, (x, y): Point, dist: u16) {
        let ix = (y * self.w + x) as usize;
        match dir {
            Ordinal(0) => self.jumps[ix].nj = dist,
            Ordinal(2) => self.jumps[ix].ej = dist,
            Ordinal(4) => self.jumps[ix].sj = dist,
            Ordinal(6) => self.jumps[ix].wj = dist,
            _ => panic!("set_jump_dist was given a diag Ordinal."),
        }
    }

    fn get_jump_dist(&self, dir: Ordinal, (x, y): Point) -> u16 {
        let ix = (y * self.w + x) as usize;
        match dir {
            Ordinal(0) => self.jumps[ix].nj,
            Ordinal(2) => self.jumps[ix].ej,
            Ordinal(4) => self.jumps[ix].sj,
            Ordinal(6) => self.jumps[ix].wj,
            _ => panic!("set_jump_dist was given a diag Ordinal."),
        }
    }
}

fn dist_between((x0, y0): Point, (x1, y1): Point) -> f64 {
    let x_dif = x0 - x1;
    let y_dif = y0 - y1;
    f64::sqrt((x_dif * x_dif + y_dif * y_dif) as f64)
}

pub fn bench() {
    let mili = 1000000.0;
    let mut rng = rand::thread_rng();
    let w: isize = 1024;
    let h: isize = 1024;
    let mut jg = PathGrid::new(w as usize, h as usize);

    println!("Generating map.");

    let start = PreciseTime::now();

    for _ in 0..((w * h) / ((w + h) * 10)) {
        let x = rng.gen_range(0, w);
        let y = rng.gen_range(0, h);

        jg.close_area((x,y,10,10));
    }

    let end = PreciseTime::now();
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nClose 100 points: {}ms", elapsed);

    let start = PreciseTime::now();

    for _ in 0..((w * h) / ((w + h))) {
        let x = rng.gen_range(0, w);
        let y = rng.gen_range(0, h);

        jg.close_point((x, y));
    }

    let end = PreciseTime::now();
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nClose 1 point: {}ms", elapsed);

    let mut total_len = 0;
    let start = PreciseTime::now();

    for _ in 0..1000 {
        let x0 = rng.gen_range(0, w / 2);
        let y0 = rng.gen_range(0, h / 2);
        let x1 = rng.gen_range(w / 2, w);
        let y1 = rng.gen_range(h / 2, h);

        if let Some(vec) = jg.find_path((x0, y0), (x1, y1)) {
            total_len += vec.len();
        }
    }

    let end = PreciseTime::now();
    let elapsed = start.to(end).num_nanoseconds().unwrap() as f32 / mili;
    println!("\nFind path time: {}ms", elapsed);
    println!("Avg Path Len: {}", total_len / 1000);
}

pub fn test() {
    let w: isize = 40;
    let h: isize = 40;
    let mut rng = rand::thread_rng();
    let mut jg = PathGrid::new(w as usize, h as usize);

    for _ in 0..100 {
        for _ in 0..1 {
            let x = rng.gen_range(0, w);
            let y = rng.gen_range(0, h);
            jg.close_area((x, y, 2, 2));
        }

        for _ in 0..9 {
            let x = rng.gen_range(0, w);
            let y = rng.gen_range(0, h);
            jg.open_area((x, y, 1, 1));
        }
    }

    jg.print_dir(Ordinal(0));
    jg.print_dir(Ordinal(4));
    jg.print_dir(Ordinal(2));
    jg.print_dir(Ordinal(6));
}

fn reconstruct(goal: Point, closed: &FnvHashMap<Point, Point>, mut xy: Point) -> Vec<Point> {
    let mut vec = Vec::with_capacity(512);
    vec.push(goal);

    loop {
        vec.push(xy);
        match closed.get(&xy) {
            Some(next) => {
                xy = *next;
            }
            None => break,
        }
    }
    vec
}

impl Eq for Node {}

impl Ord for Node {
    #[inline]
    fn cmp(&self, other: &Node) -> Ordering {
        // Notice that the we flip the ordering here
        // We need a min heap but Rust only has a max heap
        if other.f > self.f {
            Ordering::Greater
        }
        else {
            Ordering::Less
        }
    }
}

impl PartialOrd for Node {
    #[inline]
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug)]
struct Degree(i8);

const DEG_45: Degree = Degree(1);
const DEG_90: Degree = Degree(2);
const DEG_135: Degree = Degree(3);
const DEG_180: Degree = Degree(4);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Ordinal(i8);

#[inline]
fn rotate_c(Degree(rot): Degree, Ordinal(d): Ordinal) -> Ordinal {
    let d2 = d + rot;
    if d2 >= 8 {
        Ordinal(d2 - 8)
    } else {
        Ordinal(d2)
    }
}

#[inline]
fn rotate_cc(Degree(rot): Degree, Ordinal(d): Ordinal) -> Ordinal {
    if rot > d {
        Ordinal(8 + d - rot)
    } else {
        Ordinal(d - rot)
    }
}

#[inline]
fn translate(n: isize, ord: Ordinal, (x, y): (isize, isize)) -> (isize, isize) {
    match ord {
        Ordinal(0) => (x    , y + n),
        Ordinal(1) => (x + n, y + n),
        Ordinal(2) => (x + n, y    ),
        Ordinal(3) => (x + n, y - n),
        Ordinal(4) => (x    , y - n),
        Ordinal(5) => (x - n, y - n),
        Ordinal(6) => (x - n, y    ),
        Ordinal(7) => (x - n, y + n),
        Ordinal(_) => panic!("translate: Ordinal is out of range [0-8)."),
    }
}