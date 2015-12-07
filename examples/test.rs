//! Compress an input signal pass it to the output.

extern crate compressor;
extern crate dsp;
extern crate envelope_detector;

use compressor::{Compressor, Minimum};
use dsp::{CallbackFlags, CallbackResult, Node, Settings, SoundStream, StreamParams};

fn main() {

    const CHANNELS: u16 = 2;
    const SAMPLE_HZ: f64 = 44_100.0;
    const ATTACK_MS: f64 = 10.0;
    const RELEASE_MS: f64 = 10.0;
    const THRESHOLD: f32 = 0.1;
    const RATIO: f32 = 100.0;

    let n_channels = CHANNELS as usize;
    let mut compressor =
        Compressor::<Minimum>::peak(ATTACK_MS, RELEASE_MS, SAMPLE_HZ, n_channels, THRESHOLD, RATIO);

    // Callback used to construct the duplex sound stream.
    let callback = Box::new(move |input: &[f32], _in_settings: Settings,
                                  output: &mut[f32], out_settings: Settings,
                                  _dt: f64,
                                  _: CallbackFlags| {

        // Write the input to the output for fun.
        for (out_sample, in_sample) in output.iter_mut().zip(input.iter()) {
            *out_sample = *in_sample;
        }

        println!("");
        println!("{:?}", &output[0..4]);

        // Update our rms state.
        compressor.audio_requested(output, out_settings);

        println!("{:?}", &output[0..4]);

        CallbackResult::Continue
    });

    // Construct parameters for a duplex stream and the stream itself.
    let params = StreamParams::new().channels(CHANNELS as i32);
    let stream = SoundStream::new()
        .sample_hz(44_100.0)
        .frames_per_buffer(128)
        .duplex(params, params)
        .run_callback(callback)
        .unwrap();

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep_ms(16);
    }

}
