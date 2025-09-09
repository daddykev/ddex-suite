use napi::bindgen_prelude::*;
use napi_derive::napi;
use ddex_builder::{DdexBuilder, BuildRequest, BuildOptions, BuildResult};
use ddex_core::models::FlattenedRelease;
use serde::{Deserialize, Serialize};

#[napi]
pub struct DDEXBuilder {
    inner: ddex_builder::DdexBuilder,
}

#[napi]
impl DDEXBuilder {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(DDEXBuilder {
            inner: ddex_builder::DdexBuilder::new(),
        })
    }

    #[napi]
    pub async fn build(&self, request: String, options: Option<String>) -> Result<String> {
        let req: BuildRequest = serde_json::from_str(&request)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        let opts: BuildOptions = if let Some(opts_str) = options {
            serde_json::from_str(&opts_str)
                .map_err(|e| Error::from_reason(e.to_string()))?
        } else {
            BuildOptions::default()
        };

        let result = self.inner.build(req, opts)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;

        serde_json::to_string(&result)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub fn build_sync(&self, request: String, options: Option<String>) -> Result<String> {
        let req: BuildRequest = serde_json::from_str(&request)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        let opts: BuildOptions = if let Some(opts_str) = options {
            serde_json::from_str(&opts_str)
                .map_err(|e| Error::from_reason(e.to_string()))?
        } else {
            BuildOptions::default()
        };

        let result = self.inner.build_sync(req, opts)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        serde_json::to_string(&result)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn preflight(&self, request: String) -> Result<String> {
        let req: BuildRequest = serde_json::from_str(&request)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let result = self.inner.preflight(req)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;

        serde_json::to_string(&result)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub fn canonicalize(&self, xml: String) -> Result<String> {
        self.inner.canonicalize(&xml)
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}