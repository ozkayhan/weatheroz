use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationEntity {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: String,
    pub admin_region: String,
}

#[async_trait]
pub trait LocationService: Send + Sync {
    /// Resolves coordinate details using written query string.
    async fn resolve_by_name(
        &self,
        query: &str,
    ) -> Result<LocationEntity, Box<dyn std::error::Error + Send + Sync>>;

    /// Auto-resolves location coordinates using the client public IP.
    async fn resolve_by_ip(
        &self,
    ) -> Result<LocationEntity, Box<dyn std::error::Error + Send + Sync>>;
}
