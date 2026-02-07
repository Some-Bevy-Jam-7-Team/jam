fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish(),
    )
    .unwrap();

    dbg!(rtaudio::version());

    for api in rtaudio::compiled_apis() {
        dbg!(api.get_display_name());

        match rtaudio::Host::new(api) {
            Ok(rt) => {
                for device_info in rt.devices() {
                    dbg!(device_info);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }

        println!("---------------------------------------------");
    }
}
