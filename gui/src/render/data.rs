use std::ptr;
use std::cmp::{Ordering};

// use web_sys::*;

use heap::simple_heap::SimpleHeap;

#[macro_export]
macro_rules! buffer_data{
    ($name: ident,  $unit: expr) => {
        pub struct $name<T>{
            data: Vec<T>,
        }

        impl<T: Clone> $name<T> {
            pub fn new () -> $name<T>{
                // println!("new--------------------------");
                $name {
                    data: Vec::new(),
                }
            }

            pub fn with_len(capacity: usize) -> $name<T> {
                let mut arr = Vec::with_capacity(capacity);
                unsafe{arr.set_len(capacity)};
                $name {
                    data: arr,
                }
            }

            pub fn with_data(data: Vec<T>) -> $name<T> {
                $name {
                    data: data,
                }
            }

            pub fn push (&mut self, data: [T; $unit]){
                let len = self.data.len();
                let new_len = len + $unit;
                if new_len > self.data.capacity() {
                    self.data.reserve($unit);
                }
                // console::log_3(&("len-----------".into()), &(len.to_string().into()), &(new_len.to_string().into()));
                unsafe{
                    self.data.set_len(new_len);
                    // console::log_3(&("ptr-----------".into()), &((self.data.as_mut_ptr() as usize).to_string().into()), &($unit.to_string().into()));
                    // println!("ptr----------{}", self.data.as_mut_ptr() as usize);
                    ptr::copy_nonoverlapping(data.as_ptr(), self.data.as_mut_ptr().add(len), $unit);
                }
            }

            //删除一组数据， 并使用尾部数据填充, 返回当前位置新的数据的旧索引
            // Panics if index is out of bounds.
            pub unsafe fn swap_delete (&mut self, index: usize) {
                let len = self.data.len();
                let i = index * $unit;
                ptr::copy(self.data.as_ptr().add(len - $unit), self.data.as_mut_ptr().add(i), $unit);
                self.data.set_len(len - $unit);
            }
            
            //更新数据
            // Panics if index is out of bounds. Panics if data.len() > $unit - offset
            pub unsafe fn update (&mut self, index: usize, offset: usize, data: &[T]){
                assert!($unit - offset >= data.len()); //data长度不能超出index位置的数据段 
                ptr::copy(data.as_ptr(), self.data.as_mut_ptr().add(index * $unit + offset), data.len());
            }

            //data的长度必须等于self.unit, 否则painc
            pub unsafe fn get_unchecked_mut(&mut self, index: usize ) -> &mut [T; $unit]{
                &mut *(self.data.as_mut_ptr().add(index * $unit) as usize as *mut [T; $unit])
            }

            #[inline]
            pub fn get_unit(&self) -> usize{
                $unit
            }
            
            #[inline]
            pub fn len(&self) -> usize{
                self.data.len()/$unit
            }

            #[inline]
            pub fn get_data(&self) -> &[T]{
                self.data.as_slice()
            }
        }
    }
}

buffer_data!(BufferData12, 12);
buffer_data!(BufferData4, 4);
buffer_data!(BufferData6, 6);

