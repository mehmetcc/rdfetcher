use tracing_subscriber::{fmt, prelude::*};

pub fn init_logger() {
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
