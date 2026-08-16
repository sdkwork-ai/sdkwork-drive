package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;

public class ExtractArchiveEntriesRequest {
    private List<String> entryPaths;
    private String targetParentNodeId;

    public List<String> getEntryPaths() {
        return this.entryPaths;
    }

    public void setEntryPaths(List<String> entryPaths) {
        this.entryPaths = entryPaths;
    }

    public String getTargetParentNodeId() {
        return this.targetParentNodeId;
    }

    public void setTargetParentNodeId(String targetParentNodeId) {
        this.targetParentNodeId = targetParentNodeId;
    }
}
