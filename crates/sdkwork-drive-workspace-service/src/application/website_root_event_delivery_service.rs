use sdkwork_drive_contract::drive::events::derive_webhook_signing_key;

use crate::domain::provider_event_delivery::DriveProviderEventDelivery;
use crate::ports::provider_event_delivery_store::{
    DriveProviderEventDeliveryStore, DriveProviderEventResourceKind,
    EnsureDriveProviderEventDelivery,
};
use crate::DriveServiceError;

use super::root_scope_event_delivery_service::{
    validate_event_delivery_expiration, validate_verification_token,
};
use super::root_scope_subscription_service::require_text;

#[derive(Debug, Clone)]
pub struct EnsureWebsiteRootEventDeliveryCommand {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub channel_id: String,
    pub address: String,
    pub verification_token: String,
    pub expiration_epoch_ms: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct EnsureWebsiteRootEventDeliveryResult {
    pub delivery: DriveProviderEventDelivery,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct DriveWebsiteRootEventDeliveryService<S>
where
    S: DriveProviderEventDeliveryStore,
{
    store: S,
}

impl<S> DriveWebsiteRootEventDeliveryService<S>
where
    S: DriveProviderEventDeliveryStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn ensure_delivery(
        &self,
        command: EnsureWebsiteRootEventDeliveryCommand,
    ) -> Result<EnsureWebsiteRootEventDeliveryResult, DriveServiceError> {
        let tenant_id = require_text(command.tenant_id, "tenant_id", 64)?;
        let website_root_uuid = require_text(command.website_root_uuid, "website_root_uuid", 64)?;
        uuid::Uuid::parse_str(&website_root_uuid).map_err(|_| {
            DriveServiceError::Validation("website_root_uuid must be a UUID".to_string())
        })?;
        let channel_id = validate_channel_id(command.channel_id)?;
        let address = sdkwork_drive_security::validate_webhook_https_url(&command.address)
            .map_err(|detail| DriveServiceError::Validation(detail.to_string()))?
            .to_string();
        validate_verification_token(&command.verification_token)?;
        validate_event_delivery_expiration(command.expiration_epoch_ms)?;
        let operator_id = require_text(command.operator_id, "operator_id", 128)?;
        let result = self
            .store
            .ensure_delivery(&EnsureDriveProviderEventDelivery {
                tenant_id,
                resource_kind: DriveProviderEventResourceKind::WebsiteRoot,
                provider_resource_uuid: website_root_uuid,
                channel_id,
                address,
                signing_key_sha256: derive_webhook_signing_key(&command.verification_token),
                expiration_epoch_ms: command.expiration_epoch_ms,
                operator_id,
            })
            .await?;
        Ok(EnsureWebsiteRootEventDeliveryResult {
            delivery: result.delivery,
            created: result.created,
        })
    }
}

fn validate_channel_id(channel_id: String) -> Result<String, DriveServiceError> {
    let channel_id = require_text(channel_id, "channel_id", 64)?;
    if !channel_id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(DriveServiceError::Validation(
            "channel_id contains unsupported characters".to_string(),
        ));
    }
    Ok(channel_id)
}
