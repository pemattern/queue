pub mod job;
pub mod queue;
pub mod worker;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use queue::Queue;

    use crate::{job::Job, queue::QueueResult};

    use super::*;

    #[derive(Clone)]
    struct MyData {
        num: usize,
    }
    async fn proc(mut job: Job<MyData>) -> QueueResult<MyData> {
        println!("Processing Job: {}", job.uuid);
        for _ in 0..10 {
            println!("Processed {} times", job.data.num);
            job.data.num += 1;
            if job.data.num == 3 {
                return Err(job);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Ok(job)
    }

    #[tokio::test]
    async fn it_works() {
        let mut queue = Queue::new(proc);
        for i in 0..100 {
            let data = MyData { num: i };
            queue.create_job(data);
        }
        queue.run().await;
    }
}
