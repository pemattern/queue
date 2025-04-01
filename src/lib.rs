mod connection;
mod job;
mod processor;
mod queue;
mod runner;

#[cfg(test)]
mod tests {
    use connection::Connection;
    use queue::{ProcessResult, Queue};

    use super::*;

    struct MyData {
        num: usize,
    }
    async fn proc<'a>(data: &'a mut MyData) -> ProcessResult {
        println!("Processed {} times", &data.num);
        ProcessResult::Success
    }

    #[tokio::test]
    async fn it_works() {
        let mut queue = Queue::new(proc);
        let data = MyData { num: 1 };
        queue.create_job(data);

        queue.run().await;
    }
}
