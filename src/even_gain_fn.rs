
use {Compressor, Detector};
use envelope_detector::{Sample, Frame};

/// Some function that yields a gain to be applied evenly across all channels in a single frame.
pub trait EvenGainFunction: Sized {
    /// Yield the gain to be applied to each channel for the given frame of samples.
    fn next_gain<F, D>(&mut Compressor<F, D, Self>, frame: F) -> <F::Sample as Sample>::Float
        where F: Frame,
              D: Detector<F>;
}


/// An [**EvenGainFunction**](./trait.EvenGainFunction) that yields the *average* between each of
/// the produced channel gains.
#[derive(Copy, Clone, Debug)]
pub enum Average {}

impl EvenGainFunction for Average {
    /// The next compressor gain for the `Frame`.
    ///
    /// The returned gain is the *average* between each of the channel gains.
    #[inline]
    fn next_gain<F, D>(compressor: &mut Compressor<F, D, Self>, frame: F) -> <F::Sample as Sample>::Float
        where F: Frame,
              D: Detector<F>,
    {
        let next_frame = compressor.next_gain_per_channel(frame);
        let sum: <F::Sample as Sample>::Float =
            next_frame.channels().fold(Sample::equilibrium(), |s, ch_gain| s + ch_gain);
        sum / (F::n_channels() as f32).to_sample()
    }
}


/// An [**EvenGainFunction**](./trait.EvenGainFunction) that yields the *minimum* between each of
/// the produced channel gains.
#[derive(Copy, Clone, Debug)]
pub enum Minimum {}

impl EvenGainFunction for Minimum {
    /// The next compressor gain for the `Frame`.
    ///
    /// The returned gain is the *lowest* between each of the channel gains.
    #[inline]
    fn next_gain<F, D>(compressor: &mut Compressor<F, D, Self>, frame: F) -> <F::Sample as Sample>::Float
        where F: Frame,
              D: Detector<F>,
    {
        let next_frame = compressor.next_gain_per_channel(frame);
        let one = <F::Sample as Sample>::identity();
        next_frame.channels().fold(one, |min, ch_gain| if ch_gain < min { ch_gain } else { min })
    }
}
