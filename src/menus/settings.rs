//! The settings screen accessible from the title screen.
//! We can add all manner of settings and accessibility options here.
//! For 3D, we'd also place the camera sensitivity and FOV here.

use bevy::window::PresentMode;
use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_seedling::prelude::*;

use crate::{
	Pause,
	audio::{DEFAULT_MAIN_VOLUME, perceptual::PerceptualVolumeConverter},
	gameplay::player::camera::{CameraSensitivity, WorldModelFov},
	menus::Menu,
	screens::Screen,
	theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<VolumeSliderSettings>();
	app.init_resource::<VsyncSetting>();
	app.init_resource::<FpsLimiterSettings>();
	app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);
	app.add_systems(
		Update,
		go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
	);

	app.add_systems(
		Update,
		(
			update_global_volume.run_if(resource_exists_and_changed::<VolumeSliderSettings>),
			update_volume_label,
			update_camera_sensitivity_label,
			update_camera_fov_label,
			update_vsync.run_if(resource_exists_and_changed::<VsyncSetting>),
			update_vsync_label,
			update_fps_limiter.run_if(resource_exists_and_changed::<FpsLimiterSettings>),
			update_fps_limiter_enabled_label,
			update_fps_limiter_target_label,
		)
			.run_if(in_state(Menu::Settings)),
	);
}

