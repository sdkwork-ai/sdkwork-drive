package com.sdkwork.drive.app.sdk.generated.java.model;


public class NodeLabel {
    private String id;
    private String tenantId;
    private String nodeId;
    private String labelId;
    private String lifecycleStatus;
    private Integer version;
    private DriveLabelSummary label;

    public String getId() {
        return this.id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getTenantId() {
        return this.tenantId;
    }

    public void setTenantId(String tenantId) {
        this.tenantId = tenantId;
    }

    public String getNodeId() {
        return this.nodeId;
    }

    public void setNodeId(String nodeId) {
        this.nodeId = nodeId;
    }

    public String getLabelId() {
        return this.labelId;
    }

    public void setLabelId(String labelId) {
        this.labelId = labelId;
    }

    public String getLifecycleStatus() {
        return this.lifecycleStatus;
    }

    public void setLifecycleStatus(String lifecycleStatus) {
        this.lifecycleStatus = lifecycleStatus;
    }

    public Integer getVersion() {
        return this.version;
    }

    public void setVersion(Integer version) {
        this.version = version;
    }

    public DriveLabelSummary getLabel() {
        return this.label;
    }

    public void setLabel(DriveLabelSummary label) {
        this.label = label;
    }
}
