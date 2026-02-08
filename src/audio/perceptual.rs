use bevy::prelude::*;
use bevy_seedling::prelude::*;

/// Constructor for taking a user-presented control value and converting it to a volume.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PerceptualVolumeConverter {
	/// When the perceptual control value is below this value, the mapping will be linear between:
	/// - 0 perceptual = 0 volume
	/// - [`Self::pivot_pos`] perceptual = [`Self::pivot_volume`] volume
	///
	/// When above this value, the mapping will be exponential between:
	/// - [`Self::pivot_pos`] perceptual = [`Self::pivot_volume`] volume
	/// - 1.0 perceptual = 0 dB
	pub(crate) pivot_pos: f32,
	/// The volume to use at [`Self::pivot_pos`]
	pub(crate) pivot_volume: Volume,
}
impl Default for PerceptualVolumeConverter {
	fn default() -> Self {
		Self {
			pivot_volume: Volume::Decibels(-50.0),
			pivot_pos: 0.01,
		}
	}
}

impl PerceptualVolumeConverter {
	/// Converts a user-presented control value in \[0.0, 1.0\] to a [`Volume`].
	pub(crate) fn to_volume(self, perceptual: f32) -> Volume {
		if perceptual < self.pivot_pos {
			let min = 0.0_f32;
			let max = self.pivot_volume.linear();
			let t = perceptual / self.pivot_pos;
			Volume::Linear(min.lerp(max, t))
		} else {
			let min = self.pivot_volume.decibels();
			let max = 0.0;
			let t = (perceptual - self.pivot_pos) / (1.0 - self.pivot_pos);
			Volume::Decibels(min.lerp(max, t))
		}
	}

	/// Converts a [`Volume`] into a user-presented control value in [0.0, 1.0].
	pub(crate) fn to_perceptual(self, volume: Volume) -> f32 {
		if volume.linear() <= self.pivot_volume.linear() {
			let vol = volume.linear();
			let pivot = self.pivot_volume.linear();
			let t = vol / pivot;
			t * self.pivot_pos
		} else {
			let vol = volume.decibels();
			let pivot = self.pivot_volume.decibels();
			let t = (vol - pivot) / (0.0 - pivot);
			self.pivot_pos + t * (1.0 - self.pivot_pos)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn round_trip() {
		let converter = PerceptualVolumeConverter::default();
		for i in 0..100 {
			let percent = i as f32 / 100.0;
			let volume = converter.to_volume(percent);
			let perceptual = converter.to_perceptual(volume);
			assert!((perceptual - percent).abs() < 0.0001);
		}
	}
}
