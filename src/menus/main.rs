//! The main menu (seen on the title screen).
use bevy::{
	prelude::*,
	scene::SceneInstanceReady,
	window::{CursorGrabMode, CursorOptions},
};

use crate::{
	gameplay::npc::Npc, menus::Menu, theme::widget,
	third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _,
};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Menu::Main), spawn_main_menu)
		.add_systems(Update, spawn_dancer.run_if(in_state(Menu::Main)));
}

fn spawn_main_menu(mut commands: Commands, mut cursor_options: Single<&mut CursorOptions>) {
	cursor_options.grab_mode = CursorGrabMode::None;
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Play", open_level_select),
	));
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Settings", open_settings_menu),
	));
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Credits", open_credits_menu),
	));
	#[cfg(not(target_family = "wasm"))]
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Exit", exit_app),
	));
	commands.spawn((DespawnOnExit(Menu::Main), Camera3d::default()));
	commands.spawn((DespawnOnExit(Menu::Main), DirectionalLight::default()));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct DancingFriendo;

fn spawn_dancer(
	mut commands: Commands,
	assets: Res<AssetServer>,
	dancer: Query<(), With<DancingFriendo>>,
	gltfs: Res<Assets<Gltf>>,
	mut graphs: ResMut<Assets<AnimationGraph>>,
	mut gltf_handle: Local<Option<Handle<Gltf>>>,
	mut dance_index: Local<Option<(AnimationNodeIndex, Handle<AnimationGraph>)>>,
) {
	if !dancer.is_empty() {
		return;
	}
	let gltf_handle = gltf_handle.get_or_insert_with(|| assets.load(Npc::model_path()));
	let Some(gltf) = gltfs.get(gltf_handle) else {
		return;
	};
	let (dance_index, ..) = dance_index.get_or_insert_with(|| {
		let (graph, indices) =
			AnimationGraph::from_clips([gltf.named_animations.get("dance").unwrap().clone()]);
		let [dance_index] = indices.as_slice() else {
			unreachable!()
		};
		let graph_handle = graphs.add(graph);
		(*dance_index, graph_handle)
	});
	let dance_index = *dance_index;

	commands
		.spawn((
			DespawnOnExit(Menu::Main),
			SceneRoot(gltf.scenes[0].clone()),
			Transform::from_xyz(0.0, 0.0, 5.0),
			DancingFriendo,
		))
		.observe(
			move |ready: On<SceneInstanceReady>,
			      mut animation_players: Query<&mut AnimationPlayer>,
			      children: Query<&Children>| {
				let mut animation_players =
					animation_players.iter_many_mut(children.iter_descendants(ready.entity));
				let mut animation_player = animation_players.fetch_next().unwrap();
				animation_player.play(dance_index);
			},
		);
}

fn open_level_select(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::LevelSelect);
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
	app_exit.write(AppExit::Success);
}
