use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn entity_followers(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_followers(&config, entity, id).await
    }

    pub async fn entity_follow(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        user: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_follow(&config, entity, id, &user)
            .await
    }

    pub async fn entity_unfollow(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        user: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_unfollow(&config, entity, id, &user)
            .await
    }
}
