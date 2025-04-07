use std::{sync::Arc, thread::JoinHandle, time::Duration};

use tokio::sync::{
    Semaphore,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use crate::job::{Job, JobOptions, JobStatus};

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
    active_thread: tokio::task::JoinHandle<()>,
    delayed_thread: tokio::task::JoinHandle<()>,
    callback: Callback,
    add_job_sender: UnboundedSender<Job<DataType>>,
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
        let (add_job_sender, add_job_receiver) = mpsc::unbounded_channel();
        let (delay_job_sender, delay_job_receiver) = mpsc::unbounded_channel();
        let semaphore = Arc::new(Semaphore::new(options.concurrency));

        let active_thread = tokio::spawn(Self::run_active_thread(
            semaphore,
            callback,
            add_job_receiver,
            delay_job_sender,
        ));

        let delayed_thread = tokio::spawn(Self::run_delayed_thread(
            delay_job_receiver,
            add_job_sender.clone(),
        ));

        Self {
            active_thread,
            delayed_thread,
            callback,
            add_job_sender,
            options,
        }
    }

    pub fn create_job(&mut self, data: DataType) {
        let job = Job::new(data).with_options(JobOptions {
            retry_strategy: crate::job::RetryStrategy::Constant(Duration::from_secs(5)),
            max_attemps: 5,
        });
        self.add_job_sender.send(job).unwrap();
    }

    async fn run_active_thread(
        semaphore: Arc<Semaphore>,
        callback: Callback,
        mut add_job_receiver: UnboundedReceiver<Job<DataType>>,
        delay_job_sender: UnboundedSender<Job<DataType>>,
    ) {
        loop {
            let semaphore = semaphore.clone();
            let delay_job_sender = delay_job_sender.clone();
            if let Some(mut job) = add_job_receiver.recv().await {
                tokio::spawn(async move {
                    job.status = JobStatus::Waiting;
                    let permit = semaphore.acquire_owned().await.unwrap();
                    job.status = JobStatus::Active;
                    job.attemps += 1;
                    let result = (callback)(job).await;
                    drop(permit);
                    match result {
                        Ok(mut job) => {
                            println!("job completed");
                            job.status = JobStatus::Completed;
                        }
                        Err(mut job) => {
                            if job.should_fail() {
                                println!("job failed");
                                job.status = JobStatus::Failed;
                            } else {
                                delay_job_sender.send(job).unwrap();
                            }
                        }
                    };
                });
            }
        }
    }

    async fn run_delayed_thread(
        mut delay_job_receiver: UnboundedReceiver<Job<DataType>>,
        add_job_sender: UnboundedSender<Job<DataType>>,
    ) {
        loop {
            let add_job_sender = add_job_sender.clone();
            if let Some(mut job) = delay_job_receiver.recv().await {
                tokio::spawn(async move {
                    job.status = JobStatus::Delayed;
                    println!("delaying job");
                    job.delay().await;
                    println!("retrying job");
                    add_job_sender.send(job).unwrap();
                });
            }
        }
    }

    pub async fn run(self) {
        self.active_thread.await.unwrap();
    }
}
