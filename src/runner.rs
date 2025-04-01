use crate::processor::Processor;

pub struct Runner {
    processors: Vec<Box<dyn Processor>>,
}

impl Runner {
    pub fn new(processors: Vec<Box<dyn Processor>>) -> Self {
        Self { processors }
    }
}
