package com.sdkwork.drive.sdk.generated.java.model;


public class DriveOpenShareLink {
    private String id;
    private String tenantId;
    private String role;
    private Integer expiresAtEpochMs;
    private Integer downloadLimit;
    private Integer downloadCount;
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

    public Integer getExpiresAtEpochMs() {
        return this.expiresAtEpochMs;
    }

    public void setExpiresAtEpochMs(Integer expiresAtEpochMs) {
        this.expiresAtEpochMs = expiresAtEpochMs;
    }

    public Integer getDownloadLimit() {
        return this.downloadLimit;
    }

    public void setDownloadLimit(Integer downloadLimit) {
        this.downloadLimit = downloadLimit;
    }

    public Integer getDownloadCount() {
        return this.downloadCount;
    }

    public void setDownloadCount(Integer downloadCount) {
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
