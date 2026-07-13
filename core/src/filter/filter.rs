use crate::{filter::{FilterParams, BackendSupport}, image::Image};

pub trait GpuFilter {
    fn gpu_shader(&self) -> &'static str;
}

pub trait Filter {
    type Params: FilterParams;

    fn name(&self) -> &'static str;

    fn apply(&self, image: &mut Image, params: &Self::Params);

    fn backend_support(&self) -> BackendSupport;

    fn gpu_shader(&self) -> Option<&'static str>;
}
