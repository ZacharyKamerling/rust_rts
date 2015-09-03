extern crate rand;
extern crate time;

use std::ops::Rem;
use self::rand::Rng;
use self::time::{PreciseTime};

#[derive(Clone)]
pub struct KDTree<'a, T: 'a> where T: Dimensions {
    tree: Tree<'a, T>,
}

#[derive(Clone)]
enum Tree<'a, T: 'a> where T: Dimensions {
    Fork( f32              // Dividing line
        , Box<Tree<'a, T>> // Left elements
        , Box<Tree<'a, T>> // Middle elements
        , Box<Tree<'a, T>> // Right elements
        ),
    Leaf( &'a [T] )
}

pub trait Dimensions {
    fn num_dims() -> usize;
    fn dimensions(&self, dim: usize) -> f32;
    fn radii(&self, dim: usize) -> f32;
}

impl<'a, T: Clone + Dimensions> KDTree<'a, T> {

    pub fn make_from(slice: &'a mut [T]) -> KDTree<'a, T> {
        KDTree {tree: KDTree::make_tree(32, slice, 0)}
    }

    pub fn get_in_range<'b>(&self, pred: &Fn(&T) -> bool, dims: &[(f32,f32)]) -> Vec<&T> {
        let mut vec = Vec::with_capacity(10);
        KDTree::in_range(self, self.tree.clone(), dims, 0, &mut vec);
        vec.into_iter().flat_map(|s| s.into_iter()).filter(|a| pred(a)).collect::<Vec<_>>()
    }
    
    fn in_range<'b>(&'a self, tree: Tree<'a, T>, dims: &[(f32,f32)], dim: usize, vec: &mut Vec<&'a [T]>) -> () {
        let next_dim = (dim + 1).rem(<T as Dimensions>::num_dims());

        let (crd,rad) = dims[dim];

        match tree {
            Tree::Fork(div, l, m, r) =>
                {
                    if crd - rad <= div {
                        self.in_range(*l, dims, next_dim, vec);
                    }
                    self.in_range(*m, dims, next_dim, vec);
                    if crd + rad >= div {
                        self.in_range(*r, dims, next_dim, vec);
                    }
                }
            Tree::Leaf(elems) =>
                {
                    vec.push(elems);
                }
        }
    }
    
    /* Recursively builds a tree by dividing a slice into three sections.
    The left/right section is elements on the left/right of the avg/median line who's radius
    does not cross the line. The mid section is the elements who's radius crosses
    the line.
    */
    fn make_tree(depth: usize, slice: &'a mut [T], dim: usize) -> Tree<'a, T> {
        let len = slice.len();
        if len <= 342 || depth == 0 {
            Tree::Leaf(slice)
        }
        else {
            let avg = KDTree::mean(slice, dim);

            // Number of elements on the left side
            let left_count = KDTree::left_divide(slice, dim, avg);

            // Split slice into elements on the left and the rest
            let (left, mid_right) = slice.split_at_mut(left_count);

            // Number of elements that cross the avg/median line
            let mid_count = KDTree::mid_divide(mid_right, dim, avg);

            // Split slice into elements in the middle and right
            let (mid, right) = mid_right.split_at_mut(mid_count);

            // Cycle through dimensions
            let next_dim = (dim + 1).rem(<T as Dimensions>::num_dims());

            let left_tree = KDTree::make_tree(depth - 1, left, next_dim);
            let mid_tree = KDTree::make_tree(depth - 1, mid, next_dim);
            let right_tree = KDTree::make_tree(depth - 1, right, next_dim);

            Tree::Fork(avg, Box::new(left_tree), Box::new(mid_tree), Box::new(right_tree))
        }
    }

    // Move all elements who's radius doesn't cross the middle to the left side of the slice
    // Works like quicksort
    fn left_divide(slice: &mut [T], dim: usize, avg: f32) -> usize {
        let mut c = 0;
        for i in 0..slice.len() {
            let e = slice[i].clone();
            if e.dimensions(dim) + e.radii(dim) < avg {
                slice[i] = slice[c].clone();
                slice[c] = e;
                c += 1;
            }
        }
        c
    }

    // Move all elements who cross the median/avg line to the left side of the slice
    fn mid_divide(slice: &mut [T], dim: usize, avg: f32) -> usize {
        let mut c = 0;
        for i in 0..slice.len() {
            let e = slice[i].clone();
            if e.dimensions(dim) - e.radii(dim) <= avg {
                slice[i] = slice[c].clone();
                slice[c] = e;
                c += 1;
            }
        }
        c
    }

    fn mean(slice: &mut [T], dim: usize) -> f32 {
        let mut acc = 0.0;
        for e in slice.iter() {
            acc += e.dimensions(dim);
        }
        acc / (slice.len() as f32)
    }
}

/* TEST CODE
Creates two-dimensional points with a radius.
Has every point find every other point within 16 of that point.
*/
#[derive(Clone)]
struct PointAndRadii {
    id: usize,
    x: f32,
    y: f32,
    radius: f32,
    weight: f32,
}

impl Dimensions for PointAndRadii {
    fn num_dims() -> usize {
        2
    }
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

pub fn test() {
    let num_units = 2048;
    let mut rng = rand::thread_rng();

    let start1 = PreciseTime::now();
    let mut vec = Vec::with_capacity(num_units);
    for n in (0..num_units) {
        vec.push(PointAndRadii {id: n, x: rng.gen_range(0.0, 1024.0), y: rng.gen_range(0.0, 1024.0), radius: 0.5, weight: 0.5});
    }
    let mut kdt_clone = vec.clone();
    let kdt = KDTree::make_from(&mut kdt_clone);
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
            let dr = b.radius + 16.0;
            (dx * dx) + (dy * dy) <= dr * dr
        };
        let p = &pred as &Fn(&PointAndRadii) -> bool;
        let in_rng = kdt.get_in_range(p, &[(a.x,16.0),(a.y,16.0)]);
        let end2 = PreciseTime::now();
        total_kdt_search_time += start2.to(end2).num_nanoseconds().unwrap();
        total_in_rng1 += in_rng.len();
    }

    for a in vec.iter() {
        let start2 = PreciseTime::now();
        let pred = |b: &PointAndRadii| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dr = b.radius + 16.0;
            (dx * dx) + (dy * dy) <= dr * dr
        };
        let p = &pred as &Fn(&PointAndRadii) -> bool;
        let in_rng = vec.clone().into_iter().filter(|a| p(a)).collect::<Vec<_>>();
        let end2 = PreciseTime::now();
        total_search_time += start2.to(end2).num_nanoseconds().unwrap();
        total_in_rng2 += in_rng.len();
    }

    let mili = 1000000.0;
    println!("Build time: {}ms", start1.to(end1).num_nanoseconds().unwrap() as f32 / mili);
    println!("KDT search time: {}ms", total_kdt_search_time as f32 / mili);
    println!("Naive search time: {}ms", total_search_time as f32 / mili);
    println!("Improvement: {}", total_search_time as f32 / total_kdt_search_time as f32);
    println!("KDT in range: {}", total_in_rng1);
    println!("Naive in range: {}", total_in_rng2);
    println!("");
}