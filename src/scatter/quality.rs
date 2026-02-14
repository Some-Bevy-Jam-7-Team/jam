use bevy::prelude::{Reflect, Resource, default};
use bevy_feronia::prelude::{DistributionDensity, LodConfig};

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum QualitySetting {
	Low,
	#[default]
	Medium,
	High,
	Ultra,
}

impl QualitySetting {
	pub fn next(&self) -> Self {
		match self {
			QualitySetting::Low => QualitySetting::Medium,
			QualitySetting::Medium => QualitySetting::High,
			QualitySetting::High => QualitySetting::Ultra,
			QualitySetting::Ultra => QualitySetting::Low,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum GrassDensitySetting {
	Low,
	#[default]
	Medium,
	High,
	Ultra,
}

impl From<QualitySetting> for GrassDensitySetting {
	fn from(value: QualitySetting) -> Self {
		match value {
			QualitySetting::Low => GrassDensitySetting::Low,
			QualitySetting::Medium => GrassDensitySetting::Medium,
			QualitySetting::High => GrassDensitySetting::High,
			QualitySetting::Ultra => GrassDensitySetting::Ultra,
		}
	}
}

impl From<GrassDensitySetting> for DistributionDensity {
	fn from(value: GrassDensitySetting) -> Self {
		match value {
			GrassDensitySetting::Low => 160.into(),
			GrassDensitySetting::Medium => 180.into(),
			GrassDensitySetting::High => 200.into(),
			GrassDensitySetting::Ultra => 220.into(),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum MushroomDensitySetting {
	Low,
	Medium,
	#[default]
	High,
	Ultra,
}

impl From<QualitySetting> for MushroomDensitySetting {
	fn from(value: QualitySetting) -> Self {
		match value {
			QualitySetting::Low => MushroomDensitySetting::Low,
			QualitySetting::Medium => MushroomDensitySetting::Medium,
			QualitySetting::High => MushroomDensitySetting::High,
			QualitySetting::Ultra => MushroomDensitySetting::Ultra,
		}
	}
}

impl From<MushroomDensitySetting> for DistributionDensity {
	fn from(value: MushroomDensitySetting) -> Self {
		match value {
			MushroomDensitySetting::Low => 60.into(),
			MushroomDensitySetting::Medium => 80.into(),
			MushroomDensitySetting::High => 100.into(),
			MushroomDensitySetting::Ultra => 120.into(),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum RockDensitySetting {
	Low,
	Medium,
	#[default]
	High,
	Ultra,
}

impl From<QualitySetting> for RockDensitySetting {
	fn from(value: QualitySetting) -> Self {
		match value {
			QualitySetting::Low => RockDensitySetting::Low,
			QualitySetting::Medium => RockDensitySetting::Medium,
			QualitySetting::High => RockDensitySetting::High,
			QualitySetting::Ultra => RockDensitySetting::Ultra,
		}
	}
}

impl From<RockDensitySetting> for DistributionDensity {
	fn from(value: RockDensitySetting) -> Self {
		match value {
			RockDensitySetting::Low => 10.into(),
			RockDensitySetting::Medium => 20.into(),
			RockDensitySetting::High => 25.into(),
			RockDensitySetting::Ultra => 30.into(),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum VisibilityRangeQuality {
	Low,
	Medium,
	#[default]
	High,
	Ultra,
}

impl From<QualitySetting> for VisibilityRangeQuality {
	fn from(value: QualitySetting) -> Self {
		match value {
			QualitySetting::Low => VisibilityRangeQuality::Low,
			QualitySetting::Medium => VisibilityRangeQuality::Medium,
			QualitySetting::High => VisibilityRangeQuality::High,
			QualitySetting::Ultra => VisibilityRangeQuality::Ultra,
		}
	}
}

impl From<VisibilityRangeQuality> for LodConfig {
	fn from(quality: VisibilityRangeQuality) -> Self {
		match quality {
			VisibilityRangeQuality::Low => Self {
				distance: vec![20.0.into(), default()],
				density: vec![1.0.into(), 0.3.into()],
			},
			VisibilityRangeQuality::Medium => Self {
				distance: vec![20.0.into(), 50.0.into(), default()],
				density: vec![1.0.into(), 0.3.into(), 0.1.into()],
			},
			VisibilityRangeQuality::High => default(),
			VisibilityRangeQuality::Ultra => Self {
				distance: vec![40.0.into(), 120.0.into(), 360.0.into(), default()],
				density: vec![1.0.into(), 0.3.into(), 0.1.into(), default()],
			},
		}
	}
}
