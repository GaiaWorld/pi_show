use common::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use traits::context::{Context};

/** 
 * 渲染状态的trait
 */

pub trait BlendState : Sized + Clone {
    type RContext: Context;

    fn new(context: &Self::RContext, desc: &BlendStateDesc) -> Result<<Self::RContext as Context>::ContextBlendState, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_desc(&self) -> &BlendStateDesc;
}

pub trait DepthState : Sized + Clone {
    type RContext: Context;

    fn new(context: &Self::RContext, desc: &DepthStateDesc) -> Result<<Self::RContext as Context>::ContextDepthState, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_desc(&self) -> &DepthStateDesc;
}

pub trait RasterState : Sized + Clone {
    type RContext: Context;

    fn new(context: &Self::RContext, desc: &RasterStateDesc) -> Result<<Self::RContext as Context>::ContextRasterState, String>;
    
    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_desc(&self) -> &RasterStateDesc;
}

pub trait StencilState : Sized + Clone {
    type RContext: Context;
    
    fn new(context: &Self::RContext, desc: &StencilStateDesc) -> Result<<Self::RContext as Context>::ContextStencilState, String>;
    
    fn delete(&self);
    
    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;
    
    fn get_desc(&self) -> &StencilStateDesc;
}