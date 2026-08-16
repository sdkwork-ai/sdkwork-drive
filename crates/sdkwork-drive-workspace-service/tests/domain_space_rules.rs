use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;

#[test]
fn space_type_supports_special_spaces() {
    assert_eq!(DriveSpaceType::KnowledgeBase.as_str(), "knowledge_base");
    assert_eq!(DriveSpaceType::AiGenerated.as_str(), "ai_generated");
    assert_eq!(DriveSpaceType::GitRepository.as_str(), "git_repository");
    assert_eq!(DriveSpaceType::Deployment.as_str(), "deployment");
    assert_eq!(DriveSpaceType::AppUpload.as_str(), "app_upload");
    assert_eq!(DriveSpaceType::Im.as_str(), "im");
    assert_eq!(DriveSpaceType::Notary.as_str(), "notary");
    assert_eq!(
        DriveSpaceType::try_from_str("git_repository"),
        Some(DriveSpaceType::GitRepository)
    );
    assert_eq!(
        DriveSpaceType::try_from_str("deployment"),
        Some(DriveSpaceType::Deployment)
    );
    assert_eq!(DriveSpaceType::try_from_str("im"), Some(DriveSpaceType::Im));
    assert_eq!(
        DriveSpaceType::try_from_str("notary"),
        Some(DriveSpaceType::Notary)
    );
}
