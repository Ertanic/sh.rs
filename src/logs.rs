use crate::config::LogsConfig;
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logs(config: LogsConfig) -> tracing_appender::non_blocking::WorkerGuard {
    let env = if config.debug {
        EnvFilter::new("server=trace")
    } else {
        EnvFilter::new("server=info")
    };

    let time_format = config
        .time_format
        .map(Box::new)
        .map(Box::leak)
        .map_or("[year]-[month]-[day] [hour]:[minute]:[second]", |v| v);

    let format = time::format_description::parse(time_format).expect("Failed to parse time format");
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(offset, format);

    let logs_folder = config
        .folder
        .map(PathBuf::from)
        .unwrap_or_else(get_logs_folder);

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
