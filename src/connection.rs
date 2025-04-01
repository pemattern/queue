pub struct Connection {
    connection: String,
}

impl Connection {
    pub fn init(connection: impl Into<String>) -> Self {
        Self {
            connection: connection.into(),
        }
    }
}
