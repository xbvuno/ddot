@group(0) @binding(0) var<storage, read> input_pixels: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_pixels: array<u32>;

struct Dimensions {
    width: u32,
    height: u32,
}
@group(0) @binding(2) var<uniform> dims: Dimensions;

struct Params {
    coverage: f32,
    intensity: f32,
    saturation: f32,
    phase: f32,
}
@group(0) @binding(3) var<uniform> params: Params;

fn hash2D(x: f32, y: f32, seed: f32) -> f32 {
    let t = sin((x + seed * 0.17) * 12.9898 + (y + seed * 1.31) * 78.233) * 43758.5453;
    return t - floor(t);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }

    let idx = id.y * dims.width + id.x;
    let packed_color = input_pixels[idx];
    let color = unpack4x8unorm(packed_color);
    var rgb = color.rgb;
    
    let seed = params.phase + 1.0;
    
    // Check coverage mask
    if (hash2D(f32(id.x), f32(id.y), seed) < params.coverage) {
        let mono = hash2D(f32(id.x), f32(id.y), seed + 1.0) * 2.0 - 1.0;
        let nr = hash2D(f32(id.x), f32(id.y), seed + 2.0) * 2.0 - 1.0;
        let ng = hash2D(f32(id.x), f32(id.y), seed + 3.0) * 2.0 - 1.0;
        let nb = hash2D(f32(id.x), f32(id.y), seed + 4.0) * 2.0 - 1.0;
        
        let sat = params.saturation;
        let noise_r = (mono * (1.0 - sat) + nr * sat) * params.intensity;
        let noise_g = (mono * (1.0 - sat) + ng * sat) * params.intensity;
        let noise_b = (mono * (1.0 - sat) + nb * sat) * params.intensity;
        
        rgb.r = clamp(rgb.r + noise_r, 0.0, 1.0);
        rgb.g = clamp(rgb.g + noise_g, 0.0, 1.0);
        rgb.b = clamp(rgb.b + noise_b, 0.0, 1.0);
    }
    
    output_pixels[idx] = pack4x8unorm(vec4<f32>(rgb, color.a));
}
