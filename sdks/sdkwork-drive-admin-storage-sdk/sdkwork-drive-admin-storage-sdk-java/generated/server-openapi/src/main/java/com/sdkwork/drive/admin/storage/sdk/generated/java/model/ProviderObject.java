package com.sdkwork.drive.admin.storage.sdk.generated.java.model;


public class ProviderObject {
    private String providerId;
    private String bucket;
    private String objectKind;
    private String objectKey;
    private Integer contentLength;
    private String contentType;
    private String etag;
    private String versionId;
    private String storageClass;
    private Integer lastModifiedEpochMs;

    public String getProviderId() {
        return this.providerId;
    }

    public void setProviderId(String providerId) {
        this.providerId = providerId;
    }

    public String getBucket() {
        return this.bucket;
    }

    public void setBucket(String bucket) {
        this.bucket = bucket;
    }

    public String getObjectKind() {
        return this.objectKind;
    }

    public void setObjectKind(String objectKind) {
        this.objectKind = objectKind;
    }

    public String getObjectKey() {
        return this.objectKey;
    }

    public void setObjectKey(String objectKey) {
        this.objectKey = objectKey;
    }

    public Integer getContentLength() {
        return this.contentLength;
    }

    public void setContentLength(Integer contentLength) {
        this.contentLength = contentLength;
    }

    public String getContentType() {
        return this.contentType;
    }

    public void setContentType(String contentType) {
        this.contentType = contentType;
    }

    public String getEtag() {
        return this.etag;
    }

    public void setEtag(String etag) {
        this.etag = etag;
    }

    public String getVersionId() {
        return this.versionId;
    }

    public void setVersionId(String versionId) {
        this.versionId = versionId;
    }

    public String getStorageClass() {
        return this.storageClass;
    }

    public void setStorageClass(String storageClass) {
        this.storageClass = storageClass;
    }

    public Integer getLastModifiedEpochMs() {
        return this.lastModifiedEpochMs;
    }

    public void setLastModifiedEpochMs(Integer lastModifiedEpochMs) {
        this.lastModifiedEpochMs = lastModifiedEpochMs;
    }
}
