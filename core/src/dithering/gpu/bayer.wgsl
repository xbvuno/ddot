@group(0) @binding(0) var<storage, read> input_pixels: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_pixels: array<u32>;

struct Dimensions {
    width: u32,
    height: u32,
}
@group(0) @binding(2) var<uniform> dims: Dimensions;

struct Params {
    amount: f32,
    matrixScale: f32,
    colorSpace: u32, // 0 = RGB, 1 = Lab
}
@group(0) @binding(3) var<uniform> params: Params;

@group(0) @binding(4) var<storage, read> palette: array<u32>;

const BAYER_MATRIX = array<f32, 16>(
    0.0, 8.0, 2.0, 10.0,
    12.0, 4.0, 14.0, 6.0,
    3.0, 11.0, 1.0, 9.0,
    15.0, 7.0, 13.0, 5.0
);

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(
        c / 12.92,
        pow((c + 0.055) / 1.055, vec3<f32>(2.4)),
        c > vec3<f32>(0.04045)
    );
}

fn lab_f(t: f32) -> f32 {
    let epsilon = 0.00885645167; // 216.0 / 24389.0
    let kappa = 903.296296296;   // 24389.0 / 27.0
    return select(
        (kappa * t + 16.0) / 116.0,
        pow(t, 1.0 / 3.0),
        t > epsilon
    );
}

fn rgb_to_lab(rgb: vec3<f32>) -> vec3<f32> {
    let lin = srgb_to_linear(rgb);
    
    let REF_X = 0.95047;
    let REF_Y = 1.00000;
    let REF_Z = 1.08883;
    
    let x = (lin.r * 0.4124564 + lin.g * 0.3575761 + lin.b * 0.1804375) / REF_X;
    let y = (lin.r * 0.2126729 + lin.g * 0.7151522 + lin.b * 0.0721750) / REF_Y;
    let z = (lin.r * 0.0193339 + lin.g * 0.1191920 + lin.b * 0.9503041) / REF_Z;
    
    let fx = lab_f(x);
    let fy = lab_f(y);
    let fz = lab_f(z);
    
    let L = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    
    return vec3<f32>(L, a, b);
}

fn find_closest_color(rgb: vec3<f32>) -> vec4<f32> {
    let num_colors = arrayLength(&palette);
    if (num_colors == 0u) {
        return vec4<f32>(rgb, 1.0);
    }
    
    var min_dist_sq = 1e20;
    var closest_color = unpack4x8unorm(palette[0]);
    
    if (params.colorSpace == 1u) {
        let query_lab = rgb_to_lab(rgb);
        for (var i = 0u; i < num_colors; i = i + 1u) {
            let p_color = unpack4x8unorm(palette[i]);
            let p_lab = rgb_to_lab(p_color.rgb);
            let diff = query_lab - p_lab;
            let dist_sq = dot(diff, diff);
            if (dist_sq < min_dist_sq) {
                min_dist_sq = dist_sq;
                closest_color = p_color;
            }
        }
    } else {
        for (var i = 0u; i < num_colors; i = i + 1u) {
            let p_color = unpack4x8unorm(palette[i]);
            let diff = rgb - p_color.rgb;
            let dist_sq = dot(diff, diff);
            if (dist_sq < min_dist_sq) {
                min_dist_sq = dist_sq;
                closest_color = p_color;
            }
        }
    }
    
    return closest_color;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }

    let idx = id.y * dims.width + id.x;
    let packed_color = input_pixels[idx];
    let color = unpack4x8unorm(packed_color);
    
    if (color.a == 0.0) {
        output_pixels[idx] = packed_color;
        return;
    }

    var rgb = color.rgb;
    
    let scale = max(1u, u32(round(params.matrixScale)));
    let bx = (id.x / scale) & 3u;
    let by = (id.y / scale) & 3u;
    let val = BAYER_MATRIX[by * 4u + bx];
    let threshold = (val / 16.0 - 0.5) * params.amount;
    
    // Apply Bayer matrix dither offset symmetrically to RGB
    rgb = clamp(rgb + vec3<f32>(threshold, threshold * 0.9, threshold * 0.8), vec3<f32>(0.0), vec3<f32>(1.0));
    
    let closest = find_closest_color(rgb);
    
    // Binary alpha threshold
    let final_alpha = select(0.0, 1.0, color.a >= 0.5);
    output_pixels[idx] = pack4x8unorm(vec4<f32>(closest.rgb, final_alpha));
}
