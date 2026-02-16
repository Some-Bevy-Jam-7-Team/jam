use crate::gameplay::core::*;
use crate::screens::Screen;
use bevy::prelude::*;
use bevy_ahoy::CharacterController;
use bevy_eidolon::prelude::*;
use bevy_feronia::prelude::*;

pub fn plugin(app: &mut App) {
	app.add_systems(Update, update_grass.run_if(in_state(Screen::Gameplay)));
}

pub fn update_grass(
	fever: Single<
		(
			&Temperature,
			&MaxTemperature,
			&BaseTemperature,
			&TemperatureThreshold,
		),
		(With<Fever>, With<CharacterController>),
	>,
	mut q_layers: Query<
		(
			&mut InstanceColorGradient,
			&mut InstanceColor,
			&mut SubsurfaceScatteringIntensity,
		),
		With<ScatterLayer>,
	>,
	mut q_materials: Query<&mut InstanceMaterialData>,
) {
	let (current, max, base, _) = fever.into_inner();
	let range = (**max - **base).max(0.0001);
	let fever = ((**current - **base) / range).clamp(0.0, 1.0);
	let palette = FeverColorPalette::default();
	let (base_color, gradient_start, gradient_end) = palette.get_colors(fever);

	for mut material in &mut q_materials {
		material.color = LinearRgba::from(base_color);
	}

	for (mut gradient, mut color, mut sss) in &mut q_layers {
		**color = base_color.into();
		**sss = (fever * 2.).into();
		*gradient = InstanceColorGradient {
			start: 4.,
			..InstanceColorGradient::new(gradient_start, gradient_end)
		};
	}
}

#[derive(Debug, Clone)]
pub struct FeverColorPalette {
	pub base: Srgba,
	pub gradient_start: Srgba,
	pub gradient_end: Srgba,

	pub fever_dark: Srgba,
	pub fever_pink: Srgba,
	pub fever_purple: Srgba,
}

impl Default for FeverColorPalette {
	fn default() -> Self {
		Self {
			base: Srgba::hex("#1f3114").unwrap(),
			gradient_start: Srgba::hex("#3e6328").unwrap(),
			gradient_end: Srgba::hex("#0f190a").unwrap(),
			fever_dark: Srgba::rgb_u8(64, 27, 18),
			fever_pink: Srgba::rgb_u8(255, 97, 117),
			fever_purple: Srgba::rgb_u8(101, 13, 112),
		}
	}
}

impl FeverColorPalette {
	pub fn get_base_color(&self, fever: f32) -> Srgba {
		let fever = fever.clamp(0.0, 1.0);
		Self::lerp_color(self.base, self.fever_dark, fever)
	}

	pub fn get_gradient_colors(&self, fever: f32) -> (Srgba, Srgba) {
		let fever = fever.clamp(0.0, 1.0);
		let gradient_start = Self::lerp_color(self.gradient_start, self.fever_pink, fever);
		let gradient_end = Self::lerp_color(self.gradient_end, self.fever_purple, fever);
		(gradient_start, gradient_end)
	}

	pub fn get_colors(&self, fever: f32) -> (Srgba, Srgba, Srgba) {
		let base = self.get_base_color(fever);
		let (start, end) = self.get_gradient_colors(fever);
		(base, start, end)
	}

	fn lerp_color(a: Srgba, b: Srgba, t: f32) -> Srgba {
		Srgba {
			red: a.red + (b.red - a.red) * t,
			green: a.green + (b.green - a.green) * t,
			blue: a.blue + (b.blue - a.blue) * t,
			alpha: a.alpha + (b.alpha - a.alpha) * t,
		}
	}
}
