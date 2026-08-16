use async_trait::async_trait;

use crate::domain::space::DriveSpace;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct NewDriveSpace {
    pub id: String,
    pub tenant_id: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub display_name: String,
    pub space_type: String,
    pub lifecycle_status: String,
    pub presentation_icon: Option<String>,
    pub presentation_color: Option<String>,
    pub description: Option<String>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ListAccessibleSpacesQuery<'a> {
    pub tenant_id: &'a str,
    pub viewer_subject_type: &'a str,
    pub viewer_subject_id: &'a str,
    pub owner_subject_type: Option<&'a str>,
    pub owner_subject_id: Option<&'a str>,
    pub space_type: Option<&'a str>,
    pub offset: i64,
    pub limit: i64,
}

#[async_trait]
pub trait DriveSpaceStore: Send + Sync {
    async fn insert_space(
        &self,
        new_space: &NewDriveSpace,
    ) -> Result<DriveSpace, DriveServiceError>;

    async fn insert_website_space(
        &self,
        new_space: &NewDriveSpace,
    ) -> Result<DriveSpace, DriveServiceError>;

    async fn list_spaces(
        &self,
        tenant_id: &str,
        owner_subject_type: Option<&str>,
        owner_subject_id: Option<&str>,
        space_type: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveSpace>, DriveServiceError>;

    async fn list_accessible_spaces(
        &self,
        query: ListAccessibleSpacesQuery<'_>,
    ) -> Result<Vec<DriveSpace>, DriveServiceError>;

    async fn get_space(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<DriveSpace, DriveServiceError>;

    #[allow(clippy::too_many_arguments)]
    async fn update_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        display_name: Option<&str>,
        presentation_icon: Option<&str>,
        presentation_color: Option<&str>,
        description: Option<&str>,
        operator_id: &str,
    ) -> Result<DriveSpace, DriveServiceError>;

    async fn delete_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        operator_id: &str,
    ) -> Result<DriveSpace, DriveServiceError>;
}
