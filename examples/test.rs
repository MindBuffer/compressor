//! Compress an input signal pass it to the output.

extern crate compressor;
extern crate dsp;
extern crate portaudio as pa;

use compressor::Compressor;
use dsp::Node;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {

    const CHANNELS: usize = 2;
    const SAMPLE_HZ: f64 = 44_100.0;
    const ATTACK_MS: f64 = 1_000.0;
    const RELEASE_MS: f64 = 1_000.0;
    const THRESHOLD: f32 = 0.1;
    const RATIO: f32 = 100.0;

    let mut compressor = Compressor::peak_min(ATTACK_MS, RELEASE_MS, SAMPLE_HZ, THRESHOLD, RATIO);

    // Callback used to construct the duplex sound stream.
    let callback = move |pa::DuplexStreamCallbackArgs { in_buffer, out_buffer, .. }| {
        let in_buffer: &[[f32; CHANNELS]] = dsp::slice::to_frame_slice(in_buffer).unwrap();
        let out_buffer: &mut [[f32; CHANNELS]] = dsp::slice::to_frame_slice_mut(out_buffer).unwrap();

        // Write the input to the output for fun.
        dsp::slice::write(out_buffer, in_buffer);

        println!("");
        println!("{:?}", &out_buffer[0..4]);

        // Process the buffer with our compressor.
        compressor.audio_requested(out_buffer, SAMPLE_HZ);

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
        ::std::thread::sleep(::std::time::Duration::from_millis(16));
    }

    Ok(())
}
