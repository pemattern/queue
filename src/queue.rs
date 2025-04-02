use std::collections::VecDeque;

use crate::job::Job;

pub enum ProcessResult {
    Success,
    Failure,
}

pub struct QueueOptions {
    concurrency: usize,
    max_retries: usize,
}

impl Default for QueueOptions {
    fn default() -> Self {
        Self {
            concurrency: 1,
            max_retries: 1,
        }
    }
}

pub struct Queue<DataType, Callback, Fut>
where
    Callback: FnMut(DataType) -> Fut,
    Fut: Future<Output = ProcessResult>,
{
    delayed_jobs: Vec<Job<DataType>>,
    failed_jobs: Vec<Job<DataType>>,
    waiting_jobs: VecDeque<Job<DataType>>,
    active_jobs: Vec<Job<DataType>>,
    callback: Callback,
    options: QueueOptions,
}

impl<DataType, Callback, Fut> Queue<DataType, Callback, Fut>
where
    Callback: FnMut(DataType) -> Fut,
    Fut: Future<Output = ProcessResult>,
{
    pub fn new(callback: Callback) -> Self {
        Self {
            delayed_jobs: Vec::new(),
            failed_jobs: Vec::new(),
            waiting_jobs: VecDeque::new(),
            active_jobs: Vec::new(),
            callback,
            options: QueueOptions::default(),
        }
    }

    pub fn create_job(&mut self, data: DataType) {
        let job = Job::new(data);
        self.waiting_jobs.push_back(job);
    }

    pub async fn run(mut self) {
        loop {
            self = self.advance_jobs();
        }
    }

    fn advance_jobs(mut self) -> Self {
        let instant = std::time::Instant::now();
        self.waiting_jobs.extend(
            self.delayed_jobs
                .drain(..)
                .filter(|job| job.is_ready(&instant)),
        );
        while self.active_jobs.len() < self.options.concurrency && !self.waiting_jobs.is_empty() {
            let job = self.waiting_jobs.pop_front().unwrap();
            self.active_jobs.push(job);
        }
        self
    }
}
