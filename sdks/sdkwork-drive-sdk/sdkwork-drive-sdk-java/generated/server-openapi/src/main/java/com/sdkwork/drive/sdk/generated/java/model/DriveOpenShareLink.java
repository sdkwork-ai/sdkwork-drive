package com.sdkwork.drive.sdk.generated.java.model;


public class DriveOpenShareLink {
    private String id;
    private String tenantId;
    private String role;
    private String expiresAtEpochMs;
    private String downloadLimit;
    private String downloadCount;
    private Boolean accessCodeRequired;
    private OpenNode node;

    public String getId() {
        return this.id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getTenantId() {
        return this.tenantId;
    }

    public void setTenantId(String tenantId) {
        this.tenantId = tenantId;
    }

    public String getRole() {
        return this.role;
    }

    public void setRole(String role) {
        this.role = role;
    }

    public String getExpiresAtEpochMs() {
        return this.expiresAtEpochMs;
    }

    public void setExpiresAtEpochMs(String expiresAtEpochMs) {
        this.expiresAtEpochMs = expiresAtEpochMs;
    }

    public String getDownloadLimit() {
        return this.downloadLimit;
    }

    public void setDownloadLimit(String downloadLimit) {
        this.downloadLimit = downloadLimit;
    }

    public String getDownloadCount() {
        return this.downloadCount;
    }

    public void setDownloadCount(String downloadCount) {
        this.downloadCount = downloadCount;
    }

    public Boolean getAccessCodeRequired() {
        return this.accessCodeRequired;
    }

    public void setAccessCodeRequired(Boolean accessCodeRequired) {
        this.accessCodeRequired = accessCodeRequired;
    }

    public OpenNode getNode() {
        return this.node;
    }

    public void setNode(OpenNode node) {
        this.node = node;
    }
}
