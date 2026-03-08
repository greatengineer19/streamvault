use aws_config::BehaviorVersion;
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::{Client, config::Builder, presigning::PresigningConfig};
use std::time::Duration;

use crate::config::Config;

pub async fn build_client(config: &Config) -> Client {
    let credentials = Credentials::new(
        &config.r2_access_key_id,
        &config.r2_secret_access_key,
        None,
        None,
        "streamvault",
    );

    let s3_config = Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .endpoint_url(config.r2_endpoint())
        .credentials_provider(credentials)
        .region(Region::new("auto"))
        .force_path_style(true)
        .build();

    Client::from_conf(s3_config)
}

/// Generate a presigned PUT URL - browser uploads directly to R2
pub async fn presigned_put_url(
    client: &Client,
    bucket: &str,
    key: &str,
    expires_in_secs: u64,
) -> Result<String, String> {
    let presigning = PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))
        .map_err(|e| e.to_string())?;

    let presigned = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning)
        .await
        .map_err(|e| e.to_string())?;

    Ok(presigned.uri().to_string())
}

/// Generate a presigned GET URL - browser streams directly from R2
pub async fn presigned_get_url(
    client: &Client,
    bucket: &str,
    key: &str,
    expires_in_secs: u64,
) -> Result<String, String> {
    let presigning = PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))
        .map_err(|e| e.to_string())?;

    let presigned = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning)
        .await
        .map_err(|e| e.to_string())?;

    Ok(presigned.uri().to_string())
}
