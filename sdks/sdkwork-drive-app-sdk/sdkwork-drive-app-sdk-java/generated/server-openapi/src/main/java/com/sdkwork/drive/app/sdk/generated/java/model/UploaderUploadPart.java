package com.sdkwork.drive.app.sdk.generated.java.model;


public class UploaderUploadPart {
    private String id;
    private String tenantId;
    private String uploadItemId;
    private String uploadSessionId;
    private Integer partNo;
    private Integer offsetBytes;
    private Integer sizeBytes;
    private String etag;
    private String checksumSha256Hex;
    private String status;
    private Integer retryCount;
    private Integer uploadedAtEpochMs;

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

    public String getUploadItemId() {
        return this.uploadItemId;
    }

    public void setUploadItemId(String uploadItemId) {
        this.uploadItemId = uploadItemId;
    }

    public String getUploadSessionId() {
        return this.uploadSessionId;
    }

    public void setUploadSessionId(String uploadSessionId) {
        this.uploadSessionId = uploadSessionId;
    }

    public Integer getPartNo() {
        return this.partNo;
    }

    public void setPartNo(Integer partNo) {
        this.partNo = partNo;
    }

    public Integer getOffsetBytes() {
        return this.offsetBytes;
    }

    public void setOffsetBytes(Integer offsetBytes) {
        this.offsetBytes = offsetBytes;
    }

    public Integer getSizeBytes() {
        return this.sizeBytes;
    }

    public void setSizeBytes(Integer sizeBytes) {
        this.sizeBytes = sizeBytes;
    }

    public String getEtag() {
        return this.etag;
    }

    public void setEtag(String etag) {
        this.etag = etag;
    }

    public String getChecksumSha256Hex() {
        return this.checksumSha256Hex;
    }

    public void setChecksumSha256Hex(String checksumSha256Hex) {
        this.checksumSha256Hex = checksumSha256Hex;
    }

    public String getStatus() {
        return this.status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public Integer getRetryCount() {
        return this.retryCount;
    }

    public void setRetryCount(Integer retryCount) {
        this.retryCount = retryCount;
    }

    public Integer getUploadedAtEpochMs() {
        return this.uploadedAtEpochMs;
    }

    public void setUploadedAtEpochMs(Integer uploadedAtEpochMs) {
        this.uploadedAtEpochMs = uploadedAtEpochMs;
    }
}
