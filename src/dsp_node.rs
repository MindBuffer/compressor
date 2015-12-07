
extern crate dsp;

use {Compressor, Detector, EvenGainFunction, PeakCompressor, RmsCompressor};
use self::dsp::Sample;


impl<D, F> Compressor<D, F> {

    /// Compresses the given `output` using a unique gain per channel.
    #[inline]
    pub fn compress_per_channel<S>(&mut self, output: &mut [S], n_frames: usize, n_channels: usize)
        where S: Sample,
              D: Detector,
              F: EvenGainFunction,
    {
        let mut idx = 0;
        for _ in 0..n_frames {
            for j in 0..n_channels {
                let sample = output[idx].to_wave();
                let gain = self.next_gain_for_channel(j, sample);
                let compressed_sample = sample * gain;
                output[idx] = Sample::from_wave(compressed_sample);
                idx += 1;
            }
        }
    }

    /// Compresses the given `output` using an even gain across all channels.
    #[inline]
    pub fn compress<S>(&mut self, output: &mut [S], n_frames: usize, n_channels: usize)
        where S: Sample,
              D: Detector,
              F: EvenGainFunction,
    {
        let mut idx = 0;
        for _ in 0..n_frames {
            let gain = {
                let end_idx = idx + n_channels;
                let slice = &output[idx..end_idx];
                let samples = slice.iter().map(|s| s.to_wave());
                self.next_gain(samples)
            };
            for _ in 0..n_channels {
                let sample = output[idx].to_wave();
                let compressed_sample = gain * sample;
                output[idx] = Sample::from_wave(compressed_sample);
                idx += 1;
            }
        }
    }

}


impl<S, F> dsp::Node<S> for PeakCompressor<F>
    where S: Sample,
          F: EvenGainFunction,
{
    #[inline]
    fn audio_requested(&mut self, output: &mut [S], settings: dsp::Settings) {
        let sample_hz = settings.sample_hz as f64;
        let n_channels = settings.channels as usize;
        self.update_attack_to_sample_hz(sample_hz);
        self.update_release_to_sample_hz(sample_hz);
        self.set_channels(n_channels);
        self.compress(output, settings.frames as usize, n_channels);
    }
}


impl<S, F> dsp::Node<S> for RmsCompressor<F>
    where S: Sample,
          F: EvenGainFunction,
{
    #[inline]
    fn audio_requested(&mut self, output: &mut [S], settings: dsp::Settings) {
        let sample_hz = settings.sample_hz as f64;
        let n_channels = settings.channels as usize;
        self.update_attack_to_sample_hz(sample_hz);
        self.update_release_to_sample_hz(sample_hz);
        self.update_window_to_sample_hz(sample_hz);
        self.set_channels(n_channels);
        self.compress(output, settings.frames as usize, n_channels);
    }
}
