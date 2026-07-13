@group(0) @binding(0) var<storage, read> input_pixels: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_pixels: array<u32>;

struct Params {
    gamma: f32,
    blacks: f32,
    whites: f32,
    contrast: i32,
    saturation: f32,
    hue: f32,
}
@group(0) @binding(3) var<uniform> params: Params;

struct Dimensions {
    width: u32,
    height: u32,
}
@group(0) @binding(2) var<uniform> dims: Dimensions;

fn smoothstep_local(x: f32) -> f32 {
    let xc = clamp(x, 0.0, 1.0);
    return xc * xc * (3.0 - 2.0 * xc);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }

    let idx = id.y * dims.width + id.x;
    let packed_color = input_pixels[idx];

    // Unpack pixel into vec4<f32> containing RGBA values in 0.0..1.0
    var color = unpack4x8unorm(packed_color);

    var r = color.r;
    var g = color.g;
    var b = color.b;
    let a = color.a;

    // 1. Gamma
    if (params.gamma != 1.0) {
        let gamma_exp = 1.0 / params.gamma;
        r = pow(r, gamma_exp);
        g = pow(g, gamma_exp);
        b = pow(b, gamma_exp);
    }

    // 2. Blacks
    if (params.blacks != 0.0) {
        let shadow_r = smoothstep_local(1.0 - r);
        let shadow_g = smoothstep_local(1.0 - g);
        let shadow_b = smoothstep_local(1.0 - b);

        r += params.blacks * shadow_r;
        g += params.blacks * shadow_g;
        b += params.blacks * shadow_b;
    }

    // 3. Whites
    if (params.whites != 0.0) {
        let highlight_r = smoothstep_local(r);
        let highlight_g = smoothstep_local(g);
        let highlight_b = smoothstep_local(b);

        r += params.whites * highlight_r;
        g += params.whites * highlight_g;
        b += params.whites * highlight_b;
    }

    // 4. Contrast
    if (params.contrast != 0) {
        let contrast_val = clamp(f32(params.contrast), -255.0, 258.0);
        let contrast_factor = (259.0 * (contrast_val + 255.0)) / (255.0 * (259.0 - contrast_val));

        r = (r - 0.5) * contrast_factor + 0.5;
        g = (g - 0.5) * contrast_factor + 0.5;
        b = (b - 0.5) * contrast_factor + 0.5;
    }

    // Clamp after LUT adjustments
    r = clamp(r, 0.0, 1.0);
    g = clamp(g, 0.0, 1.0);
    b = clamp(b, 0.0, 1.0);

    // 5. Saturation
    if (params.saturation != 1.0) {
        let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        r = gray + (r - gray) * params.saturation;
        g = gray + (g - gray) * params.saturation;
        b = gray + (b - gray) * params.saturation;

        r = clamp(r, 0.0, 1.0);
        g = clamp(g, 0.0, 1.0);
        b = clamp(b, 0.0, 1.0);
    }

    // 6. Hue
    if (params.hue != 0.0) {
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let i = 0.596 * r - 0.274 * g - 0.322 * b;
        let q = 0.211 * r - 0.523 * g + 0.312 * b;

        let cos_h = cos(params.hue);
        let sin_h = sin(params.hue);

        let i2 = i * cos_h - q * sin_h;
        let q2 = i * sin_h + q * cos_h;

        r = y + 0.956 * i2 + 0.621 * q2;
        g = y - 0.272 * i2 - 0.647 * q2;
        b = y - 1.106 * i2 + 1.703 * q2;

        r = clamp(r, 0.0, 1.0);
        g = clamp(g, 0.0, 1.0);
        b = clamp(b, 0.0, 1.0);
    }

    output_pixels[idx] = pack4x8unorm(vec4<f32>(r, g, b, a));
}
