//! The settings screen accessible from the title screen.
//! We can add all manner of settings and accessibility options here.
//! For 3D, we'd also place the camera sensitivity and FOV here.

use bevy::ecs::query::QueryFilter;
use bevy::window::PresentMode;
use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_seedling::prelude::*;

use crate::ui_layout::RootWidget;
use crate::{
	Pause,
	audio::{MusicPool, perceptual::PerceptualVolumeConverter},
	gameplay::player::camera::{CameraSensitivity, WorldModelFov},
	menus::Menu,
	screens::Screen,
	theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
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
			update_volume_label::<With<GlobalVolumeLabel>, With<MainBus>>,
			update_volume_label::<With<MusicVolumeLabel>, With<SamplerPool<MusicPool>>>,
			update_volume_label::<With<SfxVolumeLabel>, With<SoundEffectsBus>>,
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
		RootWidget,
		DespawnOnExit(Menu::Settings),
		DespawnOnExit(Screen::Gameplay),
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
						widget::label("Global Volume"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(
						GlobalVolumeLabel,
						lower_volume::<With<MainBus>>,
						raise_volume::<With<MainBus>>
					),
					(
						widget::label("Music Volume"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(
						MusicVolumeLabel,
						lower_volume::<With<SamplerPool<MusicPool>>>,
						raise_volume::<With<SamplerPool<MusicPool>>>
					),
					(
						widget::label("Sound Effects Volume"),
						Node {
							justify_self: JustifySelf::End,
							..default()
						}
					),
					widget::plus_minus_bar(
						SfxVolumeLabel,
						lower_volume::<With<SoundEffectsBus>>,
						raise_volume::<With<SoundEffectsBus>>
					),
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
struct VolumeTicks(usize);

impl VolumeTicks {
	fn increment(&mut self) {
		self.0 = Self::MAX_TICK_COUNT.min(self.0 + 1);
	}

	fn decrement(&mut self) {
		self.0 = self.0.saturating_sub(1);
	}

	fn fraction(&self) -> f32 {
		self.0 as f32 / Self::MAX_TICK_COUNT as f32
	}

	fn label(&self) -> String {
		let filled = "█".repeat(self.0);
		let empty = " ".repeat(VolumeTicks::MAX_TICK_COUNT - self.0);
		filled + &empty + "|"
	}

	/// How many ticks the volume slider supports
	const MAX_TICK_COUNT: usize = 20;
}

impl From<VolumeTicks> for Volume {
	fn from(value: VolumeTicks) -> Self {
		PerceptualVolumeConverter::default().to_volume(value.fraction())
	}
}

impl From<Volume> for VolumeTicks {
	fn from(value: Volume) -> Self {
		VolumeTicks(
			(PerceptualVolumeConverter::default().to_perceptual(value)
				* Self::MAX_TICK_COUNT as f32)
				.round() as usize,
		)
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct MusicVolumeLabel;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct SfxVolumeLabel;

fn lower_volume<F: QueryFilter>(_on: On<Pointer<Click>>, mut volume: Single<&mut VolumeNode, F>) {
	let mut ticks = VolumeTicks::from(volume.volume);
	ticks.decrement();
	volume.volume = ticks.into();
}

fn raise_volume<F: QueryFilter>(_on: On<Pointer<Click>>, mut master: Single<&mut VolumeNode, F>) {
	let mut ticks = VolumeTicks::from(master.volume);
	ticks.increment();
	master.volume = ticks.into();
}

fn update_volume_label<F1, F2>(mut label: Single<&mut Text, F1>, master: Single<&VolumeNode, F2>)
where
	F1: QueryFilter,
	F2: QueryFilter,
{
	let ticks = VolumeTicks::from(master.volume);
	label.0 = ticks.label();
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
