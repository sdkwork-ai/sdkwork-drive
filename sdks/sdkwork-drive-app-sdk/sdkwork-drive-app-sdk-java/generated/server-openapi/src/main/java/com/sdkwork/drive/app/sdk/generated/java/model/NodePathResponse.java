package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;

public class NodePathResponse {
    private List<DriveNode> items;
    private List<String> pathSegments;

    public List<DriveNode> getItems() {
        return this.items;
    }

    public void setItems(List<DriveNode> items) {
        this.items = items;
    }

    public List<String> getPathSegments() {
        return this.pathSegments;
    }

    public void setPathSegments(List<String> pathSegments) {
        this.pathSegments = pathSegments;
    }
}
