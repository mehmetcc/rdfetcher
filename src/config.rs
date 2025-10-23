#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub kafka: KafkaConfig,
    pub dispatch: DispatchConfig,
    pub concurrency: ConcurrencyConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("Config").required(false))
            .build()?;

        config.try_deserialize::<Config>().map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub topic: String,
    pub compression: String,
    pub acks: String,
    pub timeout_ms: u32,
    pub buffering_max_ms: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DispatchConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConcurrencyConfig {
    pub buffer_size: usize,
    pub mask_tasks: usize,
}
