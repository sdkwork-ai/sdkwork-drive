package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;

public class DownloadPackageResponse {
    private String id;
    private String tenantId;
    private String packageName;
    private String state;
    private String storageProviderId;
    private String bucket;
    private String archiveObjectKey;
    private String contentType;
    private Integer fileCount;
    private Integer totalBytes;
    private Integer archiveSizeBytes;
    private Integer expiresAtEpochMs;
    private String downloadUrl;
    private String signedSourceUrl;
    private String method;
    private List<DownloadPackageItem> items;

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

    public String getPackageName() {
        return this.packageName;
    }

    public void setPackageName(String packageName) {
        this.packageName = packageName;
    }

    public String getState() {
        return this.state;
    }

    public void setState(String state) {
        this.state = state;
    }

    public String getStorageProviderId() {
        return this.storageProviderId;
    }

    public void setStorageProviderId(String storageProviderId) {
        this.storageProviderId = storageProviderId;
    }

    public String getBucket() {
        return this.bucket;
    }

    public void setBucket(String bucket) {
        this.bucket = bucket;
    }

    public String getArchiveObjectKey() {
        return this.archiveObjectKey;
    }

    public void setArchiveObjectKey(String archiveObjectKey) {
        this.archiveObjectKey = archiveObjectKey;
    }

    public String getContentType() {
        return this.contentType;
    }

    public void setContentType(String contentType) {
        this.contentType = contentType;
    }

    public Integer getFileCount() {
        return this.fileCount;
    }

    public void setFileCount(Integer fileCount) {
        this.fileCount = fileCount;
    }

    public Integer getTotalBytes() {
        return this.totalBytes;
    }

    public void setTotalBytes(Integer totalBytes) {
        this.totalBytes = totalBytes;
    }

    public Integer getArchiveSizeBytes() {
        return this.archiveSizeBytes;
    }

    public void setArchiveSizeBytes(Integer archiveSizeBytes) {
        this.archiveSizeBytes = archiveSizeBytes;
    }

    public Integer getExpiresAtEpochMs() {
        return this.expiresAtEpochMs;
    }

    public void setExpiresAtEpochMs(Integer expiresAtEpochMs) {
        this.expiresAtEpochMs = expiresAtEpochMs;
    }

    public String getDownloadUrl() {
        return this.downloadUrl;
    }

    public void setDownloadUrl(String downloadUrl) {
        this.downloadUrl = downloadUrl;
    }

    public String getSignedSourceUrl() {
        return this.signedSourceUrl;
    }

    public void setSignedSourceUrl(String signedSourceUrl) {
        this.signedSourceUrl = signedSourceUrl;
    }

    public String getMethod() {
        return this.method;
    }

    public void setMethod(String method) {
        this.method = method;
    }

    public List<DownloadPackageItem> getItems() {
        return this.items;
    }

    public void setItems(List<DownloadPackageItem> items) {
        this.items = items;
    }
}
