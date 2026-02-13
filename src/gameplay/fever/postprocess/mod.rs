use crate::gameplay::core::fever::systems::FeverTick;
use crate::gameplay::core::{
	BaseTemperature, Fever, MaxTemperature, Temperature, TemperatureThreshold,
};
use bevy::core_pipeline::prepass::ViewPrepassTextures;
use bevy::render::globals::{GlobalsBuffer, GlobalsUniform};
use bevy::window::PrimaryWindow;
use bevy::{
	core_pipeline::{
		FullscreenShader,
		core_3d::graph::{Core3d, Node3d},
	},
	ecs::query::QueryItem,
	prelude::*,
	render::{
		RenderApp, RenderStartup,
		extract_component::{
			ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
			UniformComponentPlugin,
		},
		render_graph::{
			NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
		},
		render_resource::{
			binding_types::{sampler, texture_2d, uniform_buffer},
			*,
		},
		renderer::{RenderContext, RenderDevice},
		view::ViewTarget,
	},
};
use bevy_ahoy::CharacterController;

pub fn plugin(app: &mut App) {
	app.add_plugins(FeverPostProcessPlugin)
		.add_systems(Update, update_settings)
		.add_observer(on_fever_tick);
}

const SHADER_ASSET_PATH: &str = "shaders/fullscreen_effect.wgsl";

struct FeverPostProcessPlugin;

impl Plugin for FeverPostProcessPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			ExtractComponentPlugin::<FeverPostProcessSettings>::default(),
			UniformComponentPlugin::<FeverPostProcessSettings>::default(),
		));

		let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
			return;
		};

		render_app.add_systems(RenderStartup, init_post_process_pipeline);
		render_app
			.add_render_graph_node::<ViewNodeRunner<FeverPostProcessNode>>(
				Core3d,
				FeverPostProcessLabel,
			)
			.add_render_graph_edges(
				Core3d,
				(
					Node3d::Tonemapping,
					FeverPostProcessLabel,
					Node3d::EndMainPassPostProcessing,
				),
			);
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct FeverPostProcessLabel;

#[derive(Default)]
struct FeverPostProcessNode;

impl ViewNode for FeverPostProcessNode {
	type ViewQuery = (
		&'static ViewTarget,
		&'static FeverPostProcessSettings,
		&'static DynamicUniformIndex<FeverPostProcessSettings>,
	);

