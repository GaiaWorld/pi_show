use std::hash::{Hash};
use std::sync::{Arc};
use common::{SamplerDesc};
use traits::context::{Context};

pub trait Sampler: Hash {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &SamplerDesc) -> Result<<Self::RContext as Context>::ContextSampler, String>;
    
    fn delete(&self);

    fn get_desc(&self) -> &SamplerDesc;
}