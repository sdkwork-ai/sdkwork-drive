package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;

public class CompleteUploadSessionRequest {
    private String uploadId;
    private String contentType;
    private Integer contentLength;
    private String checksumSha256Hex;
    private List<CompletedUploadPart> parts;

    public String getUploadId() {
        return this.uploadId;
    }

    public void setUploadId(String uploadId) {
        this.uploadId = uploadId;
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

    public List<CompletedUploadPart> getParts() {
        return this.parts;
    }

    public void setParts(List<CompletedUploadPart> parts) {
        this.parts = parts;
    }
}
