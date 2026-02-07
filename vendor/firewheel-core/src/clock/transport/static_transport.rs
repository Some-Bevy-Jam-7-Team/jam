use crate::clock::{
    beats_per_second, seconds_per_beat, DurationMusical, DurationSeconds, InstantMusical,
    InstantSamples, InstantSeconds, ProcTransportInfo,
};
use core::num::NonZeroU32;

/// A musical transport with a single static tempo in beats per minute.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StaticTransport {
    pub beats_per_minute: f64,
}

impl Default for StaticTransport {
    fn default() -> Self {
        Self {
            beats_per_minute: 110.0,
        }
    }
}

impl StaticTransport {
    pub const fn new(beats_per_minute: f64) -> Self {
        Self { beats_per_minute }
    }

    pub fn seconds_per_beat(&self, speed_multiplier: f64) -> f64 {
        seconds_per_beat(self.beats_per_minute, speed_multiplier)
    }

    pub fn beats_per_second(&self, speed_multiplier: f64) -> f64 {
        beats_per_second(self.beats_per_minute, speed_multiplier)
    }

    pub fn musical_to_seconds(
        &self,
        musical: InstantMusical,
        transport_start: InstantSeconds,
        speed_multiplier: f64,
    ) -> InstantSeconds {
        transport_start + DurationSeconds(musical.0 * self.seconds_per_beat(speed_multiplier))
    }

    pub fn musical_to_samples(
        &self,
        musical: InstantMusical,
        transport_start: InstantSamples,
        speed_multiplier: f64,
        sample_rate: NonZeroU32,
    ) -> InstantSamples {
        transport_start
            + DurationSeconds(musical.0 * self.seconds_per_beat(speed_multiplier))
                .to_samples(sample_rate)
    }

    pub fn seconds_to_musical(
        &self,
        seconds: InstantSeconds,
        transport_start: InstantSeconds,
        speed_multiplier: f64,
    ) -> InstantMusical {
        InstantMusical((seconds - transport_start).0 * self.beats_per_second(speed_multiplier))
    }

    pub fn samples_to_musical(
        &self,
        sample_time: InstantSamples,
        transport_start: InstantSamples,
        speed_multiplier: f64,
        sample_rate: NonZeroU32,
        sample_rate_recip: f64,
    ) -> InstantMusical {
        InstantMusical(
            (sample_time - transport_start)
                .to_seconds(sample_rate, sample_rate_recip)
                .0
                * self.beats_per_second(speed_multiplier),
        )
    }

    pub fn delta_seconds_from(
        &self,
        from: InstantMusical,
        delta_seconds: DurationSeconds,
        speed_multiplier: f64,
    ) -> InstantMusical {
        from + DurationMusical(delta_seconds.0 * self.beats_per_second(speed_multiplier))
    }

    pub fn bpm_at_musical(&self, _musical: InstantMusical, speed_multiplier: f64) -> f64 {
        self.beats_per_minute * speed_multiplier
    }

    pub fn transport_start(
        &self,
        now: InstantSamples,
        playhead: InstantMusical,
        speed_multiplier: f64,
        sample_rate: NonZeroU32,
    ) -> InstantSamples {
        now - DurationSeconds(playhead.0 * self.seconds_per_beat(speed_multiplier))
            .to_samples(sample_rate)
    }

    pub fn proc_transport_info(&self, frames: usize, speed_multiplier: f64) -> ProcTransportInfo {
        ProcTransportInfo {
            frames,
            beats_per_minute: self.beats_per_minute * speed_multiplier,
        }
    }
}
