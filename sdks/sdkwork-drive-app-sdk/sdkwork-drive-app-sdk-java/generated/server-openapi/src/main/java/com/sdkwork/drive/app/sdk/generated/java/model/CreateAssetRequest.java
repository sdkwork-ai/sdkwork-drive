package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;
import java.util.Map;

public class CreateAssetRequest {
    private String driveNodeId;
    private Map<String, Object> virtualReference;
    private String title;
    private String description;
    private String scene;
    private String source;
    private List<String> tags;

    public String getDriveNodeId() {
        return this.driveNodeId;
    }

    public void setDriveNodeId(String driveNodeId) {
        this.driveNodeId = driveNodeId;
    }

    public Map<String, Object> getVirtualReference() {
        return this.virtualReference;
    }

    public void setVirtualReference(Map<String, Object> virtualReference) {
        this.virtualReference = virtualReference;
    }

    public String getTitle() {
        return this.title;
    }

    public void setTitle(String title) {
        this.title = title;
    }

    public String getDescription() {
        return this.description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public String getScene() {
        return this.scene;
    }

    public void setScene(String scene) {
        this.scene = scene;
    }

    public String getSource() {
        return this.source;
    }

    public void setSource(String source) {
        this.source = source;
    }

    public List<String> getTags() {
        return this.tags;
    }

    public void setTags(List<String> tags) {
        this.tags = tags;
    }
}
