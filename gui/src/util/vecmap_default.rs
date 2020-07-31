
use std::ops::{Index, IndexMut};
use std::default::Default;

use map::vecmap::VecMap;
use map::Map;


pub struct VecMapWithDefault<T> {
	default_v: T,
	map: VecMap<T>,
}

impl<T: Default> Default for VecMapWithDefault<T> {
	fn default() -> Self {
		VecMapWithDefault {
			default_v: T::default(),
			map: VecMap::default(),
		}
	}
}

impl<T> Index<usize> for VecMapWithDefault<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        match self.map.get(index) {
			Some(r) => r,
			None => &self.default_v
		}
    }
}

impl<T> IndexMut<usize> for VecMapWithDefault<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        match self.map.get_mut(index) {
			Some(r) => r,
			None => &mut self.default_v
		}
    }
}

impl<T> Map for VecMapWithDefault<T> {
	type Key = usize;
	type Val = T;
    #[inline]
    fn get(&self, key: &usize) -> Option<&T> {
        self.map.get(*key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut T> {
        self.map.get_mut(*key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &usize) -> &T {
        self.map.get_unchecked(*key)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &usize) -> &mut T {
        self.map.get_unchecked_mut(*key)
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &usize) -> T {
        self.map.remove_unchecked(*key)
    }

    #[inline]
    fn insert(&mut self, key: usize, val: T) -> Option<T> {
        self.map.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<T> {
        self.map.remove(*key)
    }

    #[inline]
    fn contains(&self, key: &usize) -> bool {
        self.map.contains(*key)
    }

    #[inline]
    fn len(&self) -> usize {
        self.map.len()
    }
    #[inline]
    fn capacity(&self) -> usize {
        self.map.capacity()
    }
    #[inline]
    fn mem_size(&self) -> usize {
        self.map.capacity() * std::mem::size_of::<T>()
    }
}
