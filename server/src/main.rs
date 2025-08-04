mod logs;

fn main() {
    let _guard = logs::init_logs();

    tracing::info!("Hello, world!");
}