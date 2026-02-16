#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::globals::Globals

struct FeverPostProcessSettings {
    resolution: vec2<f32>,
    intensity: f32,
    fever: f32,
    damage_threshold: f32,
    damage_indicator: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: FeverPostProcessSettings;

@group(0) @binding(3) var<uniform> globals: Globals;
@group(0) @binding(4) var depth_texture: texture_depth_2d;
@group(0) @binding(5) var motion_texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Reference: https://www.shadertoy=.com/view/fdS3Dy

    // Filtered heavy textures, need to use textureLoad for depth/motion raw data
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let coords = vec2<i32>(in.uv * dims);

    let time = globals.time;
    let resolution = settings.resolution;
    let fever = clamp(settings.fever + settings.damage_indicator, 0.0, 1.0);

    // Motion setup
    let motion = textureLoad(motion_texture, coords, 0).xy;
    let motion_warp = motion * 10.0 * settings.fever;

    // Convert UV to pixel coordinates
    let frag_coord = in.uv * resolution.xy;

    // Center coordinates
    var coord = frag_coord - (resolution.xy * 0.5) + motion_warp;
    let x = coord.x;
    let y = coord.y;

    // Time modulation with speed
    let velocity = length(motion);
    let motion_flash = velocity * 100.0 * fever;
    let j_time = glsl_mod(4.0 * sin(0.5 * (time + motion_flash)), 261.8) + 4.0;
    coord *= pow(1.1, j_time);

    // Radial distances
    let eps = 0.001;
    let r2 = abs((x * x + y * y) / max(abs(x), eps));
    let r3 = abs((x * x + y * y) / max(abs(y), eps));
    let r4 = abs((x * x + y * y) / max(abs(x - y), eps)) * sqrt(2.0);
    let r5 = abs((x * x + y * y) / max(abs(x + y), eps)) * sqrt(2.0);

    // Pattern scaling
    let p2 = pow(sin(time * 0.05) * sin(time * 0.05) * 16.0, 6.0 - ceil(log2(r2) / 4.0));
    let p3 = pow(cos(time * 0.02) * cos(time * 0.02) * 16.0, 6.0 - ceil(log2(r3) / 4.0));
    let p4 = pow(16.0, 6.0 - ceil(log2(r4) / 4.0));
    let p5 = pow(16.0, 6.0 - ceil(log2(r5) / 4.0));

    // Integer patterns
    let a = i32(floor(r2 * p2));
    let b = i32(floor(r3 * p3));
    let c = i32(floor(r4 * p4));
    let d = i32(floor(r5 * p5));

    // Combine patterns with XOR
    let e = (a | b) ^ (c | d);

    // Color mapping
    let value = fract(f32(e) * 0.000003);
    let kaleidoscope = hsv2rgb(vec3<f32>(value + time * 0.6, 0.6, 0.6));

    // Tint
    let tint = vec3<f32>(1.0, 0.0, 0.0);
    let pattern = mix(kaleidoscope, tint, 0.6);

    // Vignette
    let dist = distance(in.uv, vec2<f32>(0.5));
    let vignette = smoothstep(0.2, 0.8, dist);

    // Depth
    let depth = textureLoad(depth_texture, coords, 0);
    let depth_mask = smoothstep(0.0, 0.1, depth);

    // Motion
    let motion_mask = smoothstep(0.0, 0.02, velocity) * 0.5 * fever;

    // Combine
    let mask = max(vignette, depth_mask);
    let base_color = textureSample(screen_texture, texture_sampler, in.uv);
    let mix_factor = clamp((fever + motion_mask) * mask * settings.intensity, 0.0, 1.0);

    return vec4<f32>(mix(base_color.rgb, pattern, mix_factor), 1.0);
}

fn glsl_mod(a: f32, b: f32) -> f32 {
    return a - b * floor(a / b);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0/3.0, 1.0/3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}
