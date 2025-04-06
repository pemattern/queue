use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::job::{Job, JobStatus};

pub type QueueResult<DataType> = Result<Job<DataType>, Job<DataType>>;

struct QueueOptions {
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
    Fut: Future<Output = QueueResult<DataType>> + Send + 'static,
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

    pub async fn create_job(&mut self, data: DataType) {
        let job = Job::new(data);
        tokio::spawn(self.run_job(job));
    }

    async fn run_job(&self, mut job: Job<DataType>) {
        loop {
            let semaphore = self.semaphore.clone();
            let callback = self.callback.clone();
            let handle = tokio::spawn(async move {
                job.status = JobStatus::Waiting;
                let permit = semaphore.acquire_owned().await.unwrap();
                job.status = JobStatus::Active;
                let result = (callback)(job).await;
                drop(permit);
                let status = match result {
                    Ok(mut job) => {
                        job.status = JobStatus::Completed;
                        (job, JobStatus::Completed)
                    }
                    Err(mut job) => {
                        if job.should_fail() {
                            job.status = JobStatus::Failed;
                            (job, JobStatus::Failed)
                        } else {
                            job.delay().await;
                            (job, JobStatus::Delayed)
                        }
                    }
                };
                status
            });
            let job_result = handle.await.unwrap();
            if matches!(job_result.1, JobStatus::Delayed) {
                job = job_result.0;
            } else {
                break;
            }
        }
    }
}
