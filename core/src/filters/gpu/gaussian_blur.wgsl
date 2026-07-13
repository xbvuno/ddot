@group(0) @binding(0) var<storage, read> input_pixels: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_pixels: array<u32>;

struct Dimensions {
    width: u32,
    height: u32,
}
@group(0) @binding(2) var<uniform> dims: Dimensions;

@group(0) @binding(3) var<storage, read> kernel_weights: array<f32>;

struct Params {
    direction: u32, // 0 = horizontal, 1 = vertical
    radius: i32,
}
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.width || id.y >= dims.height) {
        return;
    }

    let idx = id.y * dims.width + id.x;

    var r_sum = 0.0;
    var g_sum = 0.0;
    var b_sum = 0.0;
    var a_sum = 0.0;
    var w_sum = 0.0;

    for (var i = -params.radius; i <= params.radius; i = i + 1) {
        var px = i32(id.x);
        var py = i32(id.y);

        if (params.direction == 0u) {
            px = clamp(px + i, 0, i32(dims.width) - 1);
        } else {
            py = clamp(py + i, 0, i32(dims.height) - 1);
        }

        let neighbor_idx = u32(py) * dims.width + u32(px);
        let color = unpack4x8unorm(input_pixels[neighbor_idx]);
        let weight = kernel_weights[i + params.radius];

        r_sum += color.r * weight;
        g_sum += color.g * weight;
        b_sum += color.b * weight;
        a_sum += color.a * weight;
        w_sum += weight;
    }

    let blurred_rgba = vec4<f32>(r_sum, g_sum, b_sum, a_sum) / w_sum;
    output_pixels[idx] = pack4x8unorm(blurred_rgba);
}
