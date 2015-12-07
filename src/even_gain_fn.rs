
use {Compressor, Detector};

/// Some function that yields a gain to be applied evenly across all channels in a single frame.
pub trait EvenGainFunction: Sized {
    /// Yield the gain to be applied to each channel for the given frame of samples.
    fn next_gain<D, I>(&mut Compressor<D, Self>, sample_per_channel: I) -> f32
        where D: Detector,
              I: Iterator<Item=f32>;
}


/// An [**EvenGainFunction**](./trait.EvenGainFunction) that yields the *average* between each of
/// the produced channel gains.
#[derive(Copy, Clone, Debug)]
pub enum Average {}

impl EvenGainFunction for Average {
    /// The next compressor gain for the frame.
    ///
    /// The returned gain is the *average* between each of the channel gains.
    ///
    /// **Note:** This method assumes that the given number of samples is equal to the number of
    /// channels with which the Compressor is currently set.
    ///
    /// **Returns NaN** if the given iterator is empty.
    ///
    /// **Panics** if the number of samples given is greater than the number of channels stored
    /// within the **Compressor**s envelope detector.
    #[inline]
    fn next_gain<D, I>(compressor: &mut Compressor<D, Self>, sample_per_channel: I) -> f32
        where D: Detector,
              I: Iterator<Item=f32>,
    {
        let mut sum = 0.0;
        let mut channel_idx = 0;
        for sample in sample_per_channel {
            sum += compressor.next_gain_for_channel(channel_idx, sample);
            channel_idx += 1;
        }
        sum / channel_idx as f32
    }
}


/// An [**EvenGainFunction**](./trait.EvenGainFunction) that yields the *minimum* between each of
/// the produced channel gains.
#[derive(Copy, Clone, Debug)]
pub enum Minimum {}

impl EvenGainFunction for Minimum {
    /// The next compressor gain for the frame.
    ///
    /// The returned gain is the *lowest* between each of the channel gains.
    ///
    /// **Note:** This method assumes that the given number of samples is equal to the number of
    /// channels with which the Compressor is currently set.
    ///
    /// If the iterator yields no samples, the returned gain is `1.0`.
    ///
    /// **Panics** if the number of samples given is greater than the number of channels stored
    /// within the **Compressor**s envelope detector.
    #[inline]
    fn next_gain<D, I>(compressor: &mut Compressor<D, Self>, sample_per_channel: I) -> f32
        where D: Detector,
              I: Iterator<Item=f32>,
    {
        let mut gain: f32 = 1.0;
        let mut channel_idx = 0;
        for sample in sample_per_channel {
            let channel_gain = compressor.next_gain_for_channel(channel_idx, sample);
            gain = gain.min(channel_gain);
            channel_idx += 1;
        }
        gain
    }
}
