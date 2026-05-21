@group(0) @binding(0) var screen_tex: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var char_tex: texture_2d<f32>;
@group(0) @binding(3) var char_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((in_idx << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(in_idx & 2u) * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, -y * 0.5 + 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let num_chars = 32.0;
    let chars_per_row = 8.0;

    let atlas_size = vec2<f32>(textureDimensions(char_tex));
    let atlas_cols_rows = vec2<f32>(chars_per_row, ceil(num_chars / chars_per_row));
    let atlas_cell_size = atlas_size / atlas_cols_rows;
    
    let cell_size = atlas_cell_size;
    
    let block = floor(in.clip_position.xy / cell_size);
    let local_uv = (in.clip_position.xy % cell_size) / cell_size;

    let uv_per_pixel = vec2<f32>(dpdx(in.uv.x), dpdy(in.uv.y));
    let block_center_px = block * cell_size + cell_size * 0.5;
    let video_uv = in.uv + (block_center_px - in.clip_position.xy) * uv_per_pixel;
    
    let color = textureSample(screen_tex, screen_sampler, video_uv).rgb;
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));

    let char_index = clamp(floor(luminance * num_chars), 0.0, num_chars - 1.0);
    let char_x = char_index % chars_per_row;
    let char_y = floor(char_index / chars_per_row);
    
    let offset_uv = (vec2<f32>(char_x, char_y) * atlas_cell_size) / atlas_size;
    let final_uv = offset_uv + (local_uv * (atlas_cell_size / atlas_size));
    
    let char_mask = textureSample(char_tex, char_sampler, final_uv).r;

    return vec4<f32>(color * char_mask, 1.0);
}