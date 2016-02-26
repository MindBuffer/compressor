//! A digital signal compressor, designed for use with audio.
//!
//! The main type of interest is the [**Compressor**](./struct.Compressor).
//!
//! You may also find the [**EvenGainFunction**](./even_gain_fn/trait.EvenGainFunction) trait
//! (implemented for both [**Average**](./even_gain_fn/enum.Average) and
//! [**Minimum**](./even_gain_fn/enum.Minimum)) and the [**Detector**](./detector/trait.Detector)
//! trait (implemented for [**PeakEnvelopeDetector**](./detector/type.PeakEnvelopeDetector) and
//! [**RmsEnvelopeDetector**](./detector/struct.RmsEnvelopeDetector).

#[deny(missing_copy_implementations)]
#[deny(missing_docs)]

extern crate envelope_detector;
extern crate time_calc as time;

use envelope_detector::MultiChannelEnvelopeDetector;
use std::marker::PhantomData;
use time::Ms;

pub mod detector;
pub mod even_gain_fn;

#[cfg(feature = "dsp-chain")]
pub mod dsp_node;


pub use detector::{Detector, PeakEnvelopeDetector, RmsEnvelopeDetector};
pub use even_gain_fn::{EvenGainFunction, Average, Minimum};


/// A dynamics processing unit designed to compress some given audio signal that exceeds the
/// `threshold` using the `ratio`.
///
/// The **Compressor** is generic over its envelope [**Detector**](./detector/trait.Detector) and
/// the [**EvenGainFunction**](./even_gain_fn/trait.EvenGainFunction) (used to determine the gain
/// that will be applied evenly to all channels for a single frame).
#[derive(Clone, Debug)]
pub struct Compressor<D, F> {
    /// The **EnvelopeDetector** used to create a "loudness" envelope.
    envelope_detector: D,
    /// The envelope attack duration in milliseconds.
    attack_ms: Ms,
    /// The envelope release duration in milliseconds.
    release_ms: Ms,
    /// When the detected envelope exceeds this threshold, the signal is compressed via the `ratio`.
    pub threshold: f32,
    /// The slope of the `ratio`, used to calculate the compressor_gain.
    ///
    /// The ratio is the amount at which we compress the signal once the envelope exceeds the
    /// `threshold`.
    ///
    /// *ratio of 4.0 == 4:1 == compress by every 4 parts of the exceeding envelope to 1 == slope
    /// of 0.75.*
    slope: f32,
    /// Some function that yields a gain to be applied evenly across all channels in a single
    /// frame.
    even_gain_fn: PhantomData<F>,
}

/// A **Compressor** that uses a **Peak** envelope detector.
pub type PeakCompressor<F> = Compressor<PeakEnvelopeDetector, F>;
/// A **Compressor** that uses an **Rms** envelope detector.
pub type RmsCompressor<F> = Compressor<RmsEnvelopeDetector, F>;


fn calc_slope(ratio: f32) -> f32 {
    1.0 - (1.0 / ratio)
}


impl<D, F> Compressor<D, F> where D: Detector {

    /// Set the duration of the envelope's attack in milliseconds.
    pub fn set_attack_ms<M: Into<Ms>>(&mut self, ms: M, sample_hz: f64) {
        let ms: Ms = ms.into();
        self.attack_ms = ms;
        self.update_attack_to_sample_hz(sample_hz);
    }

    /// Set the duration of the envelope's release in milliseconds.
    pub fn set_release_ms<M: Into<Ms>>(&mut self, ms: M, sample_hz: f64) {
        let ms: Ms = ms.into();
        self.release_ms = ms;
        self.update_release_to_sample_hz(sample_hz);
    }

    /// Updates the **Compressor**'s `attack` gain in accordance with the current sample_hz.
    pub fn update_attack_to_sample_hz(&mut self, sample_hz: f64) {
        let frames = self.attack_ms.samples(sample_hz) as f32;
        self.envelope_detector.detector().set_attack_frames(frames);
    }

    /// Updates the **Compressor**'s `release` gain in accordance with the current sample_hz.
    pub fn update_release_to_sample_hz(&mut self, sample_hz: f64) {
        let frames = self.release_ms.samples(sample_hz) as f32;
        self.envelope_detector.detector().set_release_frames(frames);
    }

}

impl<F> PeakCompressor<F> {

