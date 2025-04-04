use std::{sync::Arc, time::Duration};

use tokio::{sync::Semaphore, task::JoinSet};

use crate::job::Job;

pub struct QueueOptions {
    concurrency: usize,
}

impl Default for QueueOptions {
    fn default() -> Self {
        Self { concurrency: 10 }
    }
}

pub struct Queue<DataType, Callback> {
    jobs: Vec<Job<DataType>>,
    callback: Callback,
    semaphore: Arc<Semaphore>,
    options: QueueOptions,
}

impl<DataType, Callback, Fut> Queue<DataType, Callback>
where
    DataType: Send + 'static,
    Callback: Fn(Job<DataType>) -> Fut + Clone + Copy + Send + 'static,
    Fut: Future<Output = Result<DataType, DataType>> + Send + 'static,
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
        let job = Job::new(data);
        self.jobs.push(job);
    }

    pub async fn run(self) {
        let mut set = JoinSet::new();
        for job in self.jobs {
            let semaphore = self.semaphore.clone();
            set.spawn(async move {
                let _permit = semaphore.acquire_owned().await.unwrap();
                let result = (self.callback)(job).await;
            });
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
