#import bevy_render::{
    view::View,
    globals::Globals,
}
#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var<uniform> globals: Globals;

struct EyeMaterial {
    iris_color: vec4<f32>,
    sclera_color: vec4<f32>,
    cursor_pos: vec2<f32>,     // normalized 0-1 screen position
    pupil_dilation: f32,       // 0.0 = normal, 1.0 = fully dilated
    blink_state: f32,          // 0.0 = open, 1.0 = closed
}

@group(1) @binding(0)
var<uniform> material: EyeMaterial;

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;

    // Center UV to -1..1 range
    let uv = (in.uv - 0.5) * 2.0;
    let aspect = in.size.x / max(in.size.y, 0.001);

    // Aspect-corrected UV
    var p = uv;
    p.x *= aspect;

    // Organic wobble - distort the coordinate space
    let wobble_freq = 3.0;
    let wobble_amt = 0.03 + sin(t * 0.7) * 0.01;
    p.x += sin(p.y * wobble_freq + t * 1.3) * wobble_amt;
    p.y += cos(p.x * wobble_freq + t * 0.9) * wobble_amt;

    // Eye tracking - offset iris based on cursor position, with twitch
    let twitch = vec2<f32>(
        sin(t * 7.3) * sin(t * 13.1) * 0.02,
        cos(t * 9.7) * sin(t * 11.3) * 0.015,
    );
    let look_dir = (material.cursor_pos - 0.5) * 0.3 + twitch;
    let iris_center = look_dir;

    // Distance from iris center
    let d = length(p - iris_center);

    // Eyeball shape - elliptical with irregular edge
    let angle_for_shape = atan2(p.y, p.x);
    let edge_noise = noise(vec2<f32>(angle_for_shape * 2.0, t * 0.2)) * 0.06;
    let eye_shape = length(vec2<f32>(p.x * 0.8, p.y)) + edge_noise;

    // Eyelid blink - squish the eye vertically, with ragged edges
    let blink = material.blink_state;
    let lid_noise = noise(vec2<f32>(p.x * 5.0, t * 0.3)) * 0.08;
    let eyelid_top = mix(0.9, 0.0, blink) + lid_noise;
    let eyelid_bottom = mix(-0.9, 0.0, blink) - lid_noise;

    // Outside the eye shape = transparent
    if eye_shape > 0.95 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Eyelid occlusion
    if p.y > eyelid_top || p.y < eyelid_bottom {
        let skin_var = noise(p * 8.0) * 0.08;
        return vec4<f32>(0.35 + skin_var, 0.15 - skin_var * 0.5, 0.18, 1.0);
    }

    // Sclera with visible veining
    let vein1 = noise(p * 15.0 + vec2<f32>(t * 0.02, 0.0));
    let vein2 = noise(p * 30.0 + vec2<f32>(0.0, t * 0.01));
    let vein_pattern = pow(vein1, 3.0) * 0.15 + pow(vein2, 4.0) * 0.08;
    var color = material.sclera_color.rgb - vec3<f32>(vein_pattern, vein_pattern * 0.3, vein_pattern * 0.2);

    // Yellowish discoloration patches
    let yellow_patch = noise(p * 4.0 + vec2<f32>(t * 0.005, 3.7));
    color += vec3<f32>(yellow_patch * 0.06, yellow_patch * 0.04, -yellow_patch * 0.03);

    // Redness at edges
    let edge_redness = smoothstep(0.4, 0.95, eye_shape) * 0.35;
    color = mix(color, vec3<f32>(0.8, 0.15, 0.1), edge_redness);

    // Iris
    let iris_radius = 0.45;
    let pupil_base_radius = mix(0.15, 0.3, material.pupil_dilation);
    // Pupil throbs slightly
    let pupil_radius = pupil_base_radius + sin(t * 2.5) * 0.012;

    if d < iris_radius {
        // Iris pattern - irregular radial streaks
        let angle = atan2(p.y - iris_center.y, p.x - iris_center.x);
        let streak1 = sin(angle * 12.0 + t * 0.5) * 0.5 + 0.5;
        let streak2 = sin(angle * 7.0 - t * 0.3 + 2.0) * 0.3;
        let radial_streak = streak1 + streak2;
        let iris_blend = smoothstep(pupil_radius, iris_radius, d);

        // Iris color with uneven streaks
        let iris = mix(
            material.iris_color.rgb * 0.4,
            material.iris_color.rgb * (1.0 + streak2 * 0.3),
            radial_streak * iris_blend
        );

        // Ragged darker rim
        let rim_noise = noise(vec2<f32>(angle * 3.0, d * 10.0)) * 0.04;
        let rim = smoothstep(iris_radius - 0.07 + rim_noise, iris_radius, d);
        let iris_with_rim = mix(iris, iris * 0.2, rim);

        color = iris_with_rim;

        // Pupil
        if d < pupil_radius {
            let pupil_edge = smoothstep(pupil_radius - 0.04, pupil_radius, d);
            color = mix(vec3<f32>(0.02, 0.005, 0.02), color, pupil_edge);
        }

        // Specular highlight - slightly wobbly
        let spec_pos = iris_center + vec2<f32>(-0.1 + sin(t * 1.1) * 0.015, 0.12);
        let spec_d = length(p - spec_pos);
        let spec = smoothstep(0.08, 0.0, spec_d);
        color = mix(color, vec3<f32>(1.0, 1.0, 1.0), spec * 0.7);

        // Second smaller specular
        let spec2_pos = iris_center + vec2<f32>(0.06, -0.05);
        let spec2_d = length(p - spec2_pos);
        let spec2 = smoothstep(0.04, 0.0, spec2_d);
        color = mix(color, vec3<f32>(1.0, 1.0, 0.95), spec2 * 0.35);
    }

    // Edge of eyeball shadow
    let shadow = smoothstep(0.6, 0.95, eye_shape);
    color *= (1.0 - shadow * 0.4);

    return vec4<f32>(color, 1.0);
}
