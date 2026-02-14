#import bevy_ui::{
    ui_node::{sd_inset_rounded_box, sd_rounded_box},
    ui_vertex_output::UiVertexOutput,
};

struct Animation {
    time: f32,
    start_translation: vec2<f32>,
    start_rotation: f32,
};

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var texture: texture_2d<f32>;
@group(1) @binding(2) var texture_sampler: sampler;
@group(1) @binding(3) var<uniform> border_color: vec4<f32>;
@group(1) @binding(4) var<uniform> animation: Animation;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let half_size = 0.5 * in.size;
    let p = in.uv * in.size - half_size;
    
    // Signed distance fields
    let external_distance = sd_rounded_box(p, in.size, in.border_radius);
    let internal_distance = sd_inset_rounded_box(
        p,
        in.size,
        in.border_radius,
        in.border_widths
    );

    // Antialiasing width
    let aa = fwidth(external_distance);

    // Masks for fill and border
    let outer_mask = 1.0 - smoothstep(0.0, aa, external_distance);
    let inner_mask = 1.0 - smoothstep(0.0, aa, internal_distance);
    let border_mask = outer_mask * (1.0 - inner_mask);

    // Cover mode for texture scaling (probably wrong, but looks aight)
    let tex_size = vec2<f32>(textureDimensions(texture));
    let node_size = in.size;

    let tex_aspect = tex_size.x / tex_size.y;
    let node_aspect = node_size.x / node_size.y;

    let scale = select(
        vec2(tex_aspect / node_aspect, 1.0),
        vec2(1.0, node_aspect / tex_aspect),
        tex_aspect <= node_aspect
    );

    let base_uv = (in.uv - 0.5) / scale + 0.5;

    // Pivot at texture center
    let centered = base_uv - vec2<f32>(0.5, 0.5);

    // Rotation
    let c = cos(animation.start_rotation);
    let s = sin(animation.start_rotation);
    let rotated = vec2<f32>(
        centered.x * c - centered.y * s,
        centered.x * s + centered.y * c
    );

    // Apply translation and scrolling animation
    let transformed_uv =
        rotated
        + vec2<f32>(0.5, 0.5)
        + animation.start_translation
        + vec2<f32>(animation.time * 0.03);

    // Wrap for tiling textures
    let uv = fract(transformed_uv);
    let tex = textureSample(texture, texture_sampler, uv);

    // Fill
    let fill_alpha = color.a * tex.a * inner_mask;
    let fill_rgb = tex.rgb * color.rgb;

    // Border
    let border_alpha = border_color.a * border_mask;
    let border_rgb = border_color.rgb;

    // Combine fill and border
    let out_alpha = fill_alpha + border_alpha;
    let out_rgb = fill_rgb * fill_alpha + border_rgb * border_alpha;

    return vec4(out_rgb, out_alpha);
}
