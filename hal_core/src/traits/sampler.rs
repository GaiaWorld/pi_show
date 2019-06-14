use std::sync::{Arc};
use common::{SamplerDesc};
use traits::context::{Context};

pub trait Sampler {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>, desc: &SamplerDesc) -> Result<<Self::RContext as Context>::ContextSampler, String>;

    fn delete(&self);

    /** 
     * 取唯一id，作为排序的依据
     */
    fn get_id(&self) -> u64;

    fn get_desc(&self) -> &SamplerDesc;
}