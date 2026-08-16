package com.sdkwork.drive.app.sdk.generated.java.model;


public class CreateShareLinkResponse {
    private String id;
    private String tenantId;
    private String nodeId;
    private String role;
    private Integer expiresAtEpochMs;
    private Integer downloadLimit;
    private Integer downloadCount;
    private Boolean accessCodeRequired;
    private String lifecycleStatus;
    private Integer version;
    private String token;
    private String accessCode;

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

    public String getNodeId() {
        return this.nodeId;
    }

    public void setNodeId(String nodeId) {
        this.nodeId = nodeId;
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

    public String getLifecycleStatus() {
        return this.lifecycleStatus;
    }

    public void setLifecycleStatus(String lifecycleStatus) {
        this.lifecycleStatus = lifecycleStatus;
    }

    public Integer getVersion() {
        return this.version;
    }

    public void setVersion(Integer version) {
        this.version = version;
    }

    public String getToken() {
        return this.token;
    }

    public void setToken(String token) {
        this.token = token;
    }

    public String getAccessCode() {
        return this.accessCode;
    }

    public void setAccessCode(String accessCode) {
        this.accessCode = accessCode;
    }
}
