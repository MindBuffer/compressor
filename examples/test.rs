//! Compress an input signal pass it to the output.

extern crate compressor;
extern crate dsp;
extern crate envelope_detector;
extern crate portaudio as pa;

use compressor::{Compressor, Minimum};
use dsp::Node;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {

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
    let callback = move |pa::DuplexStreamCallbackArgs { in_buffer, out_buffer, frames, .. }| {

        // Write the input to the output for fun.
        for (out_sample, in_sample) in out_buffer.iter_mut().zip(in_buffer.iter()) {
            *out_sample = *in_sample;
        }

        println!("");
        println!("{:?}", &out_buffer[0..4]);

        // Update our rms state.
        let settings = dsp::Settings::new(SAMPLE_HZ as u32, frames as u16, CHANNELS as u16);
        compressor.audio_requested(out_buffer, settings);

        println!("{:?}", &out_buffer[0..4]);

        pa::Continue
    };

    // Construct PortAudio and the stream.
    const FRAMES: u32 = 128;
    let pa = try!(pa::PortAudio::new());
    let chans = CHANNELS as i32;
    let settings = try!(pa.default_duplex_stream_settings::<f32, f32>(chans, chans, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep_ms(16);
    }

    Ok(())
}
