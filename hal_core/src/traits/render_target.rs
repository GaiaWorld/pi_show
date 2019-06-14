use std::sync::{Arc};
use traits::context::{Context};

/** 
 * 渲染目标
 * 
 * 一个渲染目标，含如下缓冲区：
 * 
 *     颜色缓冲区，在WebGL2/OpenGL3.3/GLES3中，会有4个颜色缓冲区；
 *         颜色缓冲区一般是纹理，因为要取出来用；
 *     深度缓冲区，可以不用；
 *         深度缓冲区一般是RenderBuffer，不需要取出来
 *     模板缓冲取，可以不用；
 *         模板缓冲区一般是RenderBuffer，不需要取出来
 */

/** 
 * 用于渲染目标的Buffer，一般用于当作渲染目标的深度缓冲
 */
pub trait RenderBuffer {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextRenderBuffer, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_size(&self) -> (u32, u32);
}

/** 
 * 渲染目标
 */
pub trait RenderTarget {

    type RContext: Context;

    fn new(context: &Arc<Self::RContext>) -> Result<<Self::RContext as Context>::ContextRenderTarget, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    /** 
     * 取大小
     */
    fn get_size(&self) -> (u32, u32);

    /**
     * 取渲染目标中特定通道的纹理
     */
    fn get_color_texture(&self, index: u32) -> Option<Arc<<<Self as RenderTarget>::RContext as Context>::ContextTexture>>;
}