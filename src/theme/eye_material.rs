use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

use crate::theme::palette::BIOLUMINESCENT;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(UiMaterialPlugin::<EyeMaterial>::default());
    app.add_systems(Update, (update_eye_blink, update_eye_cursor_tracking));
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct EyeMaterial {
    #[uniform(0)]
    pub(crate) iris_color: LinearRgba,
    #[uniform(0)]
    pub(crate) sclera_color: LinearRgba,
    #[uniform(0)]
    pub(crate) cursor_pos: Vec2,
    #[uniform(0)]
    pub(crate) pupil_dilation: f32,
    #[uniform(0)]
    pub(crate) blink_state: f32,
}

impl UiMaterial for EyeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/eye_ui.wgsl".into()
    }
}

impl EyeMaterial {
    pub(crate) fn menu_eye() -> Self {
        Self {
            iris_color: Srgba::from(BIOLUMINESCENT).into(),
            sclera_color: LinearRgba::new(0.92, 0.88, 0.85, 1.0),
            cursor_pos: Vec2::new(0.5, 0.5),
            pupil_dilation: 0.0,
            blink_state: 0.0,
        }
    }

    pub(crate) fn crosshair_eye() -> Self {
        Self {
            iris_color: Srgba::from(BIOLUMINESCENT).into(),
            sclera_color: LinearRgba::new(0.92, 0.88, 0.85, 1.0),
            cursor_pos: Vec2::new(0.5, 0.5),
            pupil_dilation: 0.0,
            blink_state: 0.0,
        }
    }
}

/// Marker component for eyes that should track the cursor
#[derive(Component)]
pub(crate) struct EyeTracksCursor;

/// Timer for eye blinking
#[derive(Component)]
pub(crate) struct EyeBlinkTimer {
    timer: Timer,
    blink_duration: f32,
    blink_progress: f32,
    is_blinking: bool,
}

impl EyeBlinkTimer {
    pub(crate) fn new(min_interval: f32, max_interval: f32) -> Self {
        let interval = rand::random::<f32>() * (max_interval - min_interval) + min_interval;
        Self {
            timer: Timer::from_seconds(interval, TimerMode::Once),
            blink_duration: 0.1 + rand::random::<f32>() * 0.15,
            blink_progress: 0.0,
            is_blinking: false,
        }
    }
}

fn update_eye_blink(
    time: Res<Time>,
    mut query: Query<(&mut EyeBlinkTimer, &MaterialNode<EyeMaterial>)>,
    mut materials: ResMut<Assets<EyeMaterial>>,
) {
    for (mut blink_timer, mat_node) in &mut query {
        blink_timer.timer.tick(time.delta());

        if blink_timer.is_blinking {
            blink_timer.blink_progress += time.delta_secs() / blink_timer.blink_duration;
            if blink_timer.blink_progress >= 1.0 {
                blink_timer.is_blinking = false;
                blink_timer.blink_progress = 0.0;
                // Sometimes double-blink (30% chance, short gap)
                let interval = if rand::random::<f32>() < 0.3 {
                    0.08 + rand::random::<f32>() * 0.12
                } else {
                    1.5 + rand::random::<f32>() * 6.0
                };
                blink_timer.blink_duration = 0.08 + rand::random::<f32>() * 0.18;
                blink_timer.timer = Timer::from_seconds(interval, TimerMode::Once);
                if let Some(mat) = materials.get_mut(mat_node) {
                    mat.blink_state = 0.0;
                }
            } else {
                // Blink curve: quick close, slower open
                let blink = if blink_timer.blink_progress < 0.3 {
                    blink_timer.blink_progress / 0.3
                } else {
                    1.0 - (blink_timer.blink_progress - 0.3) / 0.7
                };
                if let Some(mat) = materials.get_mut(mat_node) {
                    mat.blink_state = blink;
                }
            }
        } else if blink_timer.timer.just_finished() {
            blink_timer.is_blinking = true;
            blink_timer.blink_progress = 0.0;
        }
    }
}

fn update_eye_cursor_tracking(
    window_query: Query<&Window>,
    query: Query<&MaterialNode<EyeMaterial>, With<EyeTracksCursor>>,
    mut materials: ResMut<Assets<EyeMaterial>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let normalized = Vec2::new(
        cursor_pos.x / window.width(),
        cursor_pos.y / window.height(),
    );

    for mat_node in &query {
        if let Some(mat) = materials.get_mut(mat_node) {
            mat.cursor_pos = normalized;
        }
    }
}
