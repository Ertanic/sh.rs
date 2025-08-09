use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logs() -> tracing_appender::non_blocking::WorkerGuard {
    #[cfg(debug_assertions)]
    let env = EnvFilter::new("server=trace");
    #[cfg(not(debug_assertions))]
    let env = EnvFilter::new("server=info");

    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .expect("Failed to parse time format");
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(offset, format);

    let logs_folder = get_logs_folder();
    let file_writer = tracing_appender::rolling::never(logs_folder, "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_writer);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_timer(timer.clone())
        .with_ansi(false)
        .with_writer(non_blocking);

    let stdout_layer = tracing_subscriber::fmt::layer().with_timer(timer);

    tracing_subscriber::registry()
        .with(env)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    _guard
}

fn get_logs_folder() -> PathBuf {
    std::env::current_exe()
        .expect("Failed to get current exe")
        .parent()
        .expect("Failed to get current exe parent")
        .to_path_buf()
}
