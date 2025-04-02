use std::sync::Arc;

use tokio::sync::Semaphore;

pub(crate) struct Worker {
    semaphore: Arc<Semaphore>,
}

impl Worker {
    pub fn new(concurrency: usize) -> Self {
        let semaphore = Arc::new(Semaphore::new(concurrency));
        Self { semaphore }
    }
}
