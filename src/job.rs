enum JobStatus {
    Active,
    Complete,
    Waiting,
    Completed,
    Failed,
    Delayed,
    Paused,
}

enum JobMessage {}

pub(crate) struct Job<DataType> {
    id: usize,
    retries: usize,
    pub data: DataType,
    status: JobStatus,
}

impl<DataType> Job<DataType> {
    pub fn new(id: usize, data: DataType) -> Self {
        Self {
            id,
            retries: 0,
            data,
            status: JobStatus::Active,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn data_mut(&mut self) -> &mut DataType {
        &mut self.data
    }
}
