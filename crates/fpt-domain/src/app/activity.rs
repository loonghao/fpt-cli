use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;
use super::query_helpers::build_query_params;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn activity_stream(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .activity_stream(&config, entity, id, &params)
            .await
    }

    pub async fn event_log_entries(
        &self,
        overrides: ConnectionOverrides,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport.event_log_entries(&config, &params).await
    }

    pub async fn preferences_get(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.preferences_get(&config).await
    }
}
