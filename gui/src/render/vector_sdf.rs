use render::data::{TexData12, BufferData12, BufferData6, BufferData4};
use web_sys::*;

//用于绘制矢量图形Sdf
pub struct VectorSdf {
    // 浮点纹理的大小，需要2的冥
    // 能放 SIZE * SIZE个vec4向量
    pub size: usize,
    positions_dirty: bool,
    indexs_dirty: bool,
    padding: f32, // 包围盒的边界离mesh多少像素

    //rect [center[0], center[1], extend[0], extend[1],
    //      color[0],  color[1],  color[2],  color[3],
    //      1,          radius,      0,          0      ]
    
    //cricel [center[0], center[1], radius[0], radius[1],
    //        color[0],  color[1],  color[2],  color[3],
    //        0,             0,        0,          0     ]
    tex_data: TexData12, // 送到纹理的参数数据
    
    // [left_top_x,     left_top_y,     left_top_z,
    //  left_bootom_x,  left_bootom_y,  left_bootom_z,
    //  right_bootom_x, right_bootom_y, right_bootom_z,
    //  right_top_x,    right_top_y,    right_top_z,]
    positions: BufferData12<f32>, // 网格顶点的位置
    //[count, count, count, count]
    mesh_index: BufferData4<u16>, // 网格索引
    indices: BufferData6<u16>, // 网格的索引
}

#[derive(Clone)]
pub struct Index{
    pub attribute: usize,
    pub tex: usize
}

impl VectorSdf {
    pub const TEX_LEN: usize = 12; // 每个mesh的tex_data长度

    pub fn new() -> VectorSdf {
        VectorSdf {
            size: 8,
            padding: 0.0,
            tex_data: TexData12::new(8),
            positions: BufferData12::new(),
            mesh_index: BufferData4::new(),
            indices: BufferData6::new(),
            positions_dirty: false,
            indexs_dirty: false,
        }
    }

    pub fn with_size(size: usize) -> VectorSdf {
        VectorSdf {
            size,
            padding: 5.0,
            tex_data: TexData12::new(size),
            positions: BufferData12::new(),
            mesh_index: BufferData4::new(),
            indices: BufferData6::new(),
            positions_dirty: false,
            indexs_dirty: false,
        }
    }

    //设置tex大小， 会清空原有的tex
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
        self.tex_data = TexData12::new(size);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    // 分配buffer空间
    pub fn alloc(&mut self) -> Index {
        let index = match self.tex_data.alloc() {
            Some(index) => index,
            None => panic!("overflow"),
        };
        let len  = self.mesh_index.len();
        console::log_2(&("alloc--len-----------".into()), &(len.to_string().into()));
        self.mesh_index.push(index_to_buffer(len as u16));
        self.indices.push(indices_to_buffer(len as u16));
        console::log_2(&("alloc--end-----------".into()), &(len.to_string().into()));
        
        self.positions.push(rect_box(0.0, 0.0, 0.0, 0.0, 0.0, self.padding));
        self.positions_dirty = true;
        self.indexs_dirty = true;
        Index{tex: index + 1, attribute: len + 1}
    }

    pub unsafe fn update_style(&mut self, border: f32, color: &cg::color::Color<f32>,  index: usize){
        let buffer = style_to_tex_buffer(border, color);
        self.tex_data.update(index - 1, 4, &buffer); 
    }

    //更新矩形的buffer， 该方法不会检查能否成功更新， 应该正确的传出index， 否则将会painc
    pub unsafe fn update_rect(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32 , radius: f32, z: f32, index: &Index){
        console::log_7(&("update_rect".into()), &(min_x.to_string().into()), &(min_y.to_string().into()), &(max_x.to_string().into()), &(max_y.to_string().into()), &(radius.to_string().into()), &(z.to_string().into()));
        let tex_index = index.tex - 1;
        console::log_2(&("update_rect tex_index".into()), &(tex_index.to_string().into()));
        let rect_buffer = rect_box(min_x, min_y, max_x, max_y, z, self.padding);
        self.positions.update(index.attribute - 1, 0, &rect_buffer);
        let tex_data = rect_to_tex_buffer(min_x, min_y, max_x, max_y, radius);//([cx, cy, rx, ry], [1.0, radius.clone(), 0.0, 0.0])
        self.tex_data.update(tex_index, 0, &tex_data.0); // [center.x, center.y, radius.x, radius.y]
        self.tex_data.update(tex_index, 8, &tex_data.1); // [1.0 (表示矩形), radius （圆角半径）, 未知, 未知]
        self.positions_dirty = true;
    }

