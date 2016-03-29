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

use envelope_detector::{EnvelopeDetector, Frame, Sample};
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
pub struct Compressor<F, D, EGF> {
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
    even_gain_fn: PhantomData<EGF>,
    frame: PhantomData<F>,
}

/// A **Compressor** that uses a **Peak** envelope detector.
pub type PeakCompressor<F, EGF> = Compressor<F, PeakEnvelopeDetector<F>, EGF>;
/// A **Compressor** that uses the average across channels yielded by a **Peak** envelope detector.
pub type PeakAvgCompressor<F> = PeakCompressor<F, Average>;
/// A **Compressor** that uses the minimum across channels yielded by a **Peak** envelope detector.
pub type PeakMinCompressor<F> = PeakCompressor<F, Minimum>;

/// A **Compressor** that uses an **Rms** envelope detector.
pub type RmsCompressor<F, EGF> = Compressor<F, RmsEnvelopeDetector<F>, EGF>;
/// A **Compressor** that uses the average across channels yielded by a **Rms** envelope detector.
pub type RmsAvgCompressor<F> = RmsCompressor<F, Average>;
/// A **Compressor** that uses the minimum across channels yielded by a **Rms** envelope detector.
pub type RmsMinCompressor<F> = RmsCompressor<F, Minimum>;


fn calc_slope(ratio: f32) -> f32 {
    1.0 - (1.0 / ratio)
}


impl<F, D, EGF> Compressor<F, D, EGF>
    where F: Frame,
          D: Detector<F>,
          EGF: EvenGainFunction,
{

    /// Construct a new `Compressor` from its parts.
    ///
    /// This is a private constructor wrapped by the more specific `rms` and `peak` public
    /// constructors.
    fn new(detector: D, attack_ms: Ms, release_ms: Ms, threshold: f32, ratio: f32) -> Self {
        let slope = calc_slope(ratio);
        Compressor {
            envelope_detector: detector,
            attack_ms: attack_ms,
            release_ms: release_ms,
            threshold: threshold,
            slope: slope,
            even_gain_fn: std::marker::PhantomData,
            frame: std::marker::PhantomData,
        }
    }

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

    /// Steps forward the detectors using the given frame and determines the gain per-channel,
    /// yielding the result as a `Frame`.
    pub fn next_gain_per_channel(&mut self, next_frame: F) -> F::Float {
        let threshold = self.threshold.to_sample();
        let slope = self.slope.to_sample();
        let identity = <F::Sample as Sample>::identity();
        let env_frame = self.envelope_detector.detector().next(next_frame).to_float_frame();
        env_frame.map(|s| {
            let s = if s > identity { identity } else { s }; // Clamp `s` between 0.0...1.0.
            if s > threshold { identity - (s - threshold) * slope } else { identity }
        })
    }

    /// Produce the gain to be applied evenly across all channels for the next frame.
    #[inline]
    pub fn next_gain(&mut self, next_frame: F) -> <F::Sample as Sample>::Float {
        EGF::next_gain(self, next_frame)
    }

    /// Steps forward the `Compressor` by the given frame and returns the compressed result.
    #[inline]
    pub fn next_frame(&mut self, next_frame: F) -> F {
        let gain = self.next_gain(next_frame);
        println!("gain: {:?}", gain.to_sample::<f32>());
        next_frame.scale_amp(gain)
    }

}

impl<F, EGF> PeakCompressor<F, EGF>
    where F: Frame,
          EGF: EvenGainFunction,
{

    /// Construct a **Compressor** that uses a **Peak** **EnvelopeDetector**.
    pub fn peak<A, R>(attack_ms: A,
                      release_ms: R,
                      sample_hz: f64,
                      threshold: f32,
                      ratio: f32) -> Self
        where A: Into<Ms>,
              R: Into<Ms>,
    {
        let attack_ms: Ms = attack_ms.into();
        let release_ms: Ms = release_ms.into();
        let attack_frames = attack_ms.samples(sample_hz) as f32;
        let release_frames = release_ms.samples(sample_hz) as f32;
        let envelope_detector = EnvelopeDetector::peak(attack_frames, release_frames);
        Compressor::new(envelope_detector, attack_ms, release_ms, threshold, ratio)
    }

}

impl<F> PeakAvgCompressor<F>
    where F: Frame,
{

    /// Construct a **Compressor** that uses the **Average** across all channels yielded by a
    /// **Peak** **EnvelopeDetector**
    pub fn peak_avg<A, R>(attack_ms: A,
                          release_ms: R,
                          sample_hz: f64,
                          threshold: f32,
                          ratio: f32) -> Self
        where A: Into<Ms>,
              R: Into<Ms>,
    {
        Self::peak(attack_ms, release_ms, sample_hz, threshold, ratio)
    }

}

impl<F> PeakMinCompressor<F>
    where F: Frame,
{

    /// Construct a **Compressor** that uses the **Minimum** across all channels yielded by a
    /// **Peak** **EnvelopeDetector**
    pub fn peak_min<A, R>(attack_ms: A,
                          release_ms: R,
                          sample_hz: f64,
                          threshold: f32,
                          ratio: f32) -> Self
        where A: Into<Ms>,
              R: Into<Ms>,
    {
        Self::peak(attack_ms, release_ms, sample_hz, threshold, ratio)
    }

}

impl<F, EGF> RmsCompressor<F, EGF>
    where F: Frame,
          EGF: EvenGainFunction,
{

    /// Construct a **Compressor** that uses an **Rms** **EnvelopeDetector**.
    pub fn rms<W, A, R>(window_ms: W,
                        attack_ms: A,
                        release_ms: R,
                        sample_hz: f64,
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
        let envelope_detector = EnvelopeDetector::rms(window_frames, attack_frames, release_frames);
        let rms_envelope_detector = RmsEnvelopeDetector {
            rms: envelope_detector,
            window_ms: window_ms,
        };
        Compressor::new(rms_envelope_detector, attack_ms, release_ms, threshold, ratio)
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

impl<F> RmsAvgCompressor<F>
    where F: Frame,
{

    /// Construct a **Compressor** that uses the **Average** across all channels yielded by a
    /// **Rms** **EnvelopeDetector**
    pub fn rms_avg<W, A, R>(window_ms: W,
                            attack_ms: A,
                            release_ms: R,
                            sample_hz: f64,
                            threshold: f32,
                            ratio: f32) -> Self
        where W: Into<Ms>,
              A: Into<Ms>,
              R: Into<Ms>,
    {
        Self::rms(window_ms, attack_ms, release_ms, sample_hz, threshold, ratio)
    }

}

impl<F> RmsMinCompressor<F>
    where F: Frame,
{

    /// Construct a **Compressor** that uses the **Minimum** across all channels yielded by a
    /// **Rms** **EnvelopeDetector**
    pub fn rms_min<W, A, R>(window_ms: W,
                            attack_ms: A,
                            release_ms: R,
                            sample_hz: f64,
                            threshold: f32,
                            ratio: f32) -> Self
        where W: Into<Ms>,
              A: Into<Ms>,
              R: Into<Ms>,
    {
        Self::rms(window_ms, attack_ms, release_ms, sample_hz, threshold, ratio)
    }

}
