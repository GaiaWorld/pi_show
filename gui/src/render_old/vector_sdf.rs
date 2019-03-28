use render::data::{TexData16, BufferData12, BufferData6, BufferData4};

//用于绘制矢量图形Sdf
pub struct VectorSdf {
    // 浮点纹理的大小，需要2的冥
    // 能放 SIZE * SIZE个vec4向量
    pub size: usize,
    positions_dirty: bool,
    indexs_dirty: bool,
    padding: f32, // 包围盒的边界离mesh多少像素

    // rect
    // [
    //     center[0], center[1], size, typeRadiusStroke, 
    //     angleAphaBlur, colorTypeSizeAngle, strokeRG, strokeBA, 
    //     colors[0], colors[1], colors[2], colors[3],
    //     colors[4], colors[5], clips[0], clips[1],
    // ]
    
    //cricel [center[0], center[1], radius[0], radius[1],
    //        color[0],  color[1],  color[2],  color[3],
    //        0,             0,        0,          0     ]
    tex_data: TexData16, // 送到纹理的参数数据
    
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

pub enum Color{
    RGBA([f32; 4]),
    LinearGradient([f32; 13]), // 方向， 颜色。。。
    RadialGradient([f32; 13]), // 尺寸， 颜色。。。
}

impl VectorSdf {
    pub const TEX_LEN: usize = 12; // 每个mesh的tex_data长度

