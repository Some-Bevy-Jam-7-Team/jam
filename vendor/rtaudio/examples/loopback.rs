use rtaudio::{Buffers, DeviceParams, StreamConfig, StreamInfo, StreamStatus};

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish(),
    )
    .unwrap();

    let host = rtaudio::Host::default();
    dbg!(host.api());

    let mut stream_handle = host
        .open_stream(&StreamConfig {
            output_device: Some(DeviceParams::default()),
            input_device: Some(DeviceParams::default()),
            ..Default::default()
        })
        .unwrap();
    dbg!(stream_handle.info());

    stream_handle
        .start(
            move |buffers: Buffers<'_>, _info: &StreamInfo, _status: StreamStatus| {
                if let Buffers::Float32 { output, input } = buffers {
                    // Copy the input to the output.
                    output.copy_from_slice(input);
                }
            },
        )
        .unwrap();

    // Wait 3 seconds before closing.
    std::thread::sleep(std::time::Duration::from_millis(3000));
}
