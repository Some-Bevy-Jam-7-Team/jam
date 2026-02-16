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
    // Original reference: https://www.shadertoy.com/view/fdS3Dy
    // Modified a bunch and added depth & motion vectors to the shader

    let dims = vec2<f32>(textureDimensions(screen_texture));
    let coords = vec2<i32>(in.uv * dims);
    let time = globals.time;
    let resolution = settings.resolution;

    // Fever
    let dist_from_center = length(in.uv - 0.5);
    let fever = clamp(settings.fever, 0.0, 1.0);

    // Motion and Warp Setup
    let motion = textureLoad(motion_texture, coords, 0).xy;
    let velocity = length(motion);
    let motion_warp = motion * 150.0 * fever;

    // Coordinate Scaling
    let open_close = sin(time * 0.5) * 0.5 + 0.5;
    let pulse_scale = mix(0.8, 1.5, open_close * fever);
    var coord = (in.uv - 0.5) * resolution.xy * pulse_scale + motion_warp;

    let x = coord.x;
    let y = coord.y;

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

    // Smooth internal structure before integer quantization
    let f2 = fract(r2 * p2);
    let f3 = fract(r3 * p3);
    let f4 = fract(r4 * p4);
    let f5 = fract(r5 * p5);

    // Blend smooth detail
    let detail = 0.25 * f2 + 0.25 * f3 + 0.25 * f4 + 0.25 * f5;

    // Keep original structural pattern
    let structure = fract(f32(e) * (0.000000075 + sin(time * 0.05) * 0.000000025));

    // Blend structure and smooth shading
    var raw = mix(structure, detail, 0.2);

    // Add soft S-curve shaping
    let s = raw * raw * (3.0 - 2.0 * raw);
    let shaped = pow(s, 2.5);

    // Color Palette
    let color1 = vec3<f32>(64.0, 27.0, 18.0) / 255.0;
    let color2 = vec3<f32>(255.0, 97.0, 117.0) / 255.0;
    let color3 = vec3<f32>(255.0, 0.0, 116.0) / 255.0;

    var kaleidoscope = gradient3(fract(shaped + time * 0.1), color1, color2, color3);

    let depth_val = textureLoad(depth_texture, coords, 0);
    let near_boost = smoothstep(0.0, 1.0, depth_val);

    // Glow
    let glow = pow(smoothstep(0.7, 1.0, s), 3.0) * 1.5 * fever * (1.0 + near_boost * 4.0);
    kaleidoscope += (color3 * glow);

    // Vignette
    let noise = gradient_noise(in.uv * dims);
    let dithered_dist = dist_from_center + (noise - 0.5) * 0.015;
    let vignette_mask = smoothstep(0.2, 0.7, dithered_dist) * fever;

    // Iris
    let iris_threshold = mix(1.5, 0.1, open_close * fever);
    let iris_reveal = smoothstep(iris_threshold, iris_threshold + 0.8, dithered_dist);
    let motion_mask = smoothstep(0.0, 0.02, velocity) * 0.02 * fever;

    // Depth
    let depth_mask = (smoothstep(0.01, 0.2, depth_val) * smoothstep(0.99, 0.8, depth_val)) * 0.2;

    // Combine
    let mask = clamp(vignette_mask + iris_reveal + motion_mask + depth_mask, 0.0, 1.0);
    let protection = smoothstep(0.99, 0.8, depth_val);
    let final_mix_val = mask * protection * fever * settings.intensity * (1.0 + near_boost * 2.0);

    let base_color = textureSample(screen_texture, texture_sampler, in.uv).rgb;
    return vec4<f32>(mix(base_color, kaleidoscope, clamp(final_mix_val, 0.0, 1.0)), 1.0);
}

fn gradient3(t: f32, c0: vec3<f32>, c1: vec3<f32>, c2: vec3<f32>) -> vec3<f32> {
    let mid = 0.5;
    if (t < mid) { return mix(c0, c1, t / mid); }
    else { return mix(c1, c2, (t - mid) / mid); }
}

fn gradient_noise(uv: vec2<f32>) -> f32 {
    let magic = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(uv, magic.xy)));
}
