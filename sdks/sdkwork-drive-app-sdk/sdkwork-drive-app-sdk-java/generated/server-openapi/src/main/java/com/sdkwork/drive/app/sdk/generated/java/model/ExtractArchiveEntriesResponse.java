package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;

public class ExtractArchiveEntriesResponse {
    private List<DriveNode> items;
    private Integer extractedCount;

    public List<DriveNode> getItems() {
        return this.items;
    }

    public void setItems(List<DriveNode> items) {
        this.items = items;
    }

    public Integer getExtractedCount() {
        return this.extractedCount;
    }

    public void setExtractedCount(Integer extractedCount) {
        this.extractedCount = extractedCount;
    }
}
