
use envelope_detector::{Mode, MultiChannelEnvelopeDetector, Peak, Rms};
use time::Ms;


/// Envelope **Detector** types that may be used by the **Compressor**.
pub trait Detector {
    /// The detection **Mode** used by the **Detector**.
    type Mode: Mode;
    /// Mutably borrow the **MultiChannelEnvelopeDetector**.
    fn detector(&mut self) -> &mut MultiChannelEnvelopeDetector<Self::Mode>;
}


/// A multi-channel Peak envelope detector.
pub type PeakEnvelopeDetector = MultiChannelEnvelopeDetector<Peak>;

/// A multi-channel RMS envelope detector with a window adjustable in milliseconds.
#[derive(Clone, Debug)]
pub struct RmsEnvelopeDetector {
    /// The multi-channel RMS envelope detector.
    pub rms: MultiChannelEnvelopeDetector<Rms>,
    /// The duration of the RMS window used by the detector.
    pub window_ms: Ms,
}


impl Detector for PeakEnvelopeDetector {
    type Mode = Peak;
    fn detector(&mut self) -> &mut Self {
        self
    }
}

impl Detector for RmsEnvelopeDetector {
    type Mode = Rms;
    fn detector(&mut self) -> &mut MultiChannelEnvelopeDetector<Rms> {
        &mut self.rms
    }
}

