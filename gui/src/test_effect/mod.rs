// rust 测试100个世界矩阵的计算， 时间： 80微秒
// asm 测试100个世界矩阵的计算， 时间： 950微秒

use component::user::Matrix4;
use time::now_microsecond;
use cgmath::Zero;
use map::vecmap::VecMap;

#[test]
pub fn test() {
    test_cal_martix4();
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn test_cal_martix4() {
    let mut mats = VecMap::new();
    let mut dirty_mark = VecMap::new();
    let mut dirty = Vec::new();
    for i in 1..10000 {
        dirty.push(i);
        dirty_mark.insert(i, true);
        mats.insert(i, Matrix4::new(
            1.0 + i as f32, 2.0 + i as f32, 3.0 + i as f32, 4.0 + i as f32, 
            5.0 + i as f32, 6.0 + i as f32, 7.0 + i as f32, 8.0 + i as f32, 
            9.0 + i as f32, 10.0 + i as f32, 11.0 + i as f32, 13.0 + i as f32, 
           14.0 + i as f32, 15.0 + i as f32, 16.0 + i as f32, 17.0 + i as f32, 
        ));
    }

    let now = now_microsecond();
    let mut total = 0.0;
    let mut z = Matrix4::zero();
    for i in 1..10000 {
        let mark = unsafe {dirty_mark.get_unchecked_mut(i)};
        if *mark {
            *mark = false;
            z = z *  mats[i];
            total +=  mats[i][3].w;
        }
       
    }
    println!("cal matrix4: {}", now_microsecond() - now);
    println!("total: {}, z: {:?}", total, z);
}
