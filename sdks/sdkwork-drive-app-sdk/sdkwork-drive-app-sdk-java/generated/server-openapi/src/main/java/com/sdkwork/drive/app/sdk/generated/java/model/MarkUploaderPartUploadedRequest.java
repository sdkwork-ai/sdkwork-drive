package com.sdkwork.drive.app.sdk.generated.java.model;


public class MarkUploaderPartUploadedRequest {
    private String uploadSessionId;
    private String offsetBytes;
    private String sizeBytes;
    private String etag;
    private String checksumSha256Hex;
    private String uploadedAtEpochMs;

    public String getUploadSessionId() {
        return this.uploadSessionId;
    }

    public void setUploadSessionId(String uploadSessionId) {
        this.uploadSessionId = uploadSessionId;
    }

    public String getOffsetBytes() {
        return this.offsetBytes;
    }

    public void setOffsetBytes(String offsetBytes) {
        this.offsetBytes = offsetBytes;
    }

    public String getSizeBytes() {
        return this.sizeBytes;
    }

    public void setSizeBytes(String sizeBytes) {
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

    public String getUploadedAtEpochMs() {
        return this.uploadedAtEpochMs;
    }

    public void setUploadedAtEpochMs(String uploadedAtEpochMs) {
        this.uploadedAtEpochMs = uploadedAtEpochMs;
    }
}
