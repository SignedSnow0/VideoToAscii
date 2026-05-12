@group(0) @binding(0) var output_texture: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(1) var input_video: texture_2d<f32>;
@group(0) @binding(2) var video_sampler: sampler;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = textureDimensions(output_texture);
    if global_id.x >= screen_size.x || global_id.y >= screen_size.y {
        return;
    }

    let uv = vec2<f32>(global_id.xy) / vec2<f32>(screen_size);
    var color = textureSampleLevel(input_video, video_sampler, uv, 0.0);
    
    // Grayscale conversion
    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let gray_color = vec4<f32>(gray, gray, gray, color.a);

    textureStore(output_texture, global_id.xy, gray_color);
}

