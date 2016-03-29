
extern crate dsp;

use {Compressor, Detector, EvenGainFunction, PeakCompressor, RmsCompressor};


impl<F, D, EFG> Compressor<F, D, EFG>
    where F: dsp::Frame,
          D: Detector<F>,
          EFG: EvenGainFunction,
{
    /// Compresses the given `output` using an even gain across all channels.
    #[inline]
    pub fn compress_slice(&mut self, frames: &mut [F]) {
        dsp::slice::map_in_place(frames, |f| self.next_frame(f));
    }
}


impl<F, EGF> dsp::Node<F> for PeakCompressor<F, EGF>
    where F: dsp::Frame,
          EGF: EvenGainFunction,
{
    fn audio_requested(&mut self, output: &mut [F], sample_hz: f64) {
        self.update_attack_to_sample_hz(sample_hz);
        self.update_release_to_sample_hz(sample_hz);
        self.compress_slice(output);
    }
}

impl<F, EGF> dsp::Node<F> for RmsCompressor<F, EGF>
    where F: dsp::Frame,
          EGF: EvenGainFunction,
{
    fn audio_requested(&mut self, output: &mut [F], sample_hz: f64) {
        self.update_attack_to_sample_hz(sample_hz);
        self.update_release_to_sample_hz(sample_hz);
        self.update_window_to_sample_hz(sample_hz);
        self.compress_slice(output);
    }
}
