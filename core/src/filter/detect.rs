use super::GpuFilter;

pub struct Wrap<'a, T>(&'a T);

impl<'a, T> Wrap<'a, T> {
    pub fn new(val: &'a T) -> Self {
        Wrap(val)
    }
}

pub trait GpuShaderFallback {
    fn ddot_gpu_shader(&self) -> Option<&'static str>;
}

impl<'a, T> GpuShaderFallback for &Wrap<'a, T> {
    fn ddot_gpu_shader(&self) -> Option<&'static str> {
        None
    }
}

pub trait GpuShaderDetect {
    fn ddot_gpu_shader(&self) -> Option<&'static str>;
}

impl<'a, T: GpuFilter> GpuShaderDetect for Wrap<'a, T> {
    fn ddot_gpu_shader(&self) -> Option<&'static str> {
        Some(self.0.gpu_shader())
    }
}
