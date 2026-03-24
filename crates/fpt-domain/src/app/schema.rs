use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn schema_entities(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_entities(&config).await
    }

    pub async fn schema_fields(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_fields(&config, entity).await
    }

    pub async fn schema_field_create(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schema_field_create(&config, entity, &body)
            .await
    }

    pub async fn schema_field_update(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        field_name: &str,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schema_field_update(&config, entity, field_name, &body)
            .await
    }

    pub async fn schema_field_delete(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schema_field_delete(&config, entity, field_name)
            .await
    }

    pub async fn schema_field_read(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schema_field_read(&config, entity, field_name)
            .await
    }

    pub async fn schema_entity_read(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_entity_read(&config, entity).await
    }

    pub async fn schema_entity_update(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schema_entity_update(&config, entity, &body)
            .await
    }

    pub async fn schema_entity_delete(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_entity_delete(&config, entity).await
    }

    pub async fn schema_field_revive(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schema_field_revive(&config, entity, field_name)
            .await
    }

    pub async fn schema_entity_create(
        &self,
        overrides: ConnectionOverrides,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_entity_create(&config, &body).await
    }

    pub async fn schema_entity_revive(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_entity_revive(&config, entity).await
    }
}
