use crate::image::Image;
use crate::color::{rgb_to_hsl,hsl_to_rgb};


pub fn hsl(
    image: &mut Image,
    hue: f32,
    saturation: f32,
    luminosity: f32
) {

    for pixel in image.pixels.chunks_mut(4) {

        let (mut h,s,l) =
            rgb_to_hsl(
                pixel[0],
                pixel[1],
                pixel[2]
            );


        // Hue rotation
        h += hue;

        if h < 0.0 {
            h += 360.0;
        }

        if h > 360.0 {
            h -= 360.0;
        }


        // Saturation
        let s =
            (s * saturation)
            .clamp(0.0,1.0);


        // Luminosity
        let l =
            (l + luminosity)
            .clamp(0.0,1.0);


        let (r,g,b)=hsl_to_rgb(
            h,
            s,
            l
        );


        pixel[0]=r;
        pixel[1]=g;
        pixel[2]=b;
    }
}