    pub unsafe fn update_circle(&mut self, center: (f32, f32), radius: f32, z: f32, index: &Index){
        let tex_index = index.tex - 1;
        let circle_buffer = circle_box(center, radius, z, self.padding);
        self.positions.update(index.attribute - 1, 0, &circle_buffer);
        let tex_data = circle_to_tex_buffer(center, radius);//([cx, cy, rx, ry], [0.0, 0.0, 0.0, 0.0])
        self.tex_data.update(tex_index, 0, &tex_data.0); // [center.x, center.y, radius.x, radius.y]
        self.tex_data.update(tex_index, 8, &tex_data.1); // [1.0 (表示矩形), radius （圆角半径）, 未知, 未知]
        self.positions_dirty = true;
    }

    //删除矩形, 将末尾的元素移动至当前位置， 返回尾部元素的索引
    pub unsafe fn swap_delete_rect(&mut self, index: &Index) -> usize{
        self.tex_data.delete(index.tex - 1);
        self.positions.swap_delete(index.tex - 1);
        self.mesh_index.swap_delete(index.tex - 1);
        self.indices.swap_delete(index.tex - 1);
        self.positions.len() + 1
    }

    pub fn use_tex_data(&mut self) -> Option<((usize, usize, usize, usize), &[f32])> {
        self.tex_data.use_data()
    }

    pub fn use_positions_data(&mut self) -> Option<&[f32]>{
        if self.positions_dirty {
            self.positions_dirty = false;
            Some(self.positions.get_data())
        }else {
            None
        }
    }

    //return (mesh_index_buffer, indices_buffer)
    pub fn use_indexs_data(&mut self) -> Option<(&[u16], &[u16])>{
        if self.indexs_dirty {
            self.indexs_dirty = false;
            Some((self.mesh_index.get_data(), self.indices.get_data()))
        }else {
            None
        }
    }
}

#[inline]
fn rect_box(min_x: f32, min_y: f32, max_x: f32, max_y: f32, z: f32, padding: f32) -> [f32; 12]{
    [
        min_x - padding, min_y - padding, z,  // left_top
        min_x - padding, max_y + padding, z, // left_bootom
        max_x + padding, max_y + padding, z, // right_bootom
        max_x + padding, min_y - padding, z, // right_top
    ]
}

#[inline]
fn circle_box(center: (f32, f32), radius: f32, z: f32, padding: f32) -> [f32; 12]{
    let buffer = unsafe{js_sys::Float32Array::view(&[
        center.0 - radius - padding, center.1 - radius - padding, z,  // left_top
        center.0 - radius - padding, center.1 + radius + padding, z, // left_bootom
        center.0 + radius + padding, center.1 + radius + padding, z, // right_bootom
        center.0 + radius + padding, center.1 - radius - padding, z, // right_top
    ])};
    console::log_6(&("circle_box".into()), &buffer, &(center.0.to_string().into()), &(center.1.to_string().into()), &(radius.to_string().into()), &(z.to_string().into()));
    [
        center.0 - radius - padding, center.1 - radius - padding, z,  // left_top
        center.0 - radius - padding, center.1 + radius + padding, z, // left_bootom
        center.0 + radius + padding, center.1 + radius + padding, z, // right_bootom
        center.0 + radius + padding, center.1 - radius - padding, z, // right_top
    ]
}

#[inline]
fn rect_to_tex_buffer( min_x: f32, min_y: f32, max_x: f32, max_y: f32, radius: f32) -> ([f32; 4], [f32; 4]){
    let (rx, ry) = ((max_x - min_x)/2.0, (max_y - min_y)/2.0);
    let (cx, cy) = (min_x + rx, min_y + ry);
    ([cx, cy, rx, ry], [1.0, radius, 0.0, 0.0])
}

#[inline]
fn circle_to_tex_buffer(center: (f32, f32), radius: f32) -> ([f32; 4], [f32; 4]){
    ([center.0, center.1, radius, radius], [0.0, 0.0, 0.0, 0.0])
}

#[inline]
fn style_to_tex_buffer( _border: f32, color: &cg::color::Color<f32>) -> [f32; 4]{
    [color.r, color.g, color.b , color.a]
}

#[inline]
fn index_to_buffer(index: u16) -> [u16; 4]{
    [index, index, index, index]
}

    #[inline]
fn indices_to_buffer( index: u16) -> [u16; 6]{
    let pos_index = 4 * index; 
    [
        pos_index, pos_index + 1, pos_index + 2,
        pos_index, pos_index + 2, pos_index + 3,
    ]
}