pub fn init() {
    if std::env::var("RUST_LOG").is_err() {
        let mut builder = pretty_env_logger::formatted_builder();
        builder.filter(None, log::LevelFilter::Info);
        builder.init();
    } else {
        pretty_env_logger::init();
    }
}
