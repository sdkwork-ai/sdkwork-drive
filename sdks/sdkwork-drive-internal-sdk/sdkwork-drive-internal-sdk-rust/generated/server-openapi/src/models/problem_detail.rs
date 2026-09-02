use serde::{Deserialize, Serialize};

use crate::models::{FieldError};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProblemDetail {
    pub r#type: String,

    pub title: String,

    pub status: i64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    pub code: i64,

    #[serde(rename = "traceId")]
    pub trace_id: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,
}
