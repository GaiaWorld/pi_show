use std::hash::{Hash};
use std::sync::{Arc};
use common::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use traits::context::{Context};

/** 
 * 渲染状态的trait
 */

pub trait BlendState: Hash {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &BlendStateDesc) -> Result<<Self::RContext as Context>::ContextBlendState, String>;
    
    fn delete(&self);

    fn get_desc(&self) -> &BlendStateDesc;
}

pub trait DepthState: Hash {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &DepthStateDesc) -> Result<<Self::RContext as Context>::ContextDepthState, String>;
    
    fn delete(&self);

    fn get_desc(&self) -> &DepthStateDesc;
}

pub trait RasterState: Hash {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &RasterStateDesc) -> Result<<Self::RContext as Context>::ContextRasterState, String>;
    
    fn delete(&self);

    fn get_desc(&self) -> &RasterStateDesc;
}

pub trait StencilState: Hash {
    type RContext: Context;
    
    fn new(context: &Arc<Self::RContext>, desc: &StencilStateDesc) -> Result<<Self::RContext as Context>::ContextStencilState, String>;
    
    fn delete(&self);
    
    fn get_desc(&self) -> &StencilStateDesc;
}