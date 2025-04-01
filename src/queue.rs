use std::{pin::Pin, time::Duration};

use crate::{job::Job, processor::Processor};

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

type AsyncCallback<DataType> =
    Box<dyn FnMut(&mut DataType) -> Pin<Box<dyn Future<Output = ProcessResult> + Send>>>;

pub struct Queue<DataType> {
    jobs: Vec<Job<DataType>>,
    retry_strategy: RetryStrategy,
    callback: AsyncCallback<DataType>,
}

impl<DataType> Queue<DataType> {
    pub fn new<F, Fut>(mut callback: F) -> Self
    where
        F: for<'a> FnMut(&'a mut DataType) -> Fut + 'static,
        Fut: Future<Output = ProcessResult> + Send + 'static,
    {
        Self {
            jobs: Vec::new(),
            retry_strategy: RetryStrategy::default(),
            callback: Box::new(move |x| Box::pin(callback(x))),
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
            if let Some(mut job) = self.jobs.pop() {
                (self.callback)(&mut job.data).await;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
