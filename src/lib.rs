pub mod job;
pub mod mq;
pub mod queue;
pub mod worker;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use queue::Queue;

    use crate::{job::Job, queue::QueueResult};

    use super::*;
    use rand::prelude::*;

    #[derive(Clone)]
    struct MyData {
        num: usize,
    }
    async fn proc(mut job: Job<MyData>) -> QueueResult<MyData> {
        for i in 0..10 {
            job.data.num = i;
            if job.data.num == rand::rng().random_range(0..10) {
                return Err(job);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Ok(job)
    }

    #[tokio::test]
    async fn it_works() {
        let mut queue = Queue::new("test queue", proc);
        for i in 0..100 {
            let data = MyData { num: i };
            queue.create_job(data);
        }
        queue.run().await;
    }
}
