//! Demonstrates how to handle stream errors.

use rtaudio::{Buffers, StreamConfig, StreamInfo, StreamStatus};
use std::time::{Duration, Instant};

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish(),
    )
    .unwrap();

    // Set the global error handling callback that will be called whenever
    // there is an error that causes an audio stream to close. When an
    // error is received, all streams from all hosts should be manually
    // closed or dropped.
    //
    // Note, RtAudio provides no way to tell which host/stream the error
    // originates from. So prefer to stop all existing streams when an
    // error is received.
    let (error_tx, error_rx) = std::sync::mpsc::channel();
    rtaudio::set_error_callback(move |error| error_tx.send(error).unwrap());

    let host = rtaudio::Host::default();

    let mut stream_handle = host.open_stream(&StreamConfig::default()).unwrap();

    stream_handle
        .start(
            move |buffers: Buffers<'_>, _info: &StreamInfo, _status: StreamStatus| {
                if let Buffers::Float32 { output, input: _ } = buffers {
                    // Fill the output with silence.
                    output.fill(0.0);
                }
            },
        )
        .unwrap();

    // Play for 5 seconds and then close.
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(5) {
        // Periodically poll to see if an error has happened.
        if let Ok(error) = error_rx.try_recv() {
            // An error occured that caused the stream to close (for example a
            // device was unplugged). Now our stream_handle object should be
            // manually closed or dropped.
            eprintln!("{}", error);

            break;
        }

        std::thread::sleep(Duration::from_millis(16));
    }
}
