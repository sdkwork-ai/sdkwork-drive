package com.sdkwork.drive.app.sdk.generated.java.model;


public class UpdateShareLinkRequest {
    private String role;
    private Integer expiresAtEpochMs;
    private Integer downloadLimit;

    public String getRole() {
        return this.role;
    }

    public void setRole(String role) {
        this.role = role;
    }

    public Integer getExpiresAtEpochMs() {
        return this.expiresAtEpochMs;
    }

    public void setExpiresAtEpochMs(Integer expiresAtEpochMs) {
        this.expiresAtEpochMs = expiresAtEpochMs;
    }

    public Integer getDownloadLimit() {
        return this.downloadLimit;
    }

    public void setDownloadLimit(Integer downloadLimit) {
        this.downloadLimit = downloadLimit;
    }
}