	fn run(
		&self,
		_graph: &mut RenderGraphContext,
		render_context: &mut RenderContext,
		(view_target, _post_process_settings, settings_index): QueryItem<Self::ViewQuery>,
		world: &World,
	) -> Result<(), NodeRunError> {
		let post_process_pipeline = world.resource::<PostProcessPipeline>();
		let pipeline_cache = world.resource::<PipelineCache>();

		let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
		else {
			return Ok(());
		};

		let settings = world.resource::<ComponentUniforms<FeverPostProcessSettings>>();
		let Some(settings_binding) = settings.uniforms().binding() else {
			return Ok(());
		};

		let globals_buffer = world.resource::<GlobalsBuffer>();
		let Some(globals_binding) = globals_buffer.buffer.binding() else {
			return Ok(());
		};

		let prepass_textures = world.get::<ViewPrepassTextures>(_graph.view_entity());

		let depth_view = prepass_textures
			.and_then(|p| p.depth.as_ref())
			.map(|t| &t.texture.default_view);
		
		let motion_view = prepass_textures
			.and_then(|p| p.motion_vectors.as_ref())
			.map(|t| &t.texture.default_view);

		let (Some(depth), Some(motion)) = (depth_view, motion_view) else {
			return Ok(());
		};

		let post_process = view_target.post_process_write();
		let bind_group = render_context.render_device().create_bind_group(
			"post_process_bind_group",
			&pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
			&BindGroupEntries::sequential((
				post_process.source,
				&post_process_pipeline.sampler,
				settings_binding.clone(),
				globals_binding.clone(),
				depth,
				motion,
			)),
		);

		let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
			label: Some("post_process_pass"),
			color_attachments: &[Some(RenderPassColorAttachment {
				view: post_process.destination,
				depth_slice: None,
				resolve_target: None,
				ops: Operations::default(),
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
		});

		render_pass.set_render_pipeline(pipeline);
		render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
		render_pass.draw(0..3, 0..1);

		Ok(())
	}
}

#[derive(Resource)]
struct PostProcessPipeline {
	layout: BindGroupLayoutDescriptor,
	sampler: Sampler,
	pipeline_id: CachedRenderPipelineId,
}

fn init_post_process_pipeline(
	mut cmd: Commands,
	render_device: Res<RenderDevice>,
	asset_server: Res<AssetServer>,
	fullscreen_shader: Res<FullscreenShader>,
	pipeline_cache: Res<PipelineCache>,
) {
	let layout = BindGroupLayoutDescriptor::new(
		"post_process_bind_group_layout",
		&BindGroupLayoutEntries::sequential(
			ShaderStages::FRAGMENT,
			(
				texture_2d(TextureSampleType::Float { filterable: true }),
				sampler(SamplerBindingType::Filtering),
				uniform_buffer::<FeverPostProcessSettings>(true),
				uniform_buffer::<GlobalsUniform>(false),
				texture_2d(TextureSampleType::Depth),
				texture_2d(TextureSampleType::Float { filterable: false }),
			),
		),
	);
	let sampler = render_device.create_sampler(&SamplerDescriptor::default());
	let shader = asset_server.load(SHADER_ASSET_PATH);
	let vertex_state = fullscreen_shader.to_vertex_state();
	let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
		label: Some("post_process_pipeline".into()),
		layout: vec![layout.clone()],
		vertex: vertex_state,
		fragment: Some(FragmentState {
			shader,
			targets: vec![Some(ColorTargetState {
				format: TextureFormat::Rgba16Float,
				blend: None,
				write_mask: ColorWrites::ALL,
			})],
			..default()
		}),
		..default()
	});

	cmd.insert_resource(PostProcessPipeline {
		layout,
		sampler,
		pipeline_id,
	});
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct FeverPostProcessSettings {
	pub resolution: Vec2,
	pub intensity: f32,
	pub fever: f32,
	pub damage_threshold: f32,
	pub damage_indicator: f32,
	_pad: Vec2,
}

impl Default for FeverPostProcessSettings {
	fn default() -> Self {
		Self {
			intensity: 1.,
			resolution: Vec2::new(1920., 1080.),
			fever: 0.,
			damage_threshold: 0.,
			damage_indicator: 0.,
			_pad: Vec2::new(0., 0.),
		}
	}
}

fn on_fever_tick(
	_: On<FeverTick>,
	mut settings: Query<&mut FeverPostProcessSettings>,
	fever: Single<(&Temperature, &TemperatureThreshold), (With<Fever>, With<CharacterController>)>,
) {
	let (temperature, threshold) = fever.into_inner();
	if **temperature < **threshold {
		return;
	}

	for mut setting in &mut settings {
		setting.damage_indicator = 1.;
	}
}

fn update_settings(
	mut settings: Query<&mut FeverPostProcessSettings>,
	fever: Single<
		(
			&Temperature,
			&MaxTemperature,
			&BaseTemperature,
			&TemperatureThreshold,
		),
		(With<Fever>, With<CharacterController>),
	>,
	time: Res<Time>,
	window: Single<&Window, With<PrimaryWindow>>,
) {
	let window_size = window.size();
	let (current, max, base, threshold) = fever.into_inner();
	let range = (**max - **base).max(0.0001);
	let fever = (**current - **base) / range;
	let threshold = (**threshold - **base) / range;

	for mut setting in &mut settings {
		setting.fever = fever.clamp(0.0, 1.0);
		setting.damage_threshold = threshold.clamp(0.0, 1.0);
		setting.damage_indicator = (setting.damage_indicator - time.delta_secs()).max(0.0);
		setting.resolution = window_size;
	}
}
