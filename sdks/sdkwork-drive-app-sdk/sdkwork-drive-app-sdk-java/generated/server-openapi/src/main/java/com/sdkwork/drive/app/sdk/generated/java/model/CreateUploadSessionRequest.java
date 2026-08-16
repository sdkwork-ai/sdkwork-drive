package com.sdkwork.drive.app.sdk.generated.java.model;


public class CreateUploadSessionRequest {
    private String sessionId;
    private String spaceId;
    private String nodeId;
    private String bucket;
    private String objectKey;
    private String idempotencyKey;
    private Integer expiresAtEpochMs;

    public String getSessionId() {
        return this.sessionId;
    }

    public void setSessionId(String sessionId) {
        this.sessionId = sessionId;
    }

    public String getSpaceId() {
        return this.spaceId;
    }

    public void setSpaceId(String spaceId) {
        this.spaceId = spaceId;
    }

    public String getNodeId() {
        return this.nodeId;
    }

    public void setNodeId(String nodeId) {
        this.nodeId = nodeId;
    }

    public String getBucket() {
        return this.bucket;
    }

    public void setBucket(String bucket) {
        this.bucket = bucket;
    }

    public String getObjectKey() {
        return this.objectKey;
    }

    public void setObjectKey(String objectKey) {
        this.objectKey = objectKey;
    }

    public String getIdempotencyKey() {
        return this.idempotencyKey;
    }

    public void setIdempotencyKey(String idempotencyKey) {
        this.idempotencyKey = idempotencyKey;
    }

    public Integer getExpiresAtEpochMs() {
        return this.expiresAtEpochMs;
    }

    public void setExpiresAtEpochMs(Integer expiresAtEpochMs) {
        this.expiresAtEpochMs = expiresAtEpochMs;
    }
}
