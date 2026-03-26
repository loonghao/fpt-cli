use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn hierarchy_search(
        &self,
        overrides: ConnectionOverrides,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.hierarchy(&config, &body).await
    }

    pub async fn hierarchy_expand(
        &self,
        overrides: ConnectionOverrides,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.hierarchy_expand(&config, &body).await
    }
}
