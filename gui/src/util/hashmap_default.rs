
use std::ops::{Index, IndexMut};
use std::default::Default;

use map::hashmap::HashMap;
use map::Map;


pub struct HashMapWithDefault<T> {
	default_v: T,
	map: HashMap<usize, T>,
}

impl<T: Default> Default for HashMapWithDefault<T> {
	fn default() -> Self {
		HashMapWithDefault {
			default_v: T::default(),
			map: HashMap::default(),
		}
	}
}

impl<T: Default> Index<usize> for HashMapWithDefault<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        match self.map.get(&index) {
			Some(r) => r,
			None => &self.default_v
		}
    }
}

impl<T: Clone + Default> IndexMut<usize> for HashMapWithDefault<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
		unsafe { Map::get_unchecked_mut(self, &index) }
    }
}

impl<T: Clone + Default> Map for HashMapWithDefault<T> {
	type Key = usize;
	type Val = T;

	#[inline]
	fn with_capacity(capacity: usize) -> HashMapWithDefault<T> {
        HashMapWithDefault {
            default_v: T::default(),
			map: HashMap::with_capacity(capacity),
        }
	}
	
    #[inline]
    fn get(&self, key: &usize) -> Option<&T> {
        self.map.get(key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut T> {
        self.map.get_mut(key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &usize) -> &T {
        self.map.get_unchecked(key)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &usize) -> &mut T {
		// 所以这个get了两次，优化TODO
		match self.map.get(key){
			Some(_r) => self.map.get_mut(key).unwrap(),
			None => {
				self.map.insert(*key, self.default_v.clone());
				self.map.get_mut(key).unwrap()
			}
		}
		
        // self.map.get_unchecked_mut(*key)
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &usize) -> T {
        self.map.remove_unchecked(key)
    }

    #[inline]
    fn insert(&mut self, key: usize, val: T) -> Option<T> {
        self.map.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<T> {
        self.map.remove(key)
    }

    #[inline]
    fn contains(&self, key: &usize) -> bool {
        self.map.contains(key)
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

// impl<T: Default> HashMapWithDefault<T> {
// 	#[inline]
//     fn get_mut1<'a>(&'a mut self, key: &'a usize) -> Option<&'a mut T> {
// 		if self.map.get(*key).is_some() {
// 			return self.map.get_mut(*key);
// 		}
		
// 		self.map.insert(*key, T::default());
// 		self.map.get_mut(*key)
//         // self.map.get_mut(*key)
//     }
// }
