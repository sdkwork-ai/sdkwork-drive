package com.sdkwork.drive.app.sdk.generated.java.model;


public class FileVersion {
    private String id;
    private String tenantId;
    private String nodeId;
    private String storageObjectId;
    private Integer versionNo;
    private String contentType;
    private Integer contentLength;
    private String checksumSha256Hex;
    private String lifecycleStatus;
    private String createdAt;

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

    public String getStorageObjectId() {
        return this.storageObjectId;
    }

    public void setStorageObjectId(String storageObjectId) {
        this.storageObjectId = storageObjectId;
    }

    public Integer getVersionNo() {
        return this.versionNo;
    }

    public void setVersionNo(Integer versionNo) {
        this.versionNo = versionNo;
    }

    public String getContentType() {
        return this.contentType;
    }

    public void setContentType(String contentType) {
        this.contentType = contentType;
    }

    public Integer getContentLength() {
        return this.contentLength;
    }

    public void setContentLength(Integer contentLength) {
        this.contentLength = contentLength;
    }

    public String getChecksumSha256Hex() {
        return this.checksumSha256Hex;
    }

    public void setChecksumSha256Hex(String checksumSha256Hex) {
        this.checksumSha256Hex = checksumSha256Hex;
    }

    public String getLifecycleStatus() {
        return this.lifecycleStatus;
    }

    public void setLifecycleStatus(String lifecycleStatus) {
        this.lifecycleStatus = lifecycleStatus;
    }

    public String getCreatedAt() {
        return this.createdAt;
    }

    public void setCreatedAt(String createdAt) {
        this.createdAt = createdAt;
    }
}
