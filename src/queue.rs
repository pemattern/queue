use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::job::Job;

pub enum ProcessResult {
    Success,
    Failure,
}

pub struct QueueOptions {
    concurrency: usize,
}

impl Default for QueueOptions {
    fn default() -> Self {
        Self { concurrency: 1 }
    }
}

pub struct Queue<DataType, Callback, Fut>
where
    Callback: FnMut(DataType) -> Fut,
    Fut: Future<Output = ProcessResult>,
{
    jobs: Vec<Job<DataType>>,
    callback: Callback,
    semaphore: Arc<Semaphore>,
    options: QueueOptions,
}

impl<DataType, Callback, Fut> Queue<DataType, Callback, Fut>
where
    Callback: FnMut(DataType) -> Fut,
    Fut: Future<Output = ProcessResult>,
{
    pub fn new(callback: Callback) -> Self {
        let options = QueueOptions::default();
        Self {
            jobs: Vec::new(),
            callback,
            semaphore: Arc::new(Semaphore::new(options.concurrency)),
            options,
        }
    }

    pub fn create_job(&mut self, data: DataType) {
        let job = Job::new(data, self.semaphore.clone());
        self.jobs.push(job);
    }

    pub async fn run(&mut self) {}
}
