use crate::config::{load_config_from_default_env, ConfigError};
use crate::models::{ApplicationConfig, StorageConfig};
use aws_config;
use aws_config::BehaviorVersion;
use aws_credential_types;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Region;
use thiserror::Error;

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
        let subpath = subpath.trim_start_matches('/');

        let new_key = if self.key.is_empty() {
            subpath.to_string()
        } else if self.key.ends_with('/') {
            format!("{}{}", self.key, subpath)
        } else {
            format!("{}/{}", self.key, subpath)
        };

        Self {
            bucket: self.bucket.clone(),
            key: new_key,
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

    pub fn from_default_env() -> Result<Self, ConfigError> {
        let config = load_config_from_default_env()?;
        Ok(Self { config })
    }

    pub async fn get_client(&self) -> Result<Client, StorageError> {
        let storage = self
            .get_storage_config()
            .ok_or(StorageError::NoStorageConfig)?;
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

        shared_config = shared_config.region(Region::new("nyc3")); // Default make87 bucket region

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

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("No storage config found")]
    NoStorageConfig,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use super::*;
    use crate::models::{ApplicationEnvConfig, ApplicationInfo, MountedPeripherals, StorageConfig};


    fn create_test_storage_config() -> StorageConfig {
        StorageConfig {
            url: "s3://test-bucket/system".to_string(),
            access_key: "test_access_key".to_string(),
            secret_key: "test_secret_key".to_string(),
            endpoint_url: "http://localhost:9000".to_string(),
        }
    }

    fn create_test_application_config() -> ApplicationConfig {
        ApplicationEnvConfig {
            interfaces: BTreeMap::new(),
            peripherals: MountedPeripherals {
                peripherals: vec![],
            },
            config: serde_json::json!({}),
            storage: Some(create_test_storage_config()),
            application_info: ApplicationInfo {
                deployed_application_id: "test-deployed-app".to_string(),
                deployed_application_name: String::new(),
                system_id: String::new(),
                application_id: "test-app".to_string(),
                application_name: String::new(),
                git_url: None,
                git_branch: None,
                is_release_version: false,
            },
        }
    }

    #[test]
    fn test_s3path_new() {
        let path = S3Path::new("s3://my-bucket/path/to/file");
        assert_eq!(path.bucket, "my-bucket");
        assert_eq!(path.key, "path/to/file");

        let path = S3Path::new("my-bucket/path/to/file");
        assert_eq!(path.bucket, "my-bucket");
        assert_eq!(path.key, "path/to/file");

        let path = S3Path::new("s3://bucket-only");
        assert_eq!(path.bucket, "bucket-only");
        assert_eq!(path.key, "");

        let path = S3Path::new("s3://bucket/");
        assert_eq!(path.bucket, "bucket");
        assert_eq!(path.key, "");
    }

    #[test]
    fn test_s3path_join() {
        let base = S3Path::new("s3://bucket/base");
        let joined = base.join("subpath");
        assert_eq!(joined.key, "base/subpath");

        let base = S3Path::new("s3://bucket/base/");
        let joined = base.join("subpath");
        assert_eq!(joined.key, "base/subpath");

        let base = S3Path::new("s3://bucket");
        let joined = base.join("subpath");
        assert_eq!(joined.key, "subpath");

        let base = S3Path::new("s3://bucket/base");
        let joined = base.join("/subpath");
        assert_eq!(joined.key, "base/subpath");
    }

    #[test]
    fn test_s3path_to_uri() {
        let path = S3Path::new("s3://bucket/path/file");
        assert_eq!(path.to_uri(), "s3://bucket/path/file");

        let path = S3Path::new("s3://bucket");
        assert_eq!(path.to_uri(), "s3://bucket");
    }

    #[test]
    fn test_s3path_display() {
        let path = S3Path::new("s3://bucket/path/file");
        assert_eq!(format!("{}", path), "s3://bucket/path/file");

        let path = S3Path::new("s3://bucket");
        assert_eq!(format!("{}", path), "s3://bucket");
    }

    #[test]
    fn test_s3path_filename() {
        let path = S3Path::new("s3://bucket/path/to/file.txt");
        assert_eq!(path.filename(), Some("file.txt"));

        let path = S3Path::new("s3://bucket/path/to/");
        assert_eq!(path.filename(), Some(""));

        let path = S3Path::new("s3://bucket");
        assert_eq!(path.filename(), None);

        let path = S3Path::new("s3://bucket/file");
        assert_eq!(path.filename(), Some("file"));
    }

    #[test]
    fn test_s3path_parent() {
        let path = S3Path::new("s3://bucket/path/to/file");
        let parent = path.parent().unwrap();
        assert_eq!(parent.bucket, "bucket");
        assert_eq!(parent.key, "path/to");

        let path = S3Path::new("s3://bucket/file");
        let parent = path.parent().unwrap();
        assert_eq!(parent.bucket, "bucket");
        assert_eq!(parent.key, "");

        let path = S3Path::new("s3://bucket");
        assert!(path.parent().is_none());
    }

    #[test]
    fn test_blob_storage_new() {
        let config = create_test_application_config();
        let storage = BlobStorage::new(config.clone());
        assert_eq!(storage.config.application_info.application_id, "test-app");
    }

    #[test]
    fn test_blob_storage_get_system_path() {
        let config = create_test_application_config();
        let storage = BlobStorage::new(config);
        let system_path = storage.get_system_path().unwrap();
        assert_eq!(system_path.bucket, "test-bucket");
        assert_eq!(system_path.key, "system");
    }

    #[test]
    fn test_blob_storage_get_application_path() {
        let config = create_test_application_config();
        let storage = BlobStorage::new(config);
        let app_path = storage.get_application_path().unwrap();
        assert_eq!(app_path.bucket, "test-bucket");
        assert_eq!(app_path.key, "system/test-app");
    }

    #[test]
    fn test_blob_storage_get_deployed_application_path() {
        let config = create_test_application_config();
        let storage = BlobStorage::new(config);
        let deployed_path = storage.get_deployed_application_path().unwrap();
        assert_eq!(deployed_path.bucket, "test-bucket");
        assert_eq!(deployed_path.key, "system/test-deployed-app");
    }

    #[test]
    fn test_blob_storage_no_storage_config() {
        let mut config = create_test_application_config();
        config.storage = None;
        let storage = BlobStorage::new(config);
        assert!(storage.get_system_path().is_none());
        assert!(storage.get_application_path().is_none());
        assert!(storage.get_deployed_application_path().is_none());
    }

    #[test]
    fn test_s3path_equality() {
        let path1 = S3Path::new("s3://bucket/path");
        let path2 = S3Path::new("s3://bucket/path");
        let path3 = S3Path::new("s3://bucket/other");

        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
    }
}
