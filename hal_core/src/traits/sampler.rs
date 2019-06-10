
use std::sync::{Arc};
use common::{SamplerDesc};
use traits::context::{Context};

pub trait Sampler {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &SamplerDesc) -> Self;
    fn delete(&self);
}