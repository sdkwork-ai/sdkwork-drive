package com.sdkwork.drive.admin.storage.sdk.generated.java.model;


public class CopyProviderObjectRequest {
    private String sourceObjectKey;
    private String destinationObjectKey;
    private String destinationBucket;
    private String metadataDirective;

    public String getSourceObjectKey() {
        return this.sourceObjectKey;
    }

    public void setSourceObjectKey(String sourceObjectKey) {
        this.sourceObjectKey = sourceObjectKey;
    }

    public String getDestinationObjectKey() {
        return this.destinationObjectKey;
    }

    public void setDestinationObjectKey(String destinationObjectKey) {
        this.destinationObjectKey = destinationObjectKey;
    }

    public String getDestinationBucket() {
        return this.destinationBucket;
    }

    public void setDestinationBucket(String destinationBucket) {
        this.destinationBucket = destinationBucket;
    }

    public String getMetadataDirective() {
        return this.metadataDirective;
    }

    public void setMetadataDirective(String metadataDirective) {
        this.metadataDirective = metadataDirective;
    }
}
