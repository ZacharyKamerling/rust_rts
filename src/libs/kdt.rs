#![allow(needless_range_loop)]

extern crate rand;
extern crate time;

use std::ops::Rem;
use self::rand::Rng;
use self::time::PreciseTime;

pub struct KDTree<T>
where
    T: Dimensions,
{
    trees: [Tree; 1024],
    vec: Vec<T>,
}

#[derive(Clone, Copy)]
enum Tree {
    Fork(f64 // Dividing line
        , usize // Index to left tree
        , usize // Index to mid tree
        , usize // Index to right tree
        ),
    Leaf(usize, usize), // Start & end indices
}

pub trait Dimensions {
    fn bucket_size() -> usize;
    fn num_dims() -> usize;
    fn dimensions(&self, dim: usize) -> f64;
    fn radii(&self, dim: usize) -> f64;
}

impl<T: Clone + Dimensions> KDTree<T> {
    pub fn new(vec: Vec<T>) -> KDTree<T> {
        let len = vec.len();
        let depth = (len as f64 / <T as Dimensions>::bucket_size() as f64)
            .ceil()
            .log(2.0) as usize;
        let mut kdt = KDTree {
            trees: [Tree::Leaf(0, 0); 1024],
            vec: vec,
        };
        let (_, tree) = kdt.make_tree(depth, 0, 0, len, 0);
        kdt.trees[0] = tree;
        kdt
    }

    pub fn in_range(&self, pred: &Fn(&T) -> bool, dims: &[(f64, f64)]) -> Vec<T> {
        let mut vec = Vec::with_capacity(128);
        KDTree::in_range_matching(self, self.trees[0], pred, dims, 0, &mut vec);
        vec
    }

    pub fn in_range_buff(&self, pred: &Fn(&T) -> bool, dims: &[(f64, f64)], vec: &mut Vec<T>) {
        vec.clear();
        KDTree::in_range_matching(self, self.trees[0], pred, dims, 0, vec);
    }

    fn make_tree(&mut self, depth: usize, dim: usize, ix: usize, len: usize, tree_ix: usize) -> (usize, Tree) {
        if len <= <T as Dimensions>::bucket_size() || depth == 0 {
            (1, Tree::Leaf(ix, len))
        } else {
            let next_dim = (dim + 1).rem(<T as Dimensions>::num_dims());
            let next_depth = depth - 1;

            let avg = KDTree::mean(dim, &self.vec, ix, len);
            let left_count = KDTree::left_divide(dim, avg, &mut self.vec, ix, len);
            let mid_count = KDTree::mid_divide(dim, avg, &mut self.vec, ix + left_count, len - left_count);
            let right_count = len - left_count - mid_count;

            let left_ix = tree_ix + 3;
            let (left_num, left_tree) = self.make_tree(next_depth, next_dim, ix, left_count, left_ix);

            let mid_ix = left_ix + left_num;
            let (mid_num, mid_tree) = self.make_tree(next_depth, next_dim, ix + left_count, mid_count, mid_ix);

            let right_ix = mid_ix + mid_num;
            let (right_num, right_tree) = self.make_tree(
                next_depth,
                next_dim,
                ix + left_count + mid_count,
                right_count,
                right_ix,
            );

            let total_trees = left_num + mid_num + right_num + 3;

            self.trees[tree_ix + 1] = left_tree;
            self.trees[tree_ix + 2] = mid_tree;
            self.trees[tree_ix + 3] = right_tree;

            (
                total_trees,
                Tree::Fork(avg, tree_ix + 1, tree_ix + 2, tree_ix + 3),
            )
        }
    }

    fn mean(dim: usize, vec: &[T], ix: usize, len: usize) -> f64 {
        let mut acc = 0.0;
        for i in ix..ix + len {
            acc += vec[i].dimensions(dim);
        }
        acc / (len as f64)
    }

    fn left_divide(dim: usize, avg: f64, vec: &mut [T], ix: usize, len: usize) -> usize {
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
    fn mid_divide(dim: usize, avg: f64, vec: &mut [T], ix: usize, len: usize) -> usize {
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

    fn in_range_matching(&self, tree: Tree, pred: &Fn(&T) -> bool, dims: &[(f64, f64)], dim: usize, vec: &mut Vec<T>) -> () {
        let next_dim = (dim + 1).rem(<T as Dimensions>::num_dims());
        let (crd, rad) = dims[dim];

        match tree {
            Tree::Fork(div, l, m, r) => {
                if crd - rad <= div {
                    self.in_range_matching(self.trees[l], pred, dims, next_dim, vec);
                }
                self.in_range_matching(self.trees[m], pred, dims, next_dim, vec);
                if crd + rad >= div {
                    self.in_range_matching(self.trees[r], pred, dims, next_dim, vec);
                }
            }
            Tree::Leaf(ix, len) => {
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
    pub id: usize,
    pub team: usize,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub weight: f64,
    pub flying: bool,
    pub structure: bool,
    pub missile: bool,
    pub ground: bool,
}

impl Dimensions for PointAndRadii {
    fn bucket_size() -> usize {
        512
    }
    fn num_dims() -> usize {
        2
    }
    fn dimensions(&self, dim: usize) -> f64 {
        match dim {
            0 => self.x,
            _ => self.y,
        }
    }
    fn radii(&self, _: usize) -> f64 {
        self.radius
    }
}

pub fn bench() {
    let num_units = 4000;
    let search_radius = 8.0;
    let mili = 1000000.0;

    let mut rng = rand::thread_rng();

    let start1 = PreciseTime::now();
    let mut vec = Vec::with_capacity(num_units);
    for n in 0..num_units {
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

    for a in &vec {
        let start2 = PreciseTime::now();
        // Prediacate for filtering out all entities not actually in range.
        let pred = |b: &PointAndRadii| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dr = b.radius + search_radius;
            (dx * dx) + (dy * dy) <= dr * dr
        };
        let p = &pred as &Fn(&PointAndRadii) -> bool;
        let in_rng = kdt.in_range(p, &[(a.x, search_radius), (a.y, search_radius)]);
        let end2 = PreciseTime::now();
        total_kdt_search_time += start2.to(end2).num_nanoseconds().unwrap();
        total_in_rng1 += in_rng.len();
    }

    for a in &vec {
        let mut temp_vec = Vec::new();
        let start2 = PreciseTime::now();
        let pred = |b: &PointAndRadii| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dr = b.radius + search_radius;
            (dx * dx) + (dy * dy) <= dr * dr
        };
        let p = &pred as &Fn(&PointAndRadii) -> bool;

        for e in &vec {
            if p(e) {
                temp_vec.push((*e).clone())
            }
        }
        let end2 = PreciseTime::now();
        total_search_time += start2.to(end2).num_nanoseconds().unwrap();
        total_in_rng2 += temp_vec.len();
    }

    let build_time = start1.to(end1).num_nanoseconds().unwrap();
    println!("Build time: {}ms", build_time as f64 / mili);
    println!("KDT search time: {}ms", total_kdt_search_time as f64 / mili);
    println!("Naive search time: {}ms", total_search_time as f64 / mili);
    println!(
        "Improvement: {}",
        total_search_time as f64 / (total_kdt_search_time + build_time) as f64
    );
    println!("KDTree in range: {}", total_in_rng1);
    println!("Naives in range: {} \n", total_in_rng2);
}
