extern crate rand;
extern crate time;

use std::ops::Rem;
use self::rand::Rng;
use self::time::{PreciseTime};

pub struct KDTree<T> where T: Dimensions {
    tree: Tree,
    vec: Vec<T>,
}

#[derive(Clone)]
enum Tree {
    Fork( f32       // Dividing line
        , Box<Tree> // Left elements
        , Box<Tree> // Middle elements
        , Box<Tree> // Right elements
        ),
    Leaf(usize,usize)
}

pub trait Dimensions {
    fn bucket_size() -> usize;
    fn num_dims() -> usize;
    fn dimensions(&self, dim: usize) -> f32;
    fn radii(&self, dim: usize) -> f32;
}

impl<T: Clone + Dimensions> KDTree<T> {

    pub fn new(mut vec: Vec<T>) -> KDTree<T> {
        let len = vec.len();
        let depth = (len as f32 / <T as Dimensions>::bucket_size() as f32).ceil().log(2.0) as usize;
        let tree = KDTree::make_tree(depth, 0, &mut vec, 0, len);
        KDTree{tree: tree, vec: vec}
    }

    pub fn in_range(&self, pred: &Fn(&T) -> bool, dims: &[(f32,f32)]) -> Vec<T> {
        let mut vec = Vec::new();
        KDTree::in_range_matching(self, self.tree.clone(), pred, dims, 0, &mut vec);
        vec
    }

    pub fn in_range_buff(&self, pred: &Fn(&T) -> bool, dims: &[(f32,f32)], vec: &mut Vec<T>) {
        vec.clear();
        KDTree::in_range_matching(self, self.tree.clone(), pred, dims, 0, vec);
    }

    fn make_tree(depth: usize, dim: usize, vec: &mut Vec<T>, ix: usize, len: usize) -> Tree {
        if len <= <T as Dimensions>::bucket_size() || depth == 0 {
            Tree::Leaf(ix,len)
        }
        else {
            let next_dim = (dim + 1).rem(<T as Dimensions>::num_dims());

            let avg = KDTree::mean(dim, vec, ix, len);
            let left_count = KDTree::left_divide(dim, avg, vec, ix, len);
            let mid_count = KDTree::mid_divide(dim, avg, vec, ix + left_count, len - left_count);
            let right_count = len - left_count - mid_count;

            let left_tree = KDTree::make_tree(depth - 1, next_dim, vec, ix, left_count);
            let mid_tree = KDTree::make_tree(depth - 1, next_dim, vec, ix + left_count, mid_count);
            let right_tree = KDTree::make_tree(depth - 1, next_dim, vec, ix + left_count + mid_count, right_count);

            Tree::Fork(avg, Box::new(left_tree), Box::new(mid_tree), Box::new(right_tree))
        }
    }

    fn mean(dim: usize, vec: &Vec<T>, ix: usize, len: usize) -> f32 {
        let mut acc = 0.0;
        for i in ix..ix + len {
            acc += vec[i].dimensions(dim);
        }
        acc / (len as f32)
    }

    fn left_divide(dim: usize, avg: f32, vec: &mut Vec<T>, ix: usize, len: usize) -> usize {
        let mut c = ix;
        for i in ix..ix + len {
            let e = vec[i].clone();
            if e.dimensions(dim) + e.radii(dim) < avg {
                vec[i] = vec[c].clone();
                vec[c] = e;
                c += 1;
            }
        }
        c - ix
    }

    // Move all elements who cross the median/avg line to the left side of the slice
    fn mid_divide(dim: usize, avg: f32, vec: &mut Vec<T>, ix: usize, len: usize) -> usize {
        let mut c = ix;
        for i in ix..ix + len {
            let e = vec[i].clone();
            if e.dimensions(dim) - e.radii(dim) <= avg {
                vec[i] = vec[c].clone();
                vec[c] = e;
                c += 1;
            }
        }
        c - ix
    }

