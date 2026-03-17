use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn upload_url(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        field_name: &str,
        file_name: &str,
        content_type: Option<&str>,
        multipart_upload: bool,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .upload_url(
                &config,
                entity,
                id,
                field_name,
                file_name,
                content_type,
                multipart_upload,
            )
            .await
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
}
