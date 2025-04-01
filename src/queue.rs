use std::time::Duration;

use crate::job::Job;

#[derive(Default)]
pub enum RetryStrategy {
    #[default]
    Never,
    Constant(usize),
    Map(fn(usize) -> usize),
}

pub enum ProcessResult {
    Success,
    Failure,
}

pub struct Queue<DataType, Callback, Fut>
where
    Callback: FnMut(DataType) -> Fut,
    Fut: Future<Output = ProcessResult>,
{
    jobs: Vec<Job<DataType>>,
    retry_strategy: RetryStrategy,
    callback: Callback,
}

impl<DataType, Callback, Fut> Queue<DataType, Callback, Fut>
where
    Callback: FnMut(DataType) -> Fut,
    Fut: Future<Output = ProcessResult>,
{
    pub fn new(callback: Callback) -> Self {
        Self {
            jobs: Vec::new(),
            retry_strategy: RetryStrategy::default(),
            callback,
        }
    }

    pub fn create_job(&mut self, data: DataType) {
        let job = Job::new(self.next_id(), data);
        self.jobs.push(job);
    }

    fn next_id(&self) -> usize {
        self.jobs.iter().map(|item| item.id()).max().unwrap_or(0) + 1
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(job) = self.jobs.pop() {
                (self.callback)(job.data).await;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
