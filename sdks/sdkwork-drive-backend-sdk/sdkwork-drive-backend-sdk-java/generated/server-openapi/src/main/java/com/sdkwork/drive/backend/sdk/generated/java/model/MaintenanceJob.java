package com.sdkwork.drive.backend.sdk.generated.java.model;


public class MaintenanceJob {
    private Integer id;
    private String jobType;
    private String status;
    private Boolean dryRun;
    private Integer scannedCount;
    private Integer affectedCount;
    private String operatorId;
    private String correlationId;
    private String traceId;
    private String errorMessage;
    private String startedAt;
    private String finishedAt;
    private String createdAt;

    public Integer getId() {
        return this.id;
    }

    public void setId(Integer id) {
        this.id = id;
    }

    public String getJobType() {
        return this.jobType;
    }

    public void setJobType(String jobType) {
        this.jobType = jobType;
    }

    public String getStatus() {
        return this.status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public Boolean getDryRun() {
        return this.dryRun;
    }

    public void setDryRun(Boolean dryRun) {
        this.dryRun = dryRun;
    }

    public Integer getScannedCount() {
        return this.scannedCount;
    }

    public void setScannedCount(Integer scannedCount) {
        this.scannedCount = scannedCount;
    }

    public Integer getAffectedCount() {
        return this.affectedCount;
    }

    public void setAffectedCount(Integer affectedCount) {
        this.affectedCount = affectedCount;
    }

    public String getOperatorId() {
        return this.operatorId;
    }

    public void setOperatorId(String operatorId) {
        this.operatorId = operatorId;
    }

    public String getCorrelationId() {
        return this.correlationId;
    }

    public void setCorrelationId(String correlationId) {
        this.correlationId = correlationId;
    }

    public String getTraceId() {
        return this.traceId;
    }

    public void setTraceId(String traceId) {
        this.traceId = traceId;
    }

    public String getErrorMessage() {
        return this.errorMessage;
    }

    public void setErrorMessage(String errorMessage) {
        this.errorMessage = errorMessage;
    }

    public String getStartedAt() {
        return this.startedAt;
    }

    public void setStartedAt(String startedAt) {
        this.startedAt = startedAt;
    }

    public String getFinishedAt() {
        return this.finishedAt;
    }

    public void setFinishedAt(String finishedAt) {
        this.finishedAt = finishedAt;
    }

    public String getCreatedAt() {
        return this.createdAt;
    }

    public void setCreatedAt(String createdAt) {
        this.createdAt = createdAt;
    }
}
