use std::time::{Duration, Instant};

#[derive(Default)]
enum JobStatus {
    Active,
    #[default]
    Waiting,
    Completed,
    Failed,
    Delayed,
}

#[derive(Default)]
pub enum RetryStrategy {
    #[default]
    Never,
    Constant(Duration),
    Map(fn(usize) -> Duration),
}

#[derive(Default)]
pub struct JobOptions {
    pub(crate) retry_strategy: RetryStrategy,
    pub(crate) max_retries: usize,
}

pub struct Job<DataType> {
    pub(crate) uuid: uuid::Uuid,
    pub(crate) creation_instant: Instant,
    pub(crate) last_run_instant: Instant,
    pub(crate) retries: usize,
    pub(crate) data: DataType,
    pub(crate) status: JobStatus,
    pub(crate) options: JobOptions,
}

impl<DataType> Job<DataType> {
    pub fn new(data: DataType) -> Self {
        let uuid = uuid::Uuid::now_v7();
        let instant = Instant::now();
        Self {
            uuid,
            creation_instant: instant,
            last_run_instant: instant,
            retries: 0,
            data,
            status: JobStatus::default(),
            options: JobOptions::default(),
        }
    }

    pub fn with_options(mut self, options: JobOptions) -> Self {
        self.options = options;
        self
    }

    pub fn is_ready(&self, now: &Instant) -> bool {
        match self.options.retry_strategy {
            RetryStrategy::Never => false,
            RetryStrategy::Constant(duration) => {
                now.duration_since(self.last_run_instant) >= duration
            }
            RetryStrategy::Map(map) => {
                now.duration_since(self.last_run_instant) >= map(self.retries)
            }
        }
    }
}
