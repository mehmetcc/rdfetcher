use crate::config::Config;
use crate::page::Page;
use rdkafka::producer::FutureProducer;
use rdkafka::{ClientConfig, producer::FutureRecord};
use std::time::{Duration, Instant};
use tokio::{select, sync::mpsc::Receiver};
use tracing::{error, info};

pub struct KafkaSink {
    config: Config,
    producer: FutureProducer,
}

impl KafkaSink {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load()?;
        let future_producer = KafkaSink::future_producer(config.clone())?;

        Ok(Self {
            config: config,
            producer: future_producer,
        })
    }

    pub async fn dispatch(&self, mut rx: Receiver<Page>) -> anyhow::Result<()> {
        let mut buffer: Vec<Page> = Vec::new();
        let mut last_flush = Instant::now();

        let flush_interval = Duration::from_millis(self.config.dispatch.flush_interval_ms);
        let batch_size = self.config.dispatch.batch_size;
        let topic = &self.config.kafka.topic;

        loop {
            select! {
                maybe_page = rx.recv() => {
                    match maybe_page {
                        Some(page) => {
                            buffer.push(page);
                            if buffer.len() >= batch_size {
                                self.flush(&mut buffer, topic).await?;
                                last_flush = Instant::now();
                            }
                        }
                        None => {
                            if !buffer.is_empty() {
                                self.flush(&mut buffer, topic).await?;
                            }
                            info!("KafkaSink: input channel closed, exiting dispatcher");
                            break;
                        }
                    }
                }

                _ = tokio::time::sleep_until((last_flush + flush_interval).into()) => {
                    if !buffer.is_empty() {
                        self.flush(&mut buffer, topic).await?;
                        last_flush = Instant::now();
                    }
                }
            }
        }

        Ok(())
    }

    async fn flush(&self, buffer: &mut Vec<Page>, topic: &str) -> anyhow::Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let count = buffer.len();
        info!("Flushing {} pages to Kafka topic '{}'", count, topic);

        for page in buffer.drain(..) {
            let payload = serde_json::to_string(&page)?;
            let key = page.url.full_url()?;
            let record = FutureRecord::to(topic).payload(&payload).key(&key);

            if let Err((err, _)) = self.producer.send(record, Duration::from_secs(0)).await {
                error!("Failed to send record to Kafka: {:?}", err);
            }
        }

        Ok(())
    }

    fn future_producer(config: Config) -> anyhow::Result<FutureProducer> {
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", config.kafka.brokers)
            .set("compression.type", config.kafka.compression)
            .set("acks", config.kafka.acks)
            .set("message.timeout.ms", config.kafka.timeout_ms.to_string())
            .set(
                "queue.buffering.max.ms",
                config.kafka.buffering_max_ms.to_string(),
            )
            .set("message.max.bytes", "10000000"); // TODO: configure

        client_config.create().map_err(|e| e.into())
    }
}
