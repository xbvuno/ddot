use crate::{
    image::Image,
    filter::FilterParams,
};


pub trait Filter {

    type Params: FilterParams;


    fn name(&self) -> &'static str;

 
    fn apply(
        &self,
        image: &mut Image,
        params: &Self::Params
    );
}