#[macro_export]
macro_rules! tex_data{
    ($name: ident, $buffer_name: ident,  $unit: expr) => {
        pub struct $name{
            buffer: $buffer_name<f32>, //buffer数据
            vacancy_list: SimpleHeap<usize>,
            dirty_block: (usize, usize), // 描述Tex改变的最大范围， (左上x, 左上y, 右下x, 右下y), 从1开始, 如果是(0,0,0,0)
            size: usize,
        }

        impl $name {
            pub fn new (size: usize) -> $name{
                let buffer_len = size * size * 4; //每个像素包含rgba四个元素
                let heap_len = buffer_len/$unit;
                let mut min_heap = SimpleHeap::new(Ordering::Less);
                for i in 0..heap_len{
                    min_heap.push(i);
                }

                let buffer = $buffer_name::with_len(buffer_len);
                $name {
                    buffer,
                    vacancy_list: min_heap,
                    dirty_block: (0, buffer_len),
                    size,
                }
            }

            //分配一个数据
            pub fn alloc (&mut self) -> Option<usize>{
                match self.vacancy_list.pop() {
                    Some(index) => Some(index),
                    None => None,
                }
            }

            //更新纹理数据，不会检查index是否是一个空位， 如果是， 那么之后该位置会被使用，  index和offset以及data溢出时会painc
            pub unsafe fn update (&mut self, index: usize, offset: usize, data: &[f32]){
                self.buffer.update(index, offset, data);
                self.update_dirty(index);
            }

            #[inline]
            pub unsafe fn delete (&mut self, index: usize) {
                self.vacancy_list.push(index);
            }

            //self.dirty_block != (0, 0,0,0), 否则panic
            #[inline]
            pub fn use_data(&mut self) -> Option<((usize, usize, usize, usize), &[f32])>{
                if self.is_dirty() {
                    
                    let line_start = ((self.dirty_block.0)/4)/self.size;
                    let column_start = ((self.dirty_block.0)/4)%self.size;
                    let line_end = ((self.dirty_block.1 - 1)/4)/self.size;
                    let column_end = ((self.dirty_block.1 - 1)/4)%self.size;
                    let block = if line_end > line_start {
                        (line_start, 0, self.size, line_end - line_start + 1)
                    }else {
                        (line_start, column_start, column_end - column_start + 1 , 1)
                    };
                    println!("{:?}", block);
                    // let dirty_block = self.dirty_block;
                    self.dirty_block = (0, 0);
                    let line_len = 4*self.size;
                    Some((block, &self.buffer.get_data()[block.0*line_len + block.1*4..(block.0 + block.3 - 1)*line_len + (block.1 + block.2)*4]))
                } else {
                    None
                }
            }

            #[inline]
            pub fn cancel_dirty(&mut self){
                self.dirty_block = (0, 0);
            }

            #[inline]
            pub fn is_dirty(&mut self) -> bool {
                if self.dirty_block.0 == self.dirty_block.1 {
                    false
                }else {
                    true
                }
            }

            //更新脏范围
            fn update_dirty (&mut self, index: usize){
                let offset = index*$unit;

                if self.dirty_block.0 == self.dirty_block.1 { //没有脏数据
                    self.dirty_block = (offset,offset + $unit);
                    return;
                }
                if offset < self.dirty_block.0 {
                    self.dirty_block.0 = offset;
                }else{
                    let offset = offset + $unit;
                    if offset > self.dirty_block.1 {
                        self.dirty_block.1  = offset;
                    }
                }
            }
        }
    }
}

tex_data!(TexData12, BufferData12, 12);

#[test]
fn test_tex_data(){
    // let mut tex_data = TexData12::new(8);
    // tex_data.use_data();

    // unsafe{tex_data.update(0, 0, &[1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,])};
    // unsafe{tex_data.update(1, 0, &[1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,])};
    // unsafe{tex_data.update(2, 0, &[1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,])};
    // let data = tex_data.use_data().unwrap();
    // println!("{:?}, {:?}, {}", data.1, data.0, data.1.len());
}

#[test]
fn test_buffer_data(){

    // let mut arr1: Vec<u16> = Vec::new();
    // arr1.push(1);
    // arr1.reserve(1);
    // println!("ptr----------{}", arr1.as_mut_ptr() as usize);
    
    // let mut arr2: Vec<u16> = Vec::new();
    // println!("ptr----------{}", arr2.as_mut_ptr() as usize);
    // let mut buffer_data4: BufferData4<u16> = BufferData4::new();
    // let mut buffer_data6: BufferData6<u16> = BufferData6::new();
    // buffer_data4.push([1, 1, 1, 1]);
    // buffer_data6.push([1, 1, 1, 1, 1, 1]);
}