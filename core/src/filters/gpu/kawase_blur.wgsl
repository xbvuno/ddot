@group(0) @binding(0) var<storage, read> input_pixels: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_pixels: array<u32>;

struct Dimensions {
    width: u32,
    height: u32,
}
@group(0) @binding(2) var<uniform> dims: Dimensions;

struct Params {
    blurStrength: f32,
    edgeStrength: f32,
    passes: f32,
}
@group(0) @binding(3) var<uniform> params: Params;

fn sample_bilinear(px: f32, py: f32) -> vec4<f32> {
    let w = f32(dims.width);
    let h = f32(dims.height);
    let x = clamp(px, 0.0, w - 1.0);
    let y = clamp(py, 0.0, h - 1.0);
    
    let x0 = u32(floor(x));
    let y0 = u32(floor(y));
    let x1 = min(x0 + 1u, dims.width - 1u);
    let y1 = min(y0 + 1u, dims.height - 1u);
    
    let tx = x - floor(x);
    let ty = y - floor(y);
    
    let c00 = unpack4x8unorm(input_pixels[y0 * dims.width + x0]);
    let c10 = unpack4x8unorm(input_pixels[y0 * dims.width + x1]);
    let c01 = unpack4x8unorm(input_pixels[y1 * dims.width + x0]);
    let c11 = unpack4x8unorm(input_pixels[y1 * dims.width + x1]);
    
    let c0 = mix(c00, c10, tx);
    let c1 = mix(c01, c11, tx);
    return mix(c0, c1, ty);
}

fn edge_weight(a: vec3<f32>, b: vec3<f32>, strength: f32) -> f32 {
    let diff = length(a - b);
    return exp(-diff * strength);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }

    let idx = id.y * dims.width + id.x;
    let packed_center = input_pixels[idx];
    let center = unpack4x8unorm(packed_center);
    let center_rgb = center.rgb;
    
    let offset = params.blurStrength;
    let edge_strength = params.edgeStrength;
    
    let x_f = f32(id.x);
    let y_f = f32(id.y);
    
    let c1 = sample_bilinear(x_f + offset, y_f + offset);
    let c2 = sample_bilinear(x_f - offset, y_f + offset);
    let c3 = sample_bilinear(x_f + offset, y_f - offset);
    let c4 = sample_bilinear(x_f - offset, y_f - offset);
    
    let w1 = edge_weight(center_rgb, c1.rgb, edge_strength);
    let w2 = edge_weight(center_rgb, c2.rgb, edge_strength);
    let w3 = edge_weight(center_rgb, c3.rgb, edge_strength);
    let w4 = edge_weight(center_rgb, c4.rgb, edge_strength);
    
    let sum_rgb = center_rgb + c1.rgb * w1 + c2.rgb * w2 + c3.rgb * w3 + c4.rgb * w4;
    let total_w = 1.0 + w1 + w2 + w3 + w4;
    
    let blurred_rgba = vec4<f32>(sum_rgb / total_w, center.a);
    output_pixels[idx] = pack4x8unorm(blurred_rgba);
}
