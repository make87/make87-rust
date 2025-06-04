use crate::config::load_config_from_default_env;
use crate::models::{ApplicationConfig, StorageConfig};
use aws_config;
use aws_config::BehaviorVersion;
use aws_credential_types;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use serde::de::StdError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3Path {
    pub bucket: String,
    pub key: String,
}

impl S3Path {
    /// Parse from a URI like "s3://my-bucket/path/to/file"
    pub fn new(uri: &str) -> Self {
        let uri = uri.strip_prefix("s3://").unwrap_or(uri);
        let mut parts = uri.splitn(2, '/');
        let bucket = parts.next().unwrap_or("").to_string();
        let key = parts.next().unwrap_or("").trim_start_matches('/').to_string(); // ensure no leading slash
        Self { bucket, key }
    }

    /// Create a new path relative to this one
    pub fn join(&self, subpath: &str) -> Self {
        let normalized = if self.key.is_empty() {
            subpath.trim_start_matches('/')
        } else if self.key.ends_with('/') {
            subpath.trim_start_matches('/')
        } else {
            &format!("{}/{}", self.key, subpath.trim_start_matches('/'))
        };

        Self {
            bucket: self.bucket.clone(),
            key: normalized.to_string(),
        }
    }

    /// Turn into `s3://bucket/key` style URI
    pub fn to_uri(&self) -> String {
        if self.key.is_empty() {
            format!("s3://{}", self.bucket)
        } else {
            format!("s3://{}/{}", self.bucket, self.key)
        }
    }

    /// Returns just the filename (if any) from the key
    pub fn filename(&self) -> Option<&str> {
        if self.key.is_empty() {
            None
        } else {
            self.key.rsplit('/').next()
        }
    }

    /// Returns the parent path (if any)
    pub fn parent(&self) -> Option<Self> {
        let mut parts: Vec<&str> = self.key.split('/').collect();
        if parts.is_empty() || self.key.is_empty() {
            return None;
        }
        parts.pop();
        Some(Self {
            bucket: self.bucket.clone(),
            key: parts.join("/"),
        })
    }
}

impl std::fmt::Display for S3Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.key.is_empty() {
            write!(f, "s3://{}", self.bucket)
        } else {
            write!(f, "s3://{}/{}", self.bucket, self.key)
        }
    }
}

pub struct BlobStorage {
    config: ApplicationConfig,
}

impl BlobStorage {
    pub fn new(config: ApplicationConfig) -> Self {
        Self { config }
    }

    pub fn from_default_env() -> Result<Self, Box<dyn StdError + Send + Sync>> {
        let config = load_config_from_default_env()?;
        Ok(Self { config })
    }

    pub async fn get_client(&self) -> Result<Client, Box<dyn std::error::Error>> {
        let storage = self.get_storage_config().ok_or("No storage config found")?;
        let credentials = Credentials::from_keys(
            &storage.access_key,
            &storage.secret_key,
            None,
        );
        let provider = SharedCredentialsProvider::new(credentials);

        let mut shared_config = aws_config::load_defaults(BehaviorVersion::latest())
            .await
            .to_builder()
            .credentials_provider(provider);

        // Set custom endpoint if provided
        if !storage.endpoint_url.is_empty() {
            shared_config = shared_config.endpoint_url(&storage.endpoint_url);
        }

        let shared_config = shared_config.build();

        Ok(Client::new(&shared_config))
    }

    fn get_storage_config(&self) -> Option<&StorageConfig> {
        self.config.storage.as_ref()
    }

    pub fn get_system_path(&self) -> Option<S3Path> {
        self.get_storage_config().map(|storage| S3Path::new(&storage.url))
    }

    pub fn get_application_path(&self) -> Option<S3Path> {
        let app_id = &self.config.application_info.application_id;
        self.get_system_path()
            .map(|path| path.join(app_id))
    }

    pub fn get_deployed_application_path(&self) -> Option<S3Path> {
        let deployed_id = &self.config.application_info.deployed_application_id;
        self.get_system_path()
            .map(|path| path.join(deployed_id))
    }
}
