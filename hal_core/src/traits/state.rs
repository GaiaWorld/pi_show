use std::sync::{Arc};
use common::{BlendStateDesc, DepthStateDesc, RasterStateDesc, StencilStateDesc};
use traits::context::{Context};

/** 
 * 渲染状态的trait
 */

pub trait BlendState {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &BlendStateDesc) -> Self;
    fn delete(&self);
}

pub trait DepthState {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &DepthStateDesc) -> Self;
    fn delete(&self);
}

pub trait RasterState {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &RasterStateDesc) -> Self;
    fn delete(&self);
}

pub trait StencilState {
    type RContext: Context;
    
    fn new(context: &Arc<Self::RContext>, desc: &StencilStateDesc) -> Self;
    fn delete(&self);
}