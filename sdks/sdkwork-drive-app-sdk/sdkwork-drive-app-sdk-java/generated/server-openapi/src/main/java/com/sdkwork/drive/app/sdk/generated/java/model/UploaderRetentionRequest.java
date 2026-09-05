package com.sdkwork.drive.app.sdk.generated.java.model;


public class UploaderRetentionRequest {
    private String mode;
    private String ttlSeconds;
    private String cleanupAction;
    private String hardDeleteAfterSeconds;

    public String getMode() {
        return this.mode;
    }

    public void setMode(String mode) {
        this.mode = mode;
    }

    public String getTtlSeconds() {
        return this.ttlSeconds;
    }

    public void setTtlSeconds(String ttlSeconds) {
        this.ttlSeconds = ttlSeconds;
    }

    public String getCleanupAction() {
        return this.cleanupAction;
    }

    public void setCleanupAction(String cleanupAction) {
        this.cleanupAction = cleanupAction;
    }

    public String getHardDeleteAfterSeconds() {
        return this.hardDeleteAfterSeconds;
    }

    public void setHardDeleteAfterSeconds(String hardDeleteAfterSeconds) {
        this.hardDeleteAfterSeconds = hardDeleteAfterSeconds;
    }
}
