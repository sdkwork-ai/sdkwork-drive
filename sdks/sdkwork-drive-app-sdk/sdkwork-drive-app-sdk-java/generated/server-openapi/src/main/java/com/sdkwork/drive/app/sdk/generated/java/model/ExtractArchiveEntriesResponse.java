package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;

public class ExtractArchiveEntriesResponse {
    private List<DriveNode> items;
    private String extractedCount;

    public List<DriveNode> getItems() {
        return this.items;
    }

    public void setItems(List<DriveNode> items) {
        this.items = items;
    }

    public String getExtractedCount() {
        return this.extractedCount;
    }

    public void setExtractedCount(String extractedCount) {
        this.extractedCount = extractedCount;
    }
}
