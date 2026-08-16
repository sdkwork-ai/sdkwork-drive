package com.sdkwork.drive.app.sdk.generated.java.model;


public class EffectivePermission {
    private String id;
    private String tenantId;
    private String targetNodeId;
    private String nodeId;
    private String subjectType;
    private String subjectId;
    private String role;
    private Boolean inherited;
    private String inheritedFromNodeId;
    private String lifecycleStatus;
    private Integer version;

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

    public String getTargetNodeId() {
        return this.targetNodeId;
    }

    public void setTargetNodeId(String targetNodeId) {
        this.targetNodeId = targetNodeId;
    }

    public String getNodeId() {
        return this.nodeId;
    }

    public void setNodeId(String nodeId) {
        this.nodeId = nodeId;
    }

    public String getSubjectType() {
        return this.subjectType;
    }

    public void setSubjectType(String subjectType) {
        this.subjectType = subjectType;
    }

    public String getSubjectId() {
        return this.subjectId;
    }

    public void setSubjectId(String subjectId) {
        this.subjectId = subjectId;
    }

    public String getRole() {
        return this.role;
    }

    public void setRole(String role) {
        this.role = role;
    }

    public Boolean getInherited() {
        return this.inherited;
    }

    public void setInherited(Boolean inherited) {
        this.inherited = inherited;
    }

    public String getInheritedFromNodeId() {
        return this.inheritedFromNodeId;
    }

    public void setInheritedFromNodeId(String inheritedFromNodeId) {
        this.inheritedFromNodeId = inheritedFromNodeId;
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
}
