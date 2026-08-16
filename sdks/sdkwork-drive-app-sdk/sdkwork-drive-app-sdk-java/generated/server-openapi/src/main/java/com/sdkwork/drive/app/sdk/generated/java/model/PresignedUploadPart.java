package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.Map;

public class PresignedUploadPart {
    private String uploadUrl;
    private Integer expiresAtEpochMs;
    private String method;
    private Map<String, String> headers;
    private Integer partNo;
    private String uploadId;

    public String getUploadUrl() {
        return this.uploadUrl;
    }

    public void setUploadUrl(String uploadUrl) {
        this.uploadUrl = uploadUrl;
    }

    public Integer getExpiresAtEpochMs() {
        return this.expiresAtEpochMs;
    }

    public void setExpiresAtEpochMs(Integer expiresAtEpochMs) {
        this.expiresAtEpochMs = expiresAtEpochMs;
    }

    public String getMethod() {
        return this.method;
    }

    public void setMethod(String method) {
        this.method = method;
    }

    public Map<String, String> getHeaders() {
        return this.headers;
    }

    public void setHeaders(Map<String, String> headers) {
        this.headers = headers;
    }

    public Integer getPartNo() {
        return this.partNo;
    }

    public void setPartNo(Integer partNo) {
        this.partNo = partNo;
    }

    public String getUploadId() {
        return this.uploadId;
    }

    public void setUploadId(String uploadId) {
        this.uploadId = uploadId;
    }
}
