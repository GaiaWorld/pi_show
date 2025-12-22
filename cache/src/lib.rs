// extern crate bincode;
#[macro_use]
extern crate serde;

use minicbor::{decode, encode, Decode, Encode};
use pi_cache::{Cache as Cache1, CapacityIter, CapacityRefIter, Data, FrequencyState, ItemMeta, Iter, Metrics, TimeoutIter, TimeoutRefIter};
use serde::{Deserialize, Serialize};
use std::result;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Cache {
    cache: Cache1<String, CacheData>,
}


impl<'b> Decode<'b, [usize; 2]> for Cache {
    fn decode(d: &mut decode::Decoder<'b>, ctx: &mut [usize; 2]) -> Result<Self, decode::Error> {
        let len_outer = d.array()?.ok_or(decode::Error::message("Expected fixed outer array"))?;
        let mut cache = Cache::with_config(ctx[0], ctx[1]);
        for i in 0..len_outer {
            let len_inner = d.array()?.ok_or(decode::Error::message("Expected fixed inner array"))?;
            let key = d.str()?;
            let frequency = d.u8()?;
            let size = d.u32()? as usize;
            let timeout = d.u32()? as usize;
            cache.put_with_frequency(key, size, timeout, frequency as u32);
        }
        Ok(cache)
    }
}

impl<C> Encode<C> for Cache {
    fn encode<W: encode::Write>(&self, e: &mut encode::Encoder<W>, ctx: &mut C) -> Result<(), encode::Error<W::Error>> {
        let item_metas: Vec<ItemMeta<String>> = self.cache.items_metas();
        e.array(item_metas.len() as u64)?;
        // 编码每个 ItemMeta
        for item_meta in item_metas.iter() {
            e.array(4)?;
            e.str(&item_meta.key)?;
            e.u8(item_meta.frequency)?;
            e.u32(item_meta.size as u32)?;
            e.u32(item_meta.timeout as u32)?;
        }
        Ok(())
    }
}

pub struct CacheData {
    size: usize,
    timeout: usize, // 单位为：ms
}