    fn in_range_matching(&self, tree: Tree, pred: &Fn(&T) -> bool, dims: &[(f32,f32)], dim: usize, vec: &mut Vec<T>) -> () {
        let next_dim = (dim + 1).rem(<T as Dimensions>::num_dims());
        let (crd,rad) = dims[dim];

        match tree {
            Tree::Fork(div, l, m, r) =>
                {
                    if crd - rad <= div {
                        self.in_range_matching(*l, pred, dims, next_dim, vec);
                    }
                    self.in_range_matching(*m, pred, dims, next_dim, vec);
                    if crd + rad >= div {
                        self.in_range_matching(*r, pred, dims, next_dim, vec);
                    }
                }
            Tree::Leaf(ix,len) =>
                {
                    for i in ix..ix + len {
                        if pred(&self.vec[i]) {
                            vec.push(self.vec[i].clone());
                        }
                    }
                }
        }
    }
}

#[derive(Clone)]
pub struct PointAndRadii {
    pub id:         usize,
    pub team:       usize,
    pub x:          f32,
    pub y:          f32,
    pub radius:     f32,
    pub weight:     f32,
    pub flying:     bool,
    pub structure:  bool,
    pub missile:    bool,
    pub ground:     bool,
}

impl Dimensions for PointAndRadii {
    fn bucket_size() -> usize {256}
    fn num_dims() -> usize {2}
    fn dimensions(&self, dim: usize) -> f32 {
        match dim {
            0 => { self.x }
            _ => { self.y }
        }
    }
    fn radii(&self, _: usize) -> f32 {
        self.radius
    }
}

pub fn bench() {
    let num_units = 4096;
    let search_radius = 16.0;
    let mili = 1000000.0;

    let mut rng = rand::thread_rng();

    let start1 = PreciseTime::now();
    let mut vec = Vec::with_capacity(num_units);
    for n in (0..num_units) {
        vec.push(PointAndRadii {
            id: n,
            team: rng.gen_range(0, 8),
            x: rng.gen_range(0.0, 1024.0),
            y: rng.gen_range(0.0, 1024.0),
            radius: 0.0,
            weight: 1.0,
            flying: rng.gen(),
            structure: rng.gen(),
            missile: rng.gen(),
            ground: rng.gen(),
        });
    }
    let kdt = KDTree::new(vec.clone());
    let end1 = PreciseTime::now();

    let mut total_kdt_search_time = 0;
    let mut total_search_time = 0;
    let mut total_in_rng1 = 0;
    let mut total_in_rng2 = 0;

    for a in vec.iter() {
        let start2 = PreciseTime::now();
        // Prediacate for filtering out all entities not actually in range.
        let pred = |b: &PointAndRadii| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dr = b.radius + search_radius;
            (dx * dx) + (dy * dy) <= dr * dr
        };
        let p = &pred as &Fn(&PointAndRadii) -> bool;
        let in_rng = kdt.in_range(p, &[(a.x,search_radius), (a.y,search_radius)]);
        let end2 = PreciseTime::now();
        total_kdt_search_time += start2.to(end2).num_nanoseconds().unwrap();
        total_in_rng1 += in_rng.len();
    }

    for a in vec.iter() {
        let mut temp_vec = Vec::new();
        let start2 = PreciseTime::now();
        let pred = |b: &PointAndRadii| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dr = b.radius + search_radius;
            (dx * dx) + (dy * dy) <= dr * dr
        };
        let p = &pred as &Fn(&PointAndRadii) -> bool;

        for e in vec.iter() {
            if p(e) {
                temp_vec.push((*e).clone())
            }
        }
        let end2 = PreciseTime::now();
        total_search_time += start2.to(end2).num_nanoseconds().unwrap();
        total_in_rng2 += temp_vec.len();
    }

    let build_time = start1.to(end1).num_nanoseconds().unwrap();
    println!("Build time: {}ms", build_time as f32 / mili);
    println!("KDT search time: {}ms", total_kdt_search_time as f32 / mili);
    println!("Naive search time: {}ms", total_search_time as f32 / mili);
    println!("Improvement: {}", total_search_time as f32 / (total_kdt_search_time + build_time) as f32);
    println!("KDT in range: {}", total_in_rng1);
    println!("Naive in range: {} \n", total_in_rng2);
}