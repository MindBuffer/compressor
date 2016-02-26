
extern crate dsp;

use {Compressor, Detector, EvenGainFunction, PeakCompressor, RmsCompressor};
use self::dsp::{FromSample, Sample};


impl<D, F> Compressor<D, F> {

    /// Compresses the given `output` using a unique gain per channel.
    #[inline]
    pub fn compress_per_channel<S>(&mut self, output: &mut [S], n_frames: usize, n_channels: usize)
        where S: Sample + FromSample<f32>,
              D: Detector,
              F: EvenGainFunction,
              f32: FromSample<S>,
    {
        let mut idx = 0;
        for _ in 0..n_frames {
            for j in 0..n_channels {
                let sample = output[idx].to_sample::<f32>();
                let gain = self.next_gain_for_channel(j, sample);
                let compressed_sample = sample * gain;
                output[idx] = compressed_sample.to_sample::<S>();
                idx += 1;
            }
        }
    }

    /// Compresses the given `output` using an even gain across all channels.
    #[inline]
    pub fn compress<S>(&mut self, output: &mut [S], n_frames: usize, n_channels: usize)
        where S: Sample + FromSample<f32>,
              D: Detector,
              F: EvenGainFunction,
              f32: FromSample<S>,
    {
        let mut idx = 0;
        for _ in 0..n_frames {
            let gain = {
                let end_idx = idx + n_channels;
                let slice = &output[idx..end_idx];
                let samples = slice.iter().map(|s| s.to_sample::<f32>());
                self.next_gain(samples)
            };
            for _ in 0..n_channels {
                let sample = output[idx].to_sample::<f32>();
                let compressed_sample = gain * sample;
                output[idx] = compressed_sample.to_sample::<S>();
                idx += 1;
            }
        }
    }

}


impl<S, F> dsp::Node<S> for PeakCompressor<F>
    where S: Sample + FromSample<f32>,
          F: EvenGainFunction,
          f32: FromSample<S>,
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
    where S: Sample + FromSample<f32>,
          F: EvenGainFunction,
          f32: FromSample<S>,
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
