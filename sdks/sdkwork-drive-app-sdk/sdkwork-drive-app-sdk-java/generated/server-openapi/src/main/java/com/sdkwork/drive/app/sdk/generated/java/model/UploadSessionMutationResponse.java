package com.sdkwork.drive.app.sdk.generated.java.model;


public class UploadSessionMutationResponse {
    private String id;
    private String tenantId;
    private String spaceId;
    private String nodeId;
    private String bucket;
    private String objectKey;
    private String state;
    private Integer expiresAtEpochMs;
    private Integer version;
    private String storageProviderId;
    private String storageUploadId;

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

    public String getSpaceId() {
        return this.spaceId;
    }

    public void setSpaceId(String spaceId) {
        this.spaceId = spaceId;
    }

    public String getNodeId() {
        return this.nodeId;
    }

    public void setNodeId(String nodeId) {
        this.nodeId = nodeId;
    }

    public String getBucket() {
        return this.bucket;
    }

    public void setBucket(String bucket) {
        this.bucket = bucket;
    }

    public String getObjectKey() {
        return this.objectKey;
    }

    public void setObjectKey(String objectKey) {
        this.objectKey = objectKey;
    }

    public String getState() {
        return this.state;
    }

    public void setState(String state) {
        this.state = state;
    }

    public Integer getExpiresAtEpochMs() {
        return this.expiresAtEpochMs;
    }

    public void setExpiresAtEpochMs(Integer expiresAtEpochMs) {
        this.expiresAtEpochMs = expiresAtEpochMs;
    }

    public Integer getVersion() {
        return this.version;
    }

    public void setVersion(Integer version) {
        this.version = version;
    }

    public String getStorageProviderId() {
        return this.storageProviderId;
    }

    public void setStorageProviderId(String storageProviderId) {
        this.storageProviderId = storageProviderId;
    }

    public String getStorageUploadId() {
        return this.storageUploadId;
    }

    public void setStorageUploadId(String storageUploadId) {
        this.storageUploadId = storageUploadId;
    }
}
