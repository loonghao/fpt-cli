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

    pub async fn user_following(
        &self,
        overrides: ConnectionOverrides,
        user_id: u64,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = super::query_helpers::build_query_params(input)?;
        self.transport
            .user_following(&config, user_id, &params)
            .await
    }
}
