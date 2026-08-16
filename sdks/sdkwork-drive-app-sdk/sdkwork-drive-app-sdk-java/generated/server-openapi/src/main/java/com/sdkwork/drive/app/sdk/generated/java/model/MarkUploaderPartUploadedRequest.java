package com.sdkwork.drive.app.sdk.generated.java.model;


public class MarkUploaderPartUploadedRequest {
    private String uploadSessionId;
    private Integer offsetBytes;
    private Integer sizeBytes;
    private String etag;
    private String checksumSha256Hex;
    private Integer uploadedAtEpochMs;

    public String getUploadSessionId() {
        return this.uploadSessionId;
    }

    public void setUploadSessionId(String uploadSessionId) {
        this.uploadSessionId = uploadSessionId;
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

    public Integer getUploadedAtEpochMs() {
        return this.uploadedAtEpochMs;
    }

    public void setUploadedAtEpochMs(Integer uploadedAtEpochMs) {
        this.uploadedAtEpochMs = uploadedAtEpochMs;
    }
}
