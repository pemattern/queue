pub mod job;
pub mod queue;
pub mod state;
pub mod worker;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use queue::{ProcessResult, Queue};

    use super::*;

    struct MyData {
        num: usize,
    }
    async fn proc(data: &mut MyData) -> ProcessResult {
        for _ in 0..10 {
            println!("Processed {} times", data.num);
            data.num += 1;
            if data.num == 3 {
                return ProcessResult::Failure;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        ProcessResult::Success
    }

    #[tokio::test]
    async fn it_works() {
        let mut queue = Queue::new(proc);
        let mut data = MyData { num: 1 };
        queue.create_job(&mut data);

        queue.run().await;
    }
}
