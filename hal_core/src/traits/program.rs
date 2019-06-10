use std::sync::{Arc};
use traits::context::{Context};

pub trait Program {
    type RContext: Context;

    fn new(context: &Arc<Self::RContext>) -> Self;
    fn delete(&self);
}