//! Toggles for the different debug UIs that our plugins provide.

use std::any::Any as _;

use super::input::{ForceFreeCursor, ToggleDebugUi};
use crate::RenderLayer;
use crate::gameplay::crosshair::CrosshairState;
use crate::gameplay::level::LevelAssets;
use crate::{PostPhysicsAppSystems, theme::widget};
use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::dev_tools::fps_overlay::FrameTimeGraphConfig;
use bevy::ui::Val::*;
use bevy::{
	dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
	prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_feronia::prelude::{
	ChunkDebugConfig, HeightMapDebugConfig, ScatterOccupancyMapDebugConfig,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_landmass::debug::{EnableLandmassDebug, Landmass3dDebugPlugin, LandmassGizmos};
use bevy_rerecast::debug::{DetailNavmeshGizmo, NavmeshGizmoConfig};

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<DebugState>();
	app.init_resource::<InspectorActive>();
	app.add_plugins(FpsOverlayPlugin {
		config: FpsOverlayConfig {
			enabled: false,
			frame_time_graph_config: FrameTimeGraphConfig {
				enabled: false,
				..default()
			},
			..default()
		},
	});
	app.add_plugins((
		EguiPlugin::default(),
		WorldInspectorPlugin::new().run_if(is_inspector_active),
	));

	app.add_plugins((
		PhysicsDebugPlugin,
		PhysicsDiagnosticsPlugin,
		PhysicsDiagnosticsUiPlugin,
		Landmass3dDebugPlugin {
			draw_on_start: false,
			..default()
		},
	));
	app.insert_resource(PhysicsDiagnosticsUiSettings {
		enabled: false,
		..default()
	});
	app.insert_gizmo_config(
		PhysicsGizmos::default(),
		GizmoConfig {
			enabled: false,
			render_layers: RenderLayers::from(RenderLayer::GIZMO3),
			..default()
		},
	);
	app.insert_gizmo_config(
		LightGizmoConfigGroup {
			draw_all: true,
			..default()
		},
		GizmoConfig {
			enabled: false,
			render_layers: RenderLayers::from(RenderLayer::GIZMO3),
			..default()
		},
	);
	app.insert_gizmo_config(
		LandmassGizmos {
			boundary_edge: None,
			connectivity_edge: None,
			..default()
		},
		GizmoConfig {
			enabled: true,
			depth_bias: -0.03,
			render_layers: RenderLayers::from(RenderLayer::GIZMO3),
			..default()
		},
	);
	app.add_observer(advance_debug_state);
	app.add_observer(toggle_egui_inspector);

	app.add_systems(Startup, setup_debug_ui_text);
	app.add_systems(
		Update,
		update_debug_ui_text.run_if(resource_exists_and_changed::<DebugState>),
	);
	app.add_systems(
		Update,
		add_navmesh_gizmo.run_if(resource_exists_and_changed::<LevelAssets>),
	);
	app.add_systems(
		Update,
		(
			toggle_fps_overlay.run_if(toggled_state(DebugState::None)),
			toggle_debug_ui.run_if(toggled_state(DebugState::Ui)),
			toggle_lighting_debug_ui.run_if(toggled_state(DebugState::Lighting)),
			toggle_physics_debug_ui.run_if(toggled_state(DebugState::Physics)),
			toggle_landmass_debug_ui.run_if(toggled_state(DebugState::Landmass)),
			toggle_scatter_debug_ui.run_if(toggled_state(DebugState::Scatter)),
		)
			.chain()
			.in_set(PostPhysicsAppSystems::ChangeUi),
	);
}

fn add_navmesh_gizmo(
	level: Res<LevelAssets>,
	mut commands: Commands,
	mut gizmo_config: ResMut<NavmeshGizmoConfig>,
) {
	commands.spawn(DetailNavmeshGizmo::new(&level.navmesh));
	gizmo_config.detail_navmesh.enabled = false;
	gizmo_config.detail_navmesh.render_layers = RenderLayers::from(RenderLayer::GIZMO3);
}

fn setup_debug_ui_text(mut commands: Commands) {
	commands.spawn((
		Name::new("Debug UI"),
		Node {
			position_type: PositionType::Absolute,
			width: Percent(100.0),
			height: Percent(100.0),
			justify_content: JustifyContent::End,
			..default()
		},
		Pickable::IGNORE,
		children![(widget::label("Debug UI"), DebugUiText)],
	));
}

#[derive(Component)]
struct DebugUiText;

fn advance_debug_state(_on: On<Start<ToggleDebugUi>>, mut debug_state: ResMut<DebugState>) {
	*debug_state = debug_state.next();
}

fn update_debug_ui_text(
	debug_state: Res<DebugState>,
	mut text: Single<&mut Text, With<DebugUiText>>,
) {
	text.0 = match *debug_state {
		DebugState::None => "",
		DebugState::Ui => "Ui",
		DebugState::Lighting => "Lighting",
		DebugState::Physics => "Physics",
		DebugState::Landmass => "Landmass",
		DebugState::Scatter => "Scatter",
	}
	.to_string();
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
	options.toggle();
}

fn toggle_physics_debug_ui(
	mut config_store: ResMut<GizmoConfigStore>,
	mut physics_diagnostics: ResMut<PhysicsDiagnosticsUiSettings>,
) {
	let config = config_store.config_mut::<PhysicsGizmos>().0;
	config.enabled = !config.enabled;
	physics_diagnostics.enabled = !physics_diagnostics.enabled;
}

fn toggle_lighting_debug_ui(mut config_store: ResMut<GizmoConfigStore>) {
	let config = config_store.config_mut::<LightGizmoConfigGroup>().0;
	config.enabled = !config.enabled;
}

fn toggle_landmass_debug_ui(
	mut debug: ResMut<EnableLandmassDebug>,
	mut navmesh: ResMut<NavmeshGizmoConfig>,
) {
	**debug = !**debug;
	navmesh.detail_navmesh.enabled = !navmesh.detail_navmesh.enabled;
}

fn toggle_scatter_debug_ui(
	mut cmd: Commands,
	scatter_debug: Option<Res<ScatterOccupancyMapDebugConfig>>,
	height_map_debug: Option<Res<HeightMapDebugConfig>>,
	chunk_debug: Option<Res<ChunkDebugConfig>>,
) {
	if scatter_debug.is_some() {
		cmd.remove_resource::<ScatterOccupancyMapDebugConfig>();
	} else {
		cmd.init_resource::<ScatterOccupancyMapDebugConfig>();
	}
	if height_map_debug.is_some() {
		cmd.remove_resource::<HeightMapDebugConfig>();
	} else {
		cmd.init_resource::<HeightMapDebugConfig>();
	}
	if chunk_debug.is_some() {
		cmd.remove_resource::<ChunkDebugConfig>();
	} else {
		cmd.insert_resource(ChunkDebugConfig {
			show_aabbs: true,
			show_lod_ranges: true,
			..default()
		});
	}
}

fn toggle_fps_overlay(mut config: ResMut<FpsOverlayConfig>) {
	config.enabled = !config.enabled;
	config.frame_time_graph_config.enabled = config.enabled;
}

fn toggle_egui_inspector(
	_on: On<Start<ForceFreeCursor>>,
	mut crosshair_state: Single<&mut CrosshairState>,
	mut inspector_active: ResMut<InspectorActive>,
) {
	inspector_active.0 = !inspector_active.0;
	if inspector_active.0 {
		crosshair_state
			.wants_free_cursor
			.insert(toggle_egui_inspector.type_id());
	} else {
		crosshair_state
			.wants_free_cursor
			.remove(&toggle_egui_inspector.type_id());
	}
}

#[derive(Resource, Debug, Default, Eq, PartialEq)]
struct InspectorActive(bool);

fn is_inspector_active(inspector_active: Res<InspectorActive>) -> bool {
	inspector_active.0
}

#[derive(Resource, Debug, Default, Eq, PartialEq)]
enum DebugState {
	#[default]
	None,
	Ui,
	Lighting,
	Physics,
	Landmass,
	Scatter,
}

impl DebugState {
	fn next(&self) -> Self {
		match self {
			Self::None => Self::Ui,
			Self::Ui => Self::Lighting,
			Self::Lighting => Self::Physics,
			Self::Physics => Self::Landmass,
			Self::Landmass => Self::Scatter,
			Self::Scatter => Self::None,
		}
	}
}

fn toggled_state(state: DebugState) -> impl SystemCondition<()> {
	IntoSystem::into_system(move |current_state: Res<DebugState>| {
		let was_just_changed = current_state.is_changed() && !current_state.is_added();
		let entered_state = *current_state == state;
		let exited_state = *current_state == state.next();
		was_just_changed && (entered_state || exited_state)
	})
}
