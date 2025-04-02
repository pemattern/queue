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
    jobs: Vec<Job<DataType>>,
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
            jobs: Vec::new(),
            callback,
            options: QueueOptions::default(),
        }
    }

    pub fn create_job(&mut self, data: DataType) {
        let job = Job::new(data);
        self.jobs.push(job);
    }

    pub async fn run(&mut self) {}
}
