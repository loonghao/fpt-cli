use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::{ShotgridTransport, UploadUrlRequest};

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn upload_url(
        &self,
        overrides: ConnectionOverrides,
        request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.upload_url(&config, request).await
    }

    pub async fn download_url(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        field_name: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .download_url(&config, entity, id, field_name)
            .await
    }

    pub async fn thumbnail_url(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.thumbnail_url(&config, entity, id).await
    }

    pub async fn filmstrip_thumbnail(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .filmstrip_thumbnail(&config, entity, id)
            .await
    }

    pub async fn thumbnail_upload(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .thumbnail_upload(&config, entity, id, &body)
            .await
    }
}
