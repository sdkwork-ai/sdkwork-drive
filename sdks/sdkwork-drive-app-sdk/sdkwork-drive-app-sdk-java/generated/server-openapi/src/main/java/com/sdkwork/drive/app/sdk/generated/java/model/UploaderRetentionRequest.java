package com.sdkwork.drive.app.sdk.generated.java.model;


public class UploaderRetentionRequest {
    private String mode;
    private Integer ttlSeconds;
    private String cleanupAction;
    private Integer hardDeleteAfterSeconds;

    public String getMode() {
        return this.mode;
    }

    public void setMode(String mode) {
        this.mode = mode;
    }

    public Integer getTtlSeconds() {
        return this.ttlSeconds;
    }

    public void setTtlSeconds(Integer ttlSeconds) {
        this.ttlSeconds = ttlSeconds;
    }

    public String getCleanupAction() {
        return this.cleanupAction;
    }

    public void setCleanupAction(String cleanupAction) {
        this.cleanupAction = cleanupAction;
    }

    public Integer getHardDeleteAfterSeconds() {
        return this.hardDeleteAfterSeconds;
    }

    public void setHardDeleteAfterSeconds(Integer hardDeleteAfterSeconds) {
        this.hardDeleteAfterSeconds = hardDeleteAfterSeconds;
    }
}
