use std::{
    error::Error,
    path::{Path, PathBuf},
};

use clap::Parser;
use ddot_core::{
    filter::FilterParams,
    filters::{Adjustment, AdjustmentParams},
    image::Image,
};

#[derive(Debug, Parser)]
#[command(name = "ddot", about = "Apply ddot filters to images")]
struct Cli {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long, default_value_t = 1.0)]
    gamma: f32,

    #[arg(long, default_value_t = 0.0, allow_negative_numbers = true)]
    blacks: f32,

    #[arg(long, default_value_t = 0.0, allow_negative_numbers = true)]
    whites: f32,

    #[arg(long, default_value_t = 0, allow_negative_numbers = true)]
    contrast: i32,

    #[arg(long, default_value_t = 1.0)]
    saturation: f32,

    #[arg(long, default_value_t = 0.0, allow_negative_numbers = true)]
    hue: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let output = cli
        .output
        .unwrap_or_else(|| default_output_path(&cli.input));

    let params = AdjustmentParams {
        gamma: cli.gamma,
        blacks: cli.blacks,
        whites: cli.whites,
        contrast: cli.contrast,
        saturation: cli.saturation,
        hue: cli.hue,
    };

    params.validate()?;

    let mut image = load_image(&cli.input)?;

    Adjustment.apply(&mut image, &params);

    save_png(&image, &output)?;

    println!("Saved {}", output.display());

    Ok(())
}

fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("output");

    let file_name = format!("{stem}_ddot.png");

    input.with_file_name(file_name)
}

fn load_image(path: &Path) -> Result<Image, Box<dyn Error>> {
    let rgba = image::open(path)?.to_rgba8();

    Ok(Image {
        width: rgba.width(),
        height: rgba.height(),
        pixels: rgba.into_raw(),
    })
}

fn save_png(image: &Image, path: &Path) -> Result<(), Box<dyn Error>> {
    image::save_buffer(
        path,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::default_output_path;
    use std::path::Path;

    #[test]
    fn builds_default_output_path_next_to_input() {
        assert_eq!(
            default_output_path(Path::new("images/photo.jpg")),
            Path::new("images/photo_ddot.png")
        );
    }

    #[test]
    fn builds_default_output_path_for_extensionless_input() {
        assert_eq!(
            default_output_path(Path::new("photo")),
            Path::new("photo_ddot.png")
        );
    }
}
