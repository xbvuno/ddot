@group(0) @binding(0) var<storage, read> input_pixels: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_pixels: array<u32>;

struct Params {
    gamma: f32,
    blacks: f32,
    whites: f32,
    contrast: f32,
    saturation: f32,
    hue: f32,
}
@group(0) @binding(3) var<uniform> params: Params;

struct Dimensions {
    width: u32,
    height: u32,
}
@group(0) @binding(2) var<uniform> dims: Dimensions;

// RGB to YIQ transformation matrix (column-major representation in WGSL)
const kRGB_to_YIQ = mat3x3<f32>(
    vec3<f32>(0.299, 0.596, 0.211),   // Column 0
    vec3<f32>(0.587, -0.274, -0.523), // Column 1
    vec3<f32>(0.114, -0.322, 0.312)   // Column 2
);

// YIQ to RGB transformation matrix (column-major representation in WGSL)
const kYIQ_to_RGB = mat3x3<f32>(
    vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(0.956, -0.272, -1.106),
    vec3<f32>(0.621, -0.647, 1.703)
);

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }

    let idx = id.y * dims.width + id.x;
    let packed_color = input_pixels[idx];

    // Unpack pixel into vec4<f32> containing RGBA values in 0.0..1.0
    let color = unpack4x8unorm(packed_color);
    var rgb = color.rgb;
    let a = color.a;

    // 1. Gamma
    if (params.gamma != 1.0) {
        rgb = pow(rgb, vec3<f32>(1.0 / params.gamma));
    }

    // 2. Blacks
    if (params.blacks != 0.0) {
        rgb += params.blacks * smoothstep(vec3<f32>(0.0), vec3<f32>(1.0), vec3<f32>(1.0) - rgb);
    }

    // 3. Whites
    if (params.whites != 0.0) {
        rgb += params.whites * smoothstep(vec3<f32>(0.0), vec3<f32>(1.0), rgb);
    }

    // 4. Contrast
    if (params.contrast != 0.0) {
        let contrast_factor = (259.0 * (params.contrast + 255.0)) / (255.0 * (259.0 - params.contrast));
        rgb = (rgb - 0.5) * contrast_factor + 0.5;
    }

    // Clamp after levels adjustments
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));

    // 5. Saturation
    if (params.saturation != 1.0) {
        let gray = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        rgb = gray + (rgb - gray) * params.saturation;
    }

    // 6. Hue
    if (params.hue != 0.0) {
        let yiq = kRGB_to_YIQ * rgb;

        let cos_h = cos(params.hue);
        let sin_h = sin(params.hue);

        let i_rot = yiq.y * cos_h - yiq.z * sin_h;
        let q_rot = yiq.y * sin_h + yiq.z * cos_h;

        rgb = kYIQ_to_RGB * vec3<f32>(yiq.x, i_rot, q_rot);
    }

    // Final Clamp
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));

    output_pixels[idx] = pack4x8unorm(vec4<f32>(rgb, a));
}
