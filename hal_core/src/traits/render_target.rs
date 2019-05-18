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

use std::rc::{Rc};

use common::{RTAttachment};
use traits::texture::{Texture};

/** 
 * 用于渲染目标的Buffer，一般用于当作渲染目标的深度缓冲
 */
pub trait RenderBuffer: Drop {
    fn get_size() -> (u32, u32);
}

/** 
 * 渲染目标
 */
pub trait RenderTarget: Drop {

    type ContextTexture: Texture;
    type ContextRenderBuffer: RenderBuffer;

    /** 
     * 取渲染目标的宽高
     */
    fn get_size() -> (u32, u32);

    /**
     * 为渲染目标邦纹理
     */
    fn attach_texture(attachment: RTAttachment, texture: Rc<Self::ContextTexture>);
    
    /**
     * 为渲染目标邦纹理
     */
    fn attach_render_buffer(attachment: RTAttachment, buffer: Rc<Self::ContextRenderBuffer>);
    
    /**
     * 取渲染目标中特定通道的纹理
     */
    fn get_texture(attachment: RTAttachment) -> Option<Self::ContextTexture>;
}