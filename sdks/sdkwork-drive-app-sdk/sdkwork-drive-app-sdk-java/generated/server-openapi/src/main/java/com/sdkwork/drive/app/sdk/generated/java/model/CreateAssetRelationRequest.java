package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.Map;

public class CreateAssetRelationRequest {
    private String relatedAssetId;
    private String relationType;
    private String sourceDomain;
    private String sourceResourceType;
    private String sourceResourceId;
    private Map<String, Object> metadata;

    public String getRelatedAssetId() {
        return this.relatedAssetId;
    }

    public void setRelatedAssetId(String relatedAssetId) {
        this.relatedAssetId = relatedAssetId;
    }

    public String getRelationType() {
        return this.relationType;
    }

    public void setRelationType(String relationType) {
        this.relationType = relationType;
    }

    public String getSourceDomain() {
        return this.sourceDomain;
    }

    public void setSourceDomain(String sourceDomain) {
        this.sourceDomain = sourceDomain;
    }

    public String getSourceResourceType() {
        return this.sourceResourceType;
    }

    public void setSourceResourceType(String sourceResourceType) {
        this.sourceResourceType = sourceResourceType;
    }

    public String getSourceResourceId() {
        return this.sourceResourceId;
    }

    public void setSourceResourceId(String sourceResourceId) {
        this.sourceResourceId = sourceResourceId;
    }

    public Map<String, Object> getMetadata() {
        return this.metadata;
    }

    public void setMetadata(Map<String, Object> metadata) {
        this.metadata = metadata;
    }
}
