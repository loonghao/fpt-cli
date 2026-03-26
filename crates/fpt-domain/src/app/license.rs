use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn license(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.license(&config).await
    }
}