fn spawn_settings_menu(mut commands: Commands, paused: Res<State<Pause>>) {
	let mut entity_commands = commands.spawn((
		widget::ui_root("Settings Screen"),
		DespawnOnExit(Menu::Settings),
		GlobalZIndex(2),
		children![
			widget::header("Settings"),
			(
				Name::new("Settings Grid"),
				Node {
					display: Display::Grid,
					row_gap: Px(10.0),
					column_gap: Px(30.0),
					grid_template_columns: RepeatedGridTrack::px(2, 400.0),
					..default()
				},
				children![
					// Audio
					(
						widget::label("Audio Volume"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(GlobalVolumeLabel, lower_volume, raise_volume),
					// Camera Sensitivity
					(
						widget::label("Camera Sensitivity"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(
						CameraSensitivityLabel,
						lower_camera_sensitivity,
						raise_camera_sensitivity
					),
					// Camera FOV
					(
						widget::label("Camera FOV"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(CameraFovLabel, lower_camera_fov, raise_camera_fov),
					// VSync
					(
						widget::label("VSync"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(VsyncLabel, disable_vsync, enable_vsync),
					// FPS Limiter (Enable/Disable)
					(
						widget::label("FPS Limiter"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(
						FpsLimiterEnabledLabel,
						disable_fps_limiter,
						enable_fps_limiter
					),
					// FPS Target
					(
						widget::label("FPS Target"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(
						FpsLimiterTargetLabel,
						lower_fps_target,
						raise_fps_target
					),
				],
			),
			widget::button("Back", go_back_on_click),
		],
	));
	if paused.get() == &Pause(false) {
		entity_commands.insert(BackgroundColor(SCREEN_BACKGROUND));
	}
}

#[derive(Resource, Reflect, Debug)]
struct VolumeSliderSettings(usize);

impl VolumeSliderSettings {
	fn increment(&mut self) {
		self.0 = Self::MAX_TICK_COUNT.min(self.0 + 1);
	}

	fn decrement(&mut self) {
		self.0 = self.0.saturating_sub(1);
	}

	fn fraction(&self) -> f32 {
		self.0 as f32 / Self::MAX_TICK_COUNT as f32
	}

	/// How many ticks the volume slider supports
	const MAX_TICK_COUNT: usize = 20;
}

impl Default for VolumeSliderSettings {
	fn default() -> Self {
		Self(
			(PerceptualVolumeConverter::default().to_perceptual(DEFAULT_MAIN_VOLUME)
				* Self::MAX_TICK_COUNT as f32)
				.round() as usize,
		)
	}
}

fn update_global_volume(
	mut master: Single<&mut VolumeNode, With<MainBus>>,
	volume_step: Res<VolumeSliderSettings>,
) {
	master.volume = PerceptualVolumeConverter::default().to_volume(volume_step.fraction());
}

fn lower_volume(_on: On<Pointer<Click>>, mut volume_step: ResMut<VolumeSliderSettings>) {
	volume_step.decrement();
}

fn raise_volume(_on: On<Pointer<Click>>, mut volume_step: ResMut<VolumeSliderSettings>) {
	volume_step.increment();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

fn update_volume_label(
	mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
	slider: Res<VolumeSliderSettings>,
) {
	let ticks = slider.0;
	let filled = "█".repeat(ticks);
	let empty = " ".repeat(VolumeSliderSettings::MAX_TICK_COUNT - ticks);
	let text = filled + &empty + "|";
	label.0 = text;
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct CameraSensitivityLabel;

fn lower_camera_sensitivity(
	_on: On<Pointer<Click>>,
	mut camera_sensitivity: ResMut<CameraSensitivity>,
) {
	camera_sensitivity.0 -= 0.1;
	const MIN_SENSITIVITY: f32 = 0.1;
	camera_sensitivity.x = camera_sensitivity.x.max(MIN_SENSITIVITY);
	camera_sensitivity.y = camera_sensitivity.y.max(MIN_SENSITIVITY);
}

fn raise_camera_sensitivity(
	_on: On<Pointer<Click>>,
	mut camera_sensitivity: ResMut<CameraSensitivity>,
) {
	camera_sensitivity.0 += 0.1;
	const MAX_SENSITIVITY: f32 = 20.0;
	camera_sensitivity.x = camera_sensitivity.x.min(MAX_SENSITIVITY);
	camera_sensitivity.y = camera_sensitivity.y.min(MAX_SENSITIVITY);
}

fn update_camera_sensitivity_label(
	mut label: Single<&mut Text, With<CameraSensitivityLabel>>,
	camera_sensitivity: Res<CameraSensitivity>,
) {
	label.0 = format!("{:.1}", camera_sensitivity.x);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct CameraFovLabel;

fn lower_camera_fov(_on: On<Pointer<Click>>, mut camera_fov: ResMut<WorldModelFov>) {
	camera_fov.0 -= 1.0;
	camera_fov.0 = camera_fov.0.max(45.0);
}

fn raise_camera_fov(_on: On<Pointer<Click>>, mut camera_fov: ResMut<WorldModelFov>) {
	camera_fov.0 += 1.0;
	camera_fov.0 = camera_fov.0.min(130.0);
}

fn update_camera_fov_label(
	mut label: Single<&mut Text, With<CameraFovLabel>>,
	camera_fov: Res<WorldModelFov>,
) {
	label.0 = format!("{:.1}", camera_fov.0);
}

#[derive(Resource, Reflect, Debug)]
struct VsyncSetting(bool);

impl Default for VsyncSetting {
	fn default() -> Self {
		Self(true)
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct VsyncLabel;

fn enable_vsync(_on: On<Pointer<Click>>, mut setting: ResMut<VsyncSetting>) {
	setting.0 = true;
}

fn disable_vsync(_on: On<Pointer<Click>>, mut setting: ResMut<VsyncSetting>) {
	setting.0 = false;
}

fn update_vsync(mut window: Single<&mut Window>, setting: Res<VsyncSetting>) {
	window.present_mode = if setting.0 {
		PresentMode::AutoVsync
	} else {
		PresentMode::AutoNoVsync
	};
}

fn update_vsync_label(mut label: Single<&mut Text, With<VsyncLabel>>, setting: Res<VsyncSetting>) {
	label.0 = if setting.0 { "On".into() } else { "Off".into() };
}

#[derive(Resource, Reflect, Debug)]
struct FpsLimiterSettings {
	enabled: bool,
	target_fps: u32,
}

impl Default for FpsLimiterSettings {
	fn default() -> Self {
		Self {
			enabled: false,
			target_fps: 60,
		}
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct FpsLimiterEnabledLabel;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct FpsLimiterTargetLabel;

fn enable_fps_limiter(
	_on: On<Pointer<Click>>,
	mut settings: ResMut<FpsLimiterSettings>,
	mut framepace: ResMut<FramepaceSettings>,
) {
	settings.enabled = true;
	framepace.limiter = Limiter::from_framerate(settings.target_fps as f64);
}

fn disable_fps_limiter(
	_on: On<Pointer<Click>>,
	mut settings: ResMut<FpsLimiterSettings>,
	mut framepace: ResMut<FramepaceSettings>,
) {
	settings.enabled = false;
	framepace.limiter = Limiter::Off;
}

fn lower_fps_target(_on: On<Pointer<Click>>, mut settings: ResMut<FpsLimiterSettings>) {
	let min_fps = 30;
	let step = 5;
	settings.target_fps = settings.target_fps.saturating_sub(step).max(min_fps);
}

fn raise_fps_target(_on: On<Pointer<Click>>, mut settings: ResMut<FpsLimiterSettings>) {
	let max_fps = 360;
	let step = 5;
	settings.target_fps = (settings.target_fps + step).min(max_fps);
}

fn update_fps_limiter(mut framepace: ResMut<FramepaceSettings>, settings: Res<FpsLimiterSettings>) {
	framepace.limiter = if settings.enabled {
		Limiter::from_framerate(settings.target_fps as f64)
	} else {
		Limiter::Off
	};
}

fn update_fps_limiter_enabled_label(
	mut label: Single<&mut Text, With<FpsLimiterEnabledLabel>>,
	settings: Res<FpsLimiterSettings>,
) {
	label.0 = if settings.enabled {
		"On".into()
	} else {
		"Off".into()
	};
}

fn update_fps_limiter_target_label(
	mut label: Single<&mut Text, With<FpsLimiterTargetLabel>>,
	settings: Res<FpsLimiterSettings>,
) {
	label.0 = format!("{}", settings.target_fps);
}

fn go_back_on_click(
	_on: On<Pointer<Click>>,
	screen: Res<State<Screen>>,
	mut next_menu: ResMut<NextState<Menu>>,
) {
	next_menu.set(if screen.get() == &Screen::Title {
		Menu::Main
	} else {
		Menu::Pause
	});
}

fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(if screen.get() == &Screen::Title {
		Menu::Main
	} else {
		Menu::Pause
	});
}
