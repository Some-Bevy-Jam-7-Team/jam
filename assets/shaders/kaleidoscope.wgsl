#import bevy_ui::ui_vertex_output::UiVertexOutput;

struct KaleidoscopeSettings {
    resolution: vec2<f32>,
    time: f32,
    darkness: f32,          // 0 = bright, 1 = very dark
    contrast: f32,          // 1–6 typical
    highlight_strength: f32,// 0–2
    glow_strength: f32,     // 0–3
    color_low: vec3<f32>,   // shadow tone
    color_mid: vec3<f32>,   // mid tone
    color_high: vec3<f32>,  // highlight tone
};

@group(1) @binding(0) var<uniform> settings: KaleidoscopeSettings;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Original reference: https://www.shadertoy.com/view/fdS3Dy
    // Modified a bunch and added lots of parameters

    let time = settings.time*8;
    let resolution = settings.resolution;

    // Convert UV to pixel coordinates
    let frag_coord = in.uv * resolution.xy;

    // Center coordinates
    var coord = frag_coord - (resolution.xy * 0.5);
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
    let raw = mix(structure, detail, 0.2);

    // Add soft S-curve shaping
    let s = raw * raw * (3.0 - 2.0 * raw);
    let shaped = pow(s, settings.contrast);

    // Apply darkness
    let darkened = shaped * (1.0 - settings.darkness);

    // Highlight mask
    let highlight = smoothstep(0.75, 1.0, raw) * settings.highlight_strength;

    // Combine darkened base and highlights, then clamp to [0, 1]
    let t = clamp(darkened + highlight, 0.0, 1.0);

    // Gradient color mapping
    var color = gradient3(
        t,
        settings.color_low,
        settings.color_mid,
        settings.color_high
    );

    // Optional glow boost
    let glow = pow(highlight, 2.0) * settings.glow_strength;

    // Apply vignetting based on distance from center
    let dist = length(coord) / (length(resolution) * 0.5);
    let vignette = smoothstep(0.7, 1.0, dist);
    color *= 1.0 - vignette;

    return vec4(color + glow, 1.0);
}

// Helper function for 3-color gradient
fn gradient3(
    t: f32,
    c0: vec3<f32>,
    c1: vec3<f32>,
    c2: vec3<f32>
) -> vec3<f32> {

    let mid = 0.5;

    if (t < mid) {
        return mix(c0, c1, t / mid);
    } else {
        return mix(c1, c2, (t - mid) / mid);
    }
}
