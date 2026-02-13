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

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Reference: https://www.shadertoy=.com/view/fdS3Dy

    let color = textureSample(screen_texture, texture_sampler, in.uv);
    let resolution = settings.resolution;
    let time = globals.time;

    // Convert UV to pixel coordinates
    let frag_coord = in.uv * resolution.xy;

    // Center coordinates
    var coord = frag_coord - (resolution.xy * 0.5);
    let x = coord.x;
    let y = coord.y;

    // Time modulation
    let j_time = glsl_mod(4.0 * sin(0.5 * time), 261.8) + 4.0;
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
    let kaleidoscope= hsv2rgb(vec3<f32>(value + time * 0.6, 0.6, 0.6));

    // Fever tint
    let fever_tint = vec3<f32>(1.0, 0.0, 0.0);
    let pattern = mix(kaleidoscope, fever_tint, 0.6);

    // Vignette
    let dist = distance(in.uv, vec2<f32>(0.5));
    let vignette= smoothstep(0.2, 0.8, dist);

    // Combine Fever and Damage into a single intensity value,
    // tweaking the dmg indicator manually for now
    let intensity = (settings.fever + (settings.damage_indicator * 0.5)) * settings.intensity;
    let mix_factor = clamp(intensity * vignette, 0.0, 1.0);

    return vec4<f32>(mix(color.rgb, pattern, mix_factor), 1.0);
}

fn glsl_mod(a: f32, b: f32) -> f32 {
    return a - b * floor(a / b);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0/3.0, 1.0/3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}
