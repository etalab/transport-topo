use std::io::Write;

pub fn init() {
    let log_level = std::env::var("RUST_LOG");
    let app_id = std::env::var("APP_ID");
    if log_level.is_err() || app_id.is_ok() {
        let mut builder = pretty_env_logger::formatted_builder();
        if let Ok(app_id) = app_id {
            // APP_ID is used to output an id while logging
            builder.format(move |buf, record| {
                writeln!(buf, "[{}] {} - {}", app_id, record.level(), record.args())
            });
        }

        if let Ok(s) = std::env::var("RUST_LOG") {
            builder.parse_filters(&s);
        } else {
            builder.filter(None, log::LevelFilter::Info);
        }
        builder.init();
    } else {
        pretty_env_logger::init();
    }
}
