use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, resolve_site};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn server_info(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let site = resolve_site(overrides)?;
        self.transport.server_info(&site).await
    }
}
