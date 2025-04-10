use std::time::Duration;

use redis::{IntoConnectionInfo, RedisError, aio::MultiplexedConnection};

pub struct MQ {
    connection: MultiplexedConnection,
}

impl MQ {
    pub async fn instance<T: IntoConnectionInfo>(info: T) -> Result<Self, RedisError> {
        let client = redis::Client::open(info)?;
        let timeout = Duration::from_secs(15);
        let connection = client
            .get_multiplexed_tokio_connection_with_response_timeouts(timeout, timeout)
            .await?;
        Ok(MQ { connection })
    }
}
