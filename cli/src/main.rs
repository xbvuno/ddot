use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use ddot_core::{
    filter::{ParamType, FilterParams},
    filters,
    image::Image,
};
use serde::Deserialize;

mod gpu;

#[derive(Debug, Parser)]
#[command(name = "ddot", about = "Apply ddot filters to images", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Apply a filter pipeline to an image
    Apply {
        /// Path to the input image
        input: PathBuf,

        /// Path to the output image (optional, defaults to <input>_ddot.png)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// JSON pipeline inline string or path to a JSON pipeline file
        #[arg(short, long)]
        pipeline: String,

        /// Backend to use: auto, cpu, or gpu (default: auto)
        #[arg(short, long, default_value = "auto")]
        backend: String,
    },

    /// List all available filters and their parameter metadata
    List,

    /// Generate a JSON template for a specific filter
    Schema {
        /// Name of the filter (e.g. 'adjustment')
        filter_name: String,
    },
}

#[derive(Debug, Deserialize)]
struct PipelineStep {
    name: String,
    settings: serde_json::Value,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Apply {
            input,
            output,
            pipeline,
            backend,
        } => {
            handle_apply(input, output, pipeline, backend)?;
        }
        Commands::List => {
            handle_list()?;
        }
        Commands::Schema { filter_name } => {
            handle_schema(filter_name)?;
        }
    }

    Ok(())
}

fn handle_apply(
    input: PathBuf,
    output: Option<PathBuf>,
    pipeline: String,
    backend: String,
) -> Result<(), Box<dyn Error>> {
    // 1. Read input pipeline string or file
    let pipeline_json = if Path::new(&pipeline).is_file() {
        fs::read_to_string(&pipeline)?
    } else {
        pipeline
    };

    // 2. Parse pipeline steps
    let steps: Vec<PipelineStep> = match serde_json::from_str(&pipeline_json) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("Failed to parse pipeline JSON: {}", e).into());
        }
    };

    // 3. Load image
    let mut image = load_image(&input)?;

    // 4. Apply each step
    for (i, step) in steps.into_iter().enumerate() {
        if filters::filter_definition(&step.name).is_none() {
            return Err(format!(
                "Error in pipeline step {}: unknown filter '{}'",
                i + 1,
                step.name
            )
            .into());
        }

        if let Err(e) = pollster::block_on(apply_filter_cli(&mut image, &step.name, step.settings, &backend)) {
            return Err(format!(
                "Error in pipeline step {} ('{}'): {}",
                i + 1,
                step.name,
                e
            )
            .into());
        }
    }

    // 5. Determine output path
    let output_path = output.unwrap_or_else(|| default_output_path(&input));

    // 6. Save image
    save_png(&image, &output_path)?;

    println!("Saved {}", output_path.display());

    Ok(())
}

fn handle_list() -> Result<(), Box<dyn Error>> {
    println!("Available filters:");
    for filter in filters::FILTERS {
        println!("\n  * Filter: {}", filter.name);
        if filter.params.is_empty() {
            println!("    No parameters.");
        } else {
            println!("    Parameters:");
            for param in filter.params {
                let type_str = match param.kind {
                    ParamType::Int => "integer",
                    ParamType::Float => "float",
                };

                let range_str = match (param.min, param.max) {
                    (Some(min), Some(max)) => format!("range: {}..{}", min, max),
                    _ => "range: any".to_string(),
                };

                let default_val = if param.kind == ParamType::Int {
                    (param.default as i64).to_string()
                } else {
                    param.default.to_string()
                };

                println!(
                    "      - {} ({}, default: {}, {})",
                    param.name, type_str, default_val, range_str
                );
            }
        }
    }
    Ok(())
}

fn handle_schema(filter_name: String) -> Result<(), Box<dyn Error>> {
    let definition = match filters::filter_definition(&filter_name) {
        Some(d) => d,
        None => {
            eprintln!("Error: unknown filter '{}'", filter_name);
            std::process::exit(1);
        }
    };

    let mut settings = serde_json::Map::new();
    for param in definition.params {
        let value = match param.kind {
            ParamType::Int => {
                serde_json::Value::Number(serde_json::Number::from(param.default as i64))
            }
            ParamType::Float => {
                if let Some(num) = serde_json::Number::from_f64(param.default as f64) {
                    serde_json::Value::Number(num)
                } else {
                    serde_json::Value::Null
                }
            }
        };
        settings.insert(param.name.to_string(), value);
    }

    let step = serde_json::json!({
        "name": definition.name,
        "settings": settings,
    });

    let pretty = serde_json::to_string_pretty(&step)?;
    println!("{}", pretty);

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

async fn apply_filter_cli(
    image: &mut Image,
    name: &str,
    settings: serde_json::Value,
    backend: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use ddot_core::filter::Filter;
    use ddot_core::filters::{Adjustment, AdjustmentParams, Noise, NoiseParams};

    match name {
        Adjustment::NAME => {
            let params: AdjustmentParams = serde_json::from_value(settings)?;
            params.validate()?;

            match backend {
                "cpu" => {
                    Adjustment.apply(image, &params);
                    Ok(())
                }
                "gpu" => {
                    if let Some(shader) = Adjustment.gpu_shader() {
                        let params_bytes = params.to_bytes();
                        gpu::run_gpu(shader, image, &params_bytes).await?;
                        Ok(())
                    } else {
                        eprintln!("Warning: Filter '{}' does not support GPU backend, falling back to CPU", name);
                        Adjustment.apply(image, &params);
                        Ok(())
                    }
                }
                _ => { // "auto"
                    if let Some(shader) = Adjustment.gpu_shader() {
                        let params_bytes = params.to_bytes();
                        match gpu::run_gpu(shader, image, &params_bytes).await {
                            Ok(()) => Ok(()),
                            Err(ddot_core::filter::FilterError::GpuUnavailable) => {
                                // Silent fallback
                                Adjustment.apply(image, &params);
                                Ok(())
                            }
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        Adjustment.apply(image, &params);
                        Ok(())
                    }
                }
            }
        }
        Noise::NAME => {
            let params: NoiseParams = serde_json::from_value(settings)?;
            params.validate()?;

            // Noise is CPU-only
            if backend == "gpu" {
                eprintln!("Warning: Filter '{}' does not support GPU backend, falling back to CPU", name);
            }
            Noise.apply(image, &params);
            Ok(())
        }
        _ => Err(format!("Unknown filter '{}'", name).into()),
    }
}
