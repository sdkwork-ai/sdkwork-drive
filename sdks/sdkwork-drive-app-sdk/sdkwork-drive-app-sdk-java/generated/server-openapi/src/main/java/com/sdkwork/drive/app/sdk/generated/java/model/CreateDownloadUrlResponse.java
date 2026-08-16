package com.sdkwork.drive.app.sdk.generated.java.model;


public class CreateDownloadUrlResponse {
    private String downloadUrl;
    private String signedSourceUrl;
    private Integer expiresAtEpochMs;
    private String method;

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
}
