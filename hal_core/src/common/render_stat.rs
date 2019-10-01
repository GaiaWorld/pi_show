// 渲染统计情况
#[derive(Debug)]
pub struct RenderStat {
    pub rt_count: i32,
    pub texture_count: i32,
    pub buffer_count: i32,
    pub geometry_count: i32,
    pub program_count: i32,

    // 每帧统计的信息，切换了多少个相应的东西
    pub rt_change_count: i32,
    pub geometry_change_count: i32,
    pub texture_change_count: i32,
    pub program_change_count: i32,
    pub draw_call_count: i32,
}

impl RenderStat {
    pub fn new() -> Self {
        Self {
            rt_count: 0,
            texture_count: 0,
            buffer_count: 0,
            geometry_count: 0,
            program_count: 0,

            rt_change_count: 0,
            geometry_change_count: 0,
            texture_change_count: 0,
            program_change_count: 0,
            draw_call_count: 0,
        }
    }

    pub fn reset_frame(&mut self) {
        self.rt_change_count = 0;
        self.geometry_change_count = 0;
        self.texture_change_count = 0;
        self.program_change_count = 0;
        self.draw_call_count = 0;
    }

    pub fn add_geometry_change(&mut self) {
        self.geometry_change_count += 1;
    }

    pub fn add_texture_change(&mut self, count: i32) {
        self.texture_change_count += count;
    }

    pub fn add_program_change(&mut self) {
        self.program_change_count += 1;
    }

    pub fn add_rt_change(&mut self) {
        self.rt_change_count += 1;
    }

    pub fn add_draw_call(&mut self) {
        self.draw_call_count += 1;
    }
}
