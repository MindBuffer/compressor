use envelope_detector::{Frame, Mode, EnvelopeDetector, Peak, Rms};
use time::Ms;

/// A peak `Detector` type.
pub use envelope_detector::PeakEnvelopeDetector;


/// Envelope **Detector** types that may be used by the **Compressor**.
pub trait Detector<F>
    where F: Frame,
{
    /// The detection **Mode** used by the **Detector**.
    type Mode: Mode<F>;
    /// Mutably borrow the **MultiChannelEnvelopeDetector**.
    fn detector(&mut self) -> &mut EnvelopeDetector<F, Self::Mode>;
}


/// An RMS envelope detector with a window adjustable in milliseconds.
#[derive(Clone)]
pub struct RmsEnvelopeDetector<F>
    where F: Frame,
{
    /// The multi-channel RMS envelope detector.
    pub rms: EnvelopeDetector<F, Rms<F>>,
    /// The duration of the RMS window used by the detector.
    pub window_ms: Ms,
}


impl<F> Detector<F> for PeakEnvelopeDetector<F>
    where F: Frame,
{
    type Mode = Peak;
    fn detector(&mut self) -> &mut Self {
        self
    }
}

impl<F> Detector<F> for RmsEnvelopeDetector<F>
    where F: Frame,
{
    type Mode = Rms<F>;
    fn detector(&mut self) -> &mut EnvelopeDetector<F, Self::Mode> {
        &mut self.rms
    }
}

