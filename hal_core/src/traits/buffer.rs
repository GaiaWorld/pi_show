use traits::context::{Context};

#[derive(PartialEq, Clone, Copy, Debug, Hash)]
pub enum BufferType {
    Attribute,
    Indices,
}

pub enum BufferData<'a> {
    Float(&'a[f32]),
    Short(&'a[u16]),
    Int(&'a[i32]),
}

/** 
 * Buffer：显存的抽象
 */
pub trait Buffer : Sized + Clone {

    type RContext: Context;

    /** 
     * is_updatable表示是否需要更新，根据这个来让显卡决定将该buffer放到不同的地方，以便显卡优化性能。
     */
    fn new(context: &Self::RContext, btype: BufferType, data: Option<BufferData>, is_updatable: bool) -> Result<<Self::RContext as Context>::ContextBuffer, String>;

    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    /**
     * 更新数据
     * offset：单位是BufferData对应的类型单位。
     *    如果BufferData是Float，那么offet的单位就是1个float
     * 注：如果一开始就要更新数据，那么new的时候，尽量使用 is_updatable = true 来创建buffer。
     * 注：偏移 + data的长度 <= 创建时候的大小
     */
    fn update(&self, offset: usize, data: BufferData);
}