    pub fn new() -> VectorSdf {
        VectorSdf {
            size: 8,
            padding: 0.0,
            tex_data: TexData16::new(8),
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
            tex_data: TexData16::new(size),
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
        self.tex_data = TexData16::new(size);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    // 分配buffer空间
    pub fn alloc(&mut self) -> usize {
        let index = match self.tex_data.alloc() {
            Some(index) => index,
            None => panic!("overflow"),
        };
        let len  = self.mesh_index.len();
        if (index * 4) > len {
            self.mesh_index.push(index_to_buffer(len as u16));
            self.indices.push(indices_to_buffer(len as u16));
            self.positions.push(rect_box(0.0, 0.0, 0.0, 0.0, 0.0, self.padding));
        }
        index + 1
    }

    pub unsafe fn update_style(&mut self, border: f32, color: &cg::color::Color<f32>,  index: usize){
        let buffer = style_to_tex_buffer(border, color);
        self.tex_data.update(index - 1, 4, &buffer); 
    }

    pub unsafe fn update_shape(
        &mut self,
        index: usize, //索引
        radius : f32, // number, 圆角半径

        ty: u32, // number TYPE_*

        center: [f32; 3], // [x, y, z]，中心坐标
        mut extend: [f32; 2], // [w, h], 半长
        
        // 裁剪面索引，一个物体最多受4个裁剪面影响
        // -1代表不管裁剪面直接渲染，-2代表该物体完全在裁剪面外
        // 只要有一个裁剪面索引为-2，该物体就肯定不会渲染
        clip_indices: Option<[f32; 4]>,

        alpha: f32, // 物体的alpha，范围[0, 255]
        mut angle: f32, // 旋转角度，顺时针，单位：度
        stroke_color: Option<[f32; 4]>, // [r, g, b, a]，描边颜色
        stroke_size: f32, // number 描边大小
        blur: f32, //= 1.0, // 对于阴影，这个参数一般设置5.0

        color_type: u32, // 0代表不渐变，1代表线性渐变，2代表径向渐变

        mut color_angle: f32, // 仅线性渐变用到，颜色渐变角度，顺时针，单位：度，0度代表从下到上渐变
        // colorCenter = [0, 0], // 仅径向渐变用到，渐变中心点，目前不支持。
        color_size: u32, // 仅径向渐变用到，渐变大小，0代表最近边，1代表最远边，2代表最近角，3代表最远角

        // [百分比, r, g, b, a], ...] 颜色，如果有多个一个颜色，那么会产生渐变效果
        color1: Option<[f32; 5]>, // [0, 255, 255, 255, 255], // 比例[0, 100]，颜色[0, 255]
        color2:  Option<[f32; 5]>, //[1000, 255, 255, 255, 255], // 比例[0, 100]，颜色[0, 255]
        color3: Option<[f32; 5]> //[1000, 255, 255, 255, 255], // 比例[0, 100]，颜色[0, 255]
        // 注：线性渐变的意思：沿着colorAngle的角度，那个百分比，就会产生对应的颜色。
        // color_type: u8, // 0代表不渐变，1代表线性渐变，2代表径向渐变

        // color_angle: f32, // 仅线性渐变用到，颜色渐变角度，顺时针，单位：度，0度代表从下到上渐变
        // // colorCenter = [0, 0], // 仅径向渐变用到，渐变中心点，目前不支持。
        // color_size: u8, // 仅径向渐变用到，渐变大小，0代表最近边，1代表最远边，2代表最近角，3代表最远角

        // [百分比, r, g, b, a], ...] 颜色，如果有多个一个颜色，那么会产生渐变效果
        // color1: Option<[f32; 5]>, // 比例[0, 100]，颜色[0, 255]
        // color2: Option<[f32; 5]> // 比例[0, 100]，颜色[0, 255]
        // color3: Option<[f32; 5]> [1000, 255, 255, 255, 255], // 比例[0, 100]，颜色[0, 255]
    ) {
        let index = index - 1;
        let mut clip_indices = match clip_indices {
            Some(v) => v,
            None => [-1.0, -1.0, -1.0, -1.0],
        };
        let stroke_color = match stroke_color {
            Some(v) => v,
            None => [255.0, 255.0, 255.0, 255.0],
        };

        let color1 = match color1 {
            Some(v) => v,
            None => [0.0, 255.0, 255.0, 255.0, 255.0],
        };

        let color2 = match color2 {
            Some(v) => v,
            None => [1000.0, 255.0, 255.0, 255.0, 255.0],
        };

        let color3 = match color3 {
            Some(v) => v,
            None => [1000.0, 255.0, 255.0, 255.0, 255.0],
        };

        // 填充相应的数据
        let [cx, cy, _cz] = center;

        // TODO: 椭圆的宽高，不能完全相等，否则shader会画不出来
        if extend[0] == extend[1] && extend[0] != 0.0 {
            extend[1] += 0.001;
        }

        let mut r = extend[0];
        if extend[1] > r {
            r = extend[1];
        }

        r = r + self.padding + stroke_size;

        self.positions.update(index, 0, &[
            cx - r, cy - r, center[2],
            cx - r, cy + r, center[2],
            cx + r, cy + r, center[2],
            cx + r, cy - r, center[2],
        ]);

        // 一个网格就是一个四边形，4个顶点
        let pos_index = 4 * (index as u16);
        self.indices.update(index, 0, &[
            pos_index + 0, pos_index + 1, pos_index + 2,
            pos_index + 0, pos_index + 2, pos_index + 3,
        ]);

        while color_angle < 0.0 {
            color_angle += 360.0;
        }
        while color_angle > 360.0 {
            color_angle = color_angle - 360.0;
        }
        let color_angle = color_angle as u32;

        let color_type_size_angle = 2048.0 * (color_type as f32) + 512.0 * (color_size as f32) + (color_angle as f32);
        
        let type_radius_stroke = (ty as f32) * 65536.0 + radius * 256.0 + stroke_size;

        while angle < 0.0 {
            angle += 360.0;
        }
        while angle > 360.0 {
            angle -= 360.0;
        }
        let angle = angle as u32;

        let angle_apha_blur = 8192.0 * (angle as f32) + 16.0 * alpha + blur;

        let size = extend[0] * 65536.0 + extend[1];

        let stroke_rg = stroke_color[0] * 256.0 + stroke_color[1];
        let stroke_ba = stroke_color[2] * 256.0 + stroke_color[3];
        
        let clips = get_clip_plane_indices(&mut clip_indices);
        
        let colors = get_gradient_colors(&mut [color1, color2, color3]);
        // 每一行是一个vec4，不能超过8个vec4
        self.tex_data.update(index, 0, &[
            center[0], center[1], size, type_radius_stroke,
            angle_apha_blur, color_type_size_angle, stroke_rg, stroke_ba, 
            colors[0], colors[1], colors[2], colors[3],
            colors[4], colors[5], clips[0], clips[1]
        ]);

        // self.mesh_index.update(index, 0, &[
        //     index, index, self.index, self.count,
        // ]);

        // this._count = get_gradient_colors;
    }

    //更新矩形的buffer， 该方法不会检查能否成功更新， 应该正确的传入index， 否则将会painc
    pub unsafe fn update_rect(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32 , radius: f32, z: f32, index: &Index){
        let tex_index = index.tex - 1;
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

/**
 * 输入 [ [percent, r, g, b, a], [percent, r, g, b, a], [percent, r, g, b, a] ]
 * 根据colorAngle提供分点的渐变色，如果不需要用，原点到分点的有向距离填10000
 * 返回6个number的数组，代表3个颜色，格式：[比例-rg1, ba1, 比例-rg2, ba2, 比例-rg3, ba3]
 */
fn get_gradient_colors(colors: &mut [[f32; 5]; 3]) ->  Vec<f32> {

    // 按百分比从小到大排序

    colors.sort_by(|a, b| {a[0].partial_cmp(&b[0]).unwrap()} );

    let mut result = Vec::new();

    for item in colors.iter() {
        if item[0] < 0.0 || item[0] > 100.0 {
            continue;
        }

        let percent_rg = item[0] * 65536.0 + item[1] * 256.0 + item[2];
        let ba = item[3] * 256.0 + item[4];
        result.push(percent_rg);
        result.push(ba);
    }

    for _ in result.len()..3 {
        result.push(127.0 * 65536.0);
        result.push(0.0);
    }

    result
}

/** 
 * 返回的索引全部往右移动2，以便于所有数字都是非负数
 * 编码为：[0-1, 2-3]
 */
fn get_clip_plane_indices(clip_indices: &mut [f32; 4]) -> [f32; 2] {

    // 按索引从小到大排序
    clip_indices.sort_by(|a, b| {a.partial_cmp(b).unwrap() });

    let clip01 = (clip_indices[0] + 2.0) * 256.0 + (clip_indices[1] + 2.0);
    let clip23 = (clip_indices[2] + 2.0) * 256.0 + (clip_indices[3] + 2.0);
    
    return [clip01, clip23];
}