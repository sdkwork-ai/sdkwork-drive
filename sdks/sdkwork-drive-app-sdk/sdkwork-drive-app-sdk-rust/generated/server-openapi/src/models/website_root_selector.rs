use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::models::{WebsiteRootFolderSelector, WebsiteRootSpaceSelector};

#[derive(Debug, Clone)]
pub enum WebsiteRootSelector {
    WebsiteRootSpaceSelector(WebsiteRootSpaceSelector),
    WebsiteRootFolderSelector(WebsiteRootFolderSelector),
}

impl Serialize for WebsiteRootSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::WebsiteRootSpaceSelector(value) => value.serialize(serializer),
            Self::WebsiteRootFolderSelector(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for WebsiteRootSelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const VARIANTS: &[&str] = &["SPACE_ROOT", "FOLDER"];
        let value = serde_json::Value::deserialize(deserializer)?;
        let kind = value
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| de::Error::missing_field("mode"))?
            .to_owned();

        match kind.as_str() {
            "SPACE_ROOT" => Ok(Self::WebsiteRootSpaceSelector(
                serde_json::from_value(value).map_err(de::Error::custom)?,
            )),
            "FOLDER" => Ok(Self::WebsiteRootFolderSelector(
                serde_json::from_value(value).map_err(de::Error::custom)?,
            )),
            other => Err(de::Error::unknown_variant(other, VARIANTS)),
        }
    }
}

impl Default for WebsiteRootSelector {
    fn default() -> Self {
        Self::WebsiteRootSpaceSelector(WebsiteRootSpaceSelector::default())
    }
}