    /// Construct a **Compressor** that uses a **Peak** **EnvelopeDetector**.
    pub fn new<A, R>(attack_ms: A,
                     release_ms: R,
                     sample_hz: f64,
                     n_channels: usize,
                     threshold: f32,
                     ratio: f32) -> Self
        where A: Into<Ms>,
              R: Into<Ms>,
    {
        let attack_ms: Ms = attack_ms.into();
        let release_ms: Ms = release_ms.into();
        let attack_frames = attack_ms.samples(sample_hz) as f32;
        let release_frames = release_ms.samples(sample_hz) as f32;
        let envelope_detector =
            MultiChannelEnvelopeDetector::peak(attack_frames, release_frames, n_channels);
        let slope = calc_slope(ratio);
        Compressor {
            envelope_detector: envelope_detector,
            attack_ms: attack_ms,
            release_ms: release_ms,
            threshold: threshold,
            slope: slope,
            even_gain_fn: PhantomData,
        }
    }

}

impl<F> RmsCompressor<F> {

    /// Construct a **Compressor** that uses an **Rms** **EnvelopeDetector**.
    pub fn new<W, A, R>(window_ms: W,
                        attack_ms: A,
                        release_ms: R,
                        sample_hz: f64,
                        n_channels: usize,
                        threshold: f32,
                        ratio: f32) -> Self
        where W: Into<Ms>,
              A: Into<Ms>,
              R: Into<Ms>,
    {
        let window_ms: Ms = window_ms.into();
        let attack_ms: Ms = attack_ms.into();
        let release_ms: Ms = release_ms.into();
        let window_frames = window_ms.samples(sample_hz) as usize;
        let attack_frames = attack_ms.samples(sample_hz) as f32;
        let release_frames = release_ms.samples(sample_hz) as f32;
        let envelope_detector = MultiChannelEnvelopeDetector::rms(window_frames,
                                                                  attack_frames,
                                                                  release_frames,
                                                                  n_channels);
        let rms_envelope_detector = RmsEnvelopeDetector {
            rms: envelope_detector,
            window_ms: window_ms,
        };
        let slope = calc_slope(ratio);
        Compressor {
            envelope_detector: rms_envelope_detector,
            attack_ms: attack_ms,
            release_ms: release_ms,
            threshold: threshold,
            slope: slope,
            even_gain_fn: PhantomData,
        }
    }

    /// Set the duration of the envelope's RMS window in milliseconds.
    pub fn set_window_ms<M: Into<Ms>>(&mut self, ms: M, sample_hz: f64) {
        let ms: Ms = ms.into();
        self.envelope_detector.window_ms = ms;
        self.update_window_to_sample_hz(sample_hz);
    }

    /// Updates the **Compressor**'s window size in frames via the given sample_hz.
    pub fn update_window_to_sample_hz(&mut self, sample_hz: f64) {
        let frames = self.envelope_detector.window_ms.samples(sample_hz) as usize;
        self.envelope_detector.rms.set_window_frames(frames);
    }

}


impl<D, F> Compressor<D, F> {

    /// The next compressor gain for the channel at the given index.
    ///
    /// **Panics** if the given `channel_idx` is greater than the number of channels within the
    /// **Compressor**'s `envelope_detector`.
    #[inline]
    pub fn next_gain_for_channel(&mut self, channel_idx: usize, sample: f32) -> f32
        where D: Detector,
    {
        let env_sample = self.envelope_detector.detector().next(channel_idx, sample);
        if env_sample > self.threshold {
            1.0 - (env_sample - self.threshold) * self.slope
        } else {
            1.0
        }
    }

    /// Produce the gain to be applied evenly across all channels for the next frame.
    ///
    /// **Note:** This method assumes that the given number of samples is equal to the number of
    /// channels with which the Compressor is currently set.
    ///
    /// **Panics** if the given `channel_idx` is greater than the number of channels within the
    /// **Compressor**'s `envelope_detector`.
    pub fn next_gain<I>(&mut self, sample_per_channel: I) -> f32
        where D: Detector,
              I: Iterator<Item=f32>,
              F: EvenGainFunction,
    {
        F::next_gain(self, sample_per_channel)
    }

    /// Set the number of channels for the **MultiChannelEnvelopeDetector**.
    pub fn set_channels(&mut self, n_channels: usize)
        where D: Clone + Detector,
              D::Mode: Clone,
    {
        self.envelope_detector.detector().set_channels(n_channels);
    }

}