impl Data for CacheData {
    fn size(&self) -> usize { self.size }
    /// 数据的超时时间，毫秒时间
    fn timeout(&self) -> u64 { self.timeout as u64 }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Cache {
    /// 用初始表大小，CuckooFilter窗口大小，整理率创建Cache
    pub fn with_config(
        map_capacity: usize,
        // cuckoo_filter_window_size: usize,
        frequency_down_rate: usize,
    ) -> Self {
        Self {
            cache: Cache1::with_config(map_capacity, frequency_down_rate),
        }
    }
    /// 判断是否有指定键的数据
    pub fn contains_key(&self, k: &str) -> bool { self.cache.contains_key(&k.to_string()) }
    /// 获得指定键的频次
    pub fn get_frequency(&self, k: &str) -> i8 {
        match self.cache.get_frequency(&k.to_string()) {
            FrequencyState::None => -1,
            FrequencyState::TakenAway => -2,
            FrequencyState::Garbaged => -3,
            FrequencyState::Frequency(frequency) => frequency as i8,
        }
    }
    /// 获得指定键的size
    pub fn getSize(&self, k: &str) -> Option<u32> {
        let v = self.cache.get(&k.to_string())?;
        Some(v.size as u32)
    }

    pub fn getKeys(&self) -> Vec<String> {
        let mut arr: Vec<String> = Vec::new();
        for r in self.cache.iter() {
            arr.push(r.0.clone());
        }
        arr
    }

    /// GetMut by key
    // pub fn get_mut(&mut self, k: &String) -> Option<&mut CacheData> { self.cache.get_mut(k) }
    /// adjust size
    // pub fn adjust_size(&mut self, size: isize) { self.cache.adjust_size(size); }
    /// 拿走的数据， 如果拿到了数据，就必须保证会调用put还回来
    pub fn take(&mut self, k: &str) { self.cache.take(&k.to_string()); }
    // /// 放入的数据，返回Some(V)表示被替换的数据
    pub fn put(&mut self, k: &str, size: usize, timeout: usize) {
        let cache_data = CacheData { size, timeout };
        self.cache.put(k.to_string(), cache_data);
    }
    /// 带频次的数据插入  不会进行降频
    pub fn put_with_frequency(&mut self, k: &str, size: usize, timeout: usize, frequency: u32) {
        let cache_data = CacheData { size, timeout };
        self.cache.put_with_frequency(k.to_string(), cache_data, frequency);
    }
    /// 激活并获取可写应用，会增加频次和最后使用时间，等于拿走并立即还回来，但性能更高
    pub fn active_mut(&mut self, k: &str) { self.cache.active_mut(&k.to_string()); }
    /// 移走
    pub fn remove(&mut self, k: &str) { self.cache.remove(&k.to_string()); }
    /// 将指定键的数据标记为垃圾回收，从频率队列中移除，但保留数据本身， 返回数据引用
    // pub fn garbage(&mut self, k: &String) -> Option<&CacheData> { self.cache.garbage(k) }
    /// 移走垃圾回收的数据
    // pub fn collect(&mut self, k: &str) -> Option<CacheData> { self.cache.collect(k.to_string()) }
    /// 全部的频率信息数量，包括被拿走的数据
    pub fn frequency_len(&self) -> usize { self.cache.frequency_len() }
    /// 当前数量，被缓存的数据数量
    pub fn len(&self) -> usize { self.cache.len() }
    /// 当前缓存的数据总大小
    pub fn size(&self) -> usize { self.cache.size() }
    // /// 获得当前的统计
    // pub fn metrics(&self) -> Metrics { std::mem::transmute(self.cache.metrics()) }
    // /// 迭代器，按频率由低到高，同频率先进先出的顺序迭代
    // pub fn iter(&self) -> Iter<'_, String, CacheData> { self.cache.iter() }
    // /// 超时整理方法， 参数为最小容量及毫秒时间，清理最小容量外的超时数据
    // pub fn timeout_ref_collect(&mut self, capacity: usize, now: u64) -> TimeoutRefIter<'_, String, CacheData> {
    //     self.cache.timeout_ref_collect(capacity, now)
    // }
    // /// 超量整理方法， 参数为容量， 按照频率优先， 同频先进先出的原则，清理超出容量的数据
    // pub fn capacity_ref_collect(&mut self, capacity: usize, ) -> CapacityRefIter<'_, String, CacheData> { self.cache.capacity_ref_collect(capacity) }
    // /// 超时整理方法， 参数为最小容量及毫秒时间，清理最小容量外的超时数据
    // pub fn timeout_collect(&mut self, capacity: usize, now: u64) -> TimeoutIter<'_, String, CacheData> { self.cache.timeout_collect(capacity, now) }
    /// 超量整理方法， 参数为容量， 按照频率优先， 同频先进先出的原则，清理超出容量的数据
    pub fn capacity_collect(&mut self, capacity: usize) -> Vec<String> {
        let mut arr: Vec<String> = Vec::new();
        for r in self.cache.capacity_collect(capacity) {
            let k = r.0;
            arr.push(k);
        }
        arr
    }

    // CBOR序列化
    pub fn serialize(&mut self) -> Option<Vec<u8>> { minicbor::to_vec(&self).ok() }

    // CBOR反序列化
    pub fn deserialize(bin: Vec<u8>, map_capacity: usize, frequency_down_rate: usize) -> Option<Self> {
        minicbor::decode_with::<[usize; 2], Cache>(&bin, &mut [map_capacity, frequency_down_rate]).ok()
    }
}


#[cfg(test)]
mod test_mod {
    use crate::*;
    #[test]
    pub fn serialize_test() {
        let mut cache = Cache::with_config(10, 10);
        cache.put("1", 1000, 20);
        cache.put("2", 1000, 20);
        cache.put("3", 1000, 20);
        cache.put("1", 1000, 20);
        cache.put("1", 1000, 20);
        let u8 = minicbor::to_vec(&cache).unwrap();
        std::fs::write("path", &u8);
        let cache2 = minicbor::decode_with::<[usize; 2], Cache>(&u8, &mut [1, 2]).unwrap();
        assert_eq!(cache2.get_frequency(&"1"), 2);
        assert_eq!(cache2.get_frequency(&"2"), 0);
        assert_eq!(cache2.get_frequency(&"3"), 0);
        println!("==== keys {:?}", cache.getKeys())
    }
}
