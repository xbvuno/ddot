use crate::{filter::FilterParams, image::Image};

pub trait Filter {
    type Params: FilterParams;

    fn name(&self) -> &'static str;

    fn apply(&self, image: &mut Image, params: &Self::Params);
}
