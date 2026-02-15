use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_seedling::firewheel::nodes::svf::SvfNode;
use bevy_seedling::prelude::*;

use crate::menus::Menu;
use animation::AnimateCutoff;

pub(crate) mod animation;
pub(crate) mod doppler;
pub(crate) mod layers;
pub(crate) mod perceptual;
pub(crate) mod world_emitter;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins((
		layers::plugin,
		doppler::DopplerPlugin,
		world_emitter::EmitterPlugin,
	))
	.add_systems(Startup, initialize_audio)
	.register_node::<SvfNode<2>>()
	.register_type::<SvfNode<2>>()
	.add_systems(Update, manage_filter_enabled)
	.add_systems(Update, layer_testing)
	.add_systems(OnExit(Menu::Pause), enable_music_filter)
	.add_systems(OnEnter(Menu::Pause), disable_music_filter);
}

#[derive(PoolLabel, Reflect, PartialEq, Eq, Debug, Hash, Clone)]
#[reflect(Component)]
pub(crate) struct SpatialPool;

#[derive(PoolLabel, Reflect, PartialEq, Eq, Debug, Hash, Clone)]
#[reflect(Component)]
pub(crate) struct SfxPool;

#[derive(PoolLabel, Reflect, PartialEq, Eq, Debug, Hash, Clone)]
#[reflect(Component)]
pub(crate) struct MusicPool;

#[derive(Component)]
pub(crate) struct MusicFilter;

/// Set somewhere below 0 dB so that the user can turn the volume up if they want to.
pub(crate) const DEFAULT_MAIN_VOLUME: Volume = Volume::Linear(0.5);

fn initialize_audio(server: Res<AssetServer>, mut commands: Commands) {
	// Tuned by ear
	const DEFAULT_POOL_VOLUME: Volume = Volume::Linear(1.6);

	// Buses
	commands
		.spawn((
			MainBus,
			VolumeNode {
				volume: DEFAULT_MAIN_VOLUME,
				..Default::default()
			},
			Name::new("Main Bus"),
		))
		.chain_node(LimiterNode::new(0.003, 0.15))
		.connect(AudioGraphOutput);

	commands.spawn((
		SoundEffectsBus,
		VolumeNode::from_decibels(-3.0),
		Name::new("Sound Effects Bus"),
	));

	commands
		.spawn((
			Name::new("Music audio sampler pool"),
			SamplerPool(MusicPool),
			sample_effects![VolumeNode::default()],
			VolumeNode { ..default() },
		))
		// we'll add a cute filter for menus
		.chain_node((
			SvfNode::<2> {
				filter_type: firewheel::nodes::svf::SvfType::LowpassX2,
				q_factor: 1.5,
				cutoff_hz: 20_000.0,
				enabled: false,
				..default()
			},
			MusicFilter,
		));

	// commands
	// 	.spawn((
	// 		Name::new("HRTF SFX pool"),
	// 		SamplerPool(HrtfPool),
	// 		sample_effects![(HrtfNode::default(), SpatialScale(Vec3::splat(2.0)))],
	// 		VolumeNode {
	// 			volume: DEFAULT_POOL_VOLUME,
	// 			..default()
	// 		},
	// 	))
	// 	.connect(SoundEffectsBus);

	commands
		.spawn((
			Name::new("SFX audio sampler pool"),
			SamplerPool(SpatialPool),
			sample_effects![(SpatialBasicNode::default(), SpatialScale(Vec3::splat(2.0)))],
			VolumeNode {
				volume: DEFAULT_POOL_VOLUME,
				..default()
			},
		))
		.connect(SoundEffectsBus);

	commands
		.spawn((
			Name::new("UI SFX audio sampler pool"),
			SamplerPool(SfxPool),
			VolumeNode {
				volume: DEFAULT_POOL_VOLUME,
				..default()
			},
		))
		.connect(SoundEffectsBus);

	commands.spawn(silly_breakcore_layers(&server));
}

fn silly_breakcore_layers(server: &AssetServer) -> impl Bundle {
	use layers::*;

	(
		Name::new("Silly Breakcore"),
		LayeredMusic { amount: 0.0 },
		// optional
		Intro {
			sample: server.load("audio/music/silly-breakcore/intro.wav"),
			duration: DurationSeconds(4.0 / 3.0),
		},
		children![
			Layer(server.load("audio/music/silly-breakcore/dnb.wav")),
			Layer(server.load("audio/music/silly-breakcore/lead.wav")),
			Layer(server.load("audio/music/silly-breakcore/voices.wav")),
		],
	)
}

/// A basic demonstration of how layering can be used.
fn layer_testing(
	mut events: MessageReader<KeyboardInput>,
	lay: Single<(Entity, &mut layers::LayeredMusic, Has<layers::ActiveMusic>)>,
	mut commands: Commands,
	mut level: Local<usize>,
) {
	use layers::*;

	let (layer_entity, mut amount, is_active) = lay.into_inner();

	for input in events.read() {
		if !input.state.is_pressed() {
			continue;
		}

		match input.key_code {
			KeyCode::ArrowRight => {
				*level = (*level + 1).min(4);
				if !is_active {
					commands.trigger(PlayLayeredMusic(layer_entity));
				}
				amount.amount = *level as f32 / 4.0;

				info!("Set layer level to {}", amount.amount);
			}
			KeyCode::ArrowLeft => {
				*level = level.saturating_sub(1);

				if *level == 0 && is_active {
					commands.trigger(StopLayeredMusic(layer_entity));
				}
				amount.amount = *level as f32 / 4.0;

				info!("Set layer level to {}", amount.amount);
			}
			_ => {}
		}
	}
}

// Sweep the filter down when entering a menu.
fn enable_music_filter(
	filter: Single<(&SvfNode, &mut AudioEvents), With<MusicFilter>>,
	time: Res<Time<Audio>>,
) {
	let (node, mut events) = filter.into_inner();
	node.animate_cutoff(800.0, 0.3, &time, &mut events);
}

// Sweep the filter back up when exiting a menu.
fn disable_music_filter(
	filter: Single<(&SvfNode, &mut AudioEvents), With<MusicFilter>>,
	time: Res<Time<Audio>>,
) {
	let (node, mut events) = filter.into_inner();
	node.animate_cutoff(20_000.0, 0.6, &time, &mut events);
}

// I want to make sure the filter is always disabled when above 20kHz.
//
// This is a bit more robust than scheduling events, since this can't be dropped.
fn manage_filter_enabled(filters: Query<&mut SvfNode, Changed<SvfNode>>) {
	for mut filter in filters {
		if filter.cutoff_hz >= 20_000.0 && filter.enabled {
			filter.enabled = false;
		} else if filter.cutoff_hz < 20_000.0 && !filter.enabled {
			filter.enabled = true;
		}
	}
}
