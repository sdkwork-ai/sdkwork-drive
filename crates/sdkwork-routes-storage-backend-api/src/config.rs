#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminStorageConfig {
    pub object_store_adapter: DriveAdminStorageObjectStoreAdapter,
}

impl Default for AdminStorageConfig {
    fn default() -> Self {
        Self {
            object_store_adapter: DriveAdminStorageObjectStoreAdapter::AwsSdkS3,
        }
    }
}

impl AdminStorageConfig {
    pub fn from_env() -> Result<Self, String> {
        Self::from_env_pairs(std::env::vars())
    }

    pub fn from_env_pairs<I, K, V>(pairs: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut config = Self::default();
        for (key, value) in pairs {
            if key.as_ref() != "SDKWORK_DRIVE_ADMIN_STORAGE_OBJECT_STORE_ADAPTER" {
                continue;
            }
            let raw = value.as_ref().trim();
            if raw.is_empty() {
                continue;
            }
            config.object_store_adapter =
                DriveAdminStorageObjectStoreAdapter::try_from_config(raw).ok_or_else(|| {
                    format!(
                        "SDKWORK_DRIVE_ADMIN_STORAGE_OBJECT_STORE_ADAPTER is invalid: {raw}; expected aws_sdk_s3 or opendal_s3"
                    )
                })?;
        }
        Ok(config)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveAdminStorageObjectStoreAdapter {
    AwsSdkS3,
    OpendalS3,
}

impl DriveAdminStorageObjectStoreAdapter {
    fn try_from_config(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "aws_sdk_s3" | "aws-s3" | "aws_s3" | "s3" => Some(Self::AwsSdkS3),
            "opendal_s3" | "opendal-s3" => Some(Self::OpendalS3),
            _ => None,
        }
    }
}
