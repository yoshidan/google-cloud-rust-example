use std::sync::Arc;
use google_cloud_spanner::client::Client as SpannerClient;

#[derive(Clone)]
pub struct InjectedApi {
}

impl InjectedApi {
    pub fn new(spanner_client: SpannerClient) -> Self {
        Self {
        }
    }
}
