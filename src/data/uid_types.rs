extern crate core;
extern crate num;

use self::core::marker::PhantomData;
use std::collections::vec_deque::VecDeque;
use std::ops::{Index, IndexMut};
use std::fmt::Debug;

pub unsafe trait USizeWrapper {
    unsafe fn usize_unwrap(self) -> usize;
    unsafe fn usize_wrap(usize) -> Self;
}

#[derive(Clone, Debug)]
pub struct VecUID<UID, T> {
    vec: Vec<T>,
    index_type: PhantomData<UID>,
}

impl<UID, T: Clone> VecUID<UID, T> {
    pub fn full_vec(size: usize, default: T) -> VecUID<UID, T> {
        let mut vec = Vec::with_capacity(size);
        for _ in 0..size {
            vec.push(default.clone());
        }

        VecUID {
            vec: vec,
            index_type: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

impl<UID: USizeWrapper, T> Index<UID> for VecUID<UID, T> {
    type Output = T;

    fn index(&self, ix: UID) -> &T {
        unsafe { self.vec.get_unchecked(ix.usize_unwrap()) }
    }
}

impl<UID: USizeWrapper, T> IndexMut<UID> for VecUID<UID, T> {
    fn index_mut(&mut self, ix: UID) -> &mut T {
        unsafe { &mut self.vec[ix.usize_unwrap()] }
    }
}

pub struct UIDPool<T> {
    available_ids: VecDeque<T>,
    iteratable_ids: Vec<T>,
}

impl<T: USizeWrapper + Ord + Copy + Debug> UIDPool<T> {
    pub fn new(size: usize) -> UIDPool<T> {
        let mut available_ids = VecDeque::with_capacity(size);
        let mut c: usize = size;

        while c > 0 {
            c -= 1;
            unsafe {
                available_ids.push_front(T::usize_wrap(c));
            }
        }
        UIDPool {
            available_ids: available_ids,
            iteratable_ids: Vec::with_capacity(size),
        }
    }

    pub fn get_id(&mut self) -> Option<T> {
        match self.available_ids.pop_front() {
            Some(id) => {
                match self.iteratable_ids.binary_search(&id) {
                    Ok(_) => {
                        println!("I don't know how you did it, but you took the same ID from a UIDPool twice.");
                        None
                    }
                    Err(i) => {
                        self.iteratable_ids.insert(i, id);
                        Some(id)
                    }
                }
            }
            None => None,
        }
    }

    pub fn put_id(&mut self, id: T) {
        match self.iteratable_ids.binary_search(&id) {
            Ok(i) => {
                self.available_ids.push_back(id);
                self.iteratable_ids.remove(i);
            }
            Err(_) => {
                println!(
                    "You tried to put the same ID into a UIDPool twice. {:?}",
                    id
                );
            }
        }
    }

    pub fn iter(&self) -> Vec<T> {
        self.iteratable_ids.to_vec()
    }
}

macro_rules! id_wrappers {
    ( $( $x:ident ),* $(,)* ) => {
        $(
            #[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
            pub struct $x(usize);

            unsafe impl USizeWrapper for $x {
                unsafe fn usize_unwrap(self) -> usize {
                    let $x(ix) = self;
                    ix
                }
                unsafe fn usize_wrap(id: usize) -> $x {
                    $x(id)
                }
            }
        )*
    }
}

id_wrappers!(
    UnitID,
    TeamID,
    WeaponID,
    MissileID,
    UnitTypeID,
    WeaponTypeID,
    MissileTypeID,
    OrderID
);

macro_rules! copy_or_borrow_getters_setters_soa {
    ($struct_name:ident, $field_name:ident, $set_field:ident, copy, $ty:ty ) => {
        impl $struct_name {
                pub fn $field_name(&self, id: UnitID) -> $ty {
                    self.$field_name[id]
                }

                pub fn $set_field(&mut self, id: UnitID, val: $ty) {
                    self.$field_name[id] = val;
                }
        }
    };
    ($struct_name:ident, $field_name:ident, $mut_field_name:ident, borrow, $ty:ty ) => {
        impl $struct_name {
                pub fn $field_name(&self, id: UnitID) -> &$ty {
                    &self.$field_name[id]
                }

                pub fn $mut_field_name(&mut self, id: UnitID) -> &mut $ty {
                    &mut self.$field_name[id]
                }
        }
    };
    ($struct_name:ident, $field_name:ident, $mut_field_name:ident, none, $ty:ty ) => ();
}

macro_rules! uid_soa {
    ($struct_name: ident, $uid: ty, $( ($field_name: ident, $set_field: ident, $ty: ty, $copy_or_borrow: ident, $expr: expr) ),* ) => {
        pub struct $struct_name {
            available_ids: UIDPool<$uid>,
            prototypes: VecUID<UnitTypeID,ProtoUnit>,
            $(
                $field_name: VecUID<$uid,$ty>
            ),*
        }

        impl $struct_name {
            pub fn new(num: usize, prototypes: VecUID<UnitTypeID,ProtoUnit>) -> $struct_name {
                let available_ids = UIDPool::new(num);

                $struct_name {
                    available_ids: available_ids,
                    prototypes: prototypes,
                    $(
                        $field_name: VecUID::full_vec(num, $expr)
                    ),*
                }
            }
        }

        $(
            copy_or_borrow_getters_setters_soa!($struct_name, $field_name, $set_field, $copy_or_borrow, $ty);
        )*
    }
}

macro_rules! copy_or_borrow_getters_setters_aos {
    ($struct_name:ident, $field_name:ident, $set_field:ident, copy, $ty:ty ) => {
        impl Units {
                pub fn $field_name(&self, id: UnitID) -> $ty {
                    self.elements[id].$field_name
                }

                pub fn $set_field(&mut self, id: UnitID, val: $ty) {
                    self.elements[id].$field_name = val;
                }
        }
    };
    ($struct_name:ident, $field_name:ident, $mut_field_name:ident, borrow, $ty:ty ) => {
        impl Units {
                pub fn $field_name(&self, id: UnitID) -> &$ty {
                    &self.elements[id].$field_name
                }

                pub fn $mut_field_name(&mut self, id: UnitID) -> &mut $ty {
                    &mut self.elements[id].$field_name
                }
        }
    };
    ($struct_name:ident, $field_name:ident, $mut_field_name:ident, none, $ty:ty ) => ();
}

macro_rules! uid_aos {
    ( $struct_name: ident, $proto_name: ident, $uid: ty, $( ($field_name: ident, $set_field: ident, $ty: ty, $copy_or_borrow: ident, $expr: expr) ),* ) => {
        #[derive(Clone)]
        pub struct $proto_name {
            $(
                $field_name: $ty
            ),*
        }

        impl $proto_name {
            $(
                pub fn $set_field(&mut self, val: $ty) {
                    self.$field_name = val;
                }
            )*
        }

        pub struct $struct_name {
            available_ids: UIDPool<$uid>,
            prototypes: VecUID<UnitTypeID,ProtoUnit>,
            elements: VecUID<$uid,$proto_name>,
        }

        impl $struct_name {
            pub fn new(num: usize, prototypes: VecUID<UnitTypeID,ProtoUnit>) -> $struct_name {
                let available_ids = UIDPool::new(num);
                let element = $proto_name {
                    $(
                        $field_name: $expr
                    ),*
                };

                $struct_name {
                    available_ids: available_ids,
                    prototypes: prototypes,
                    elements: VecUID::full_vec(num, element)
                }
            }
        }

        $(
            copy_or_borrow_getters_setters_aos!($struct_name, $field_name, $set_field, $copy_or_borrow, $ty);
        )*
    }
}
