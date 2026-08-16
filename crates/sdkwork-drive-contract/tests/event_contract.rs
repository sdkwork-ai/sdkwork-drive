use serde_json::Value;

const ASYNC_API: &str = include_str!("../../../apis/events/drive/drive-events.asyncapi.json");

#[test]
fn drive_async_api_declares_complete_node_lifecycle_contract_without_provider_secrets() {
    let document: Value = serde_json::from_str(ASYNC_API).expect("AsyncAPI must be valid JSON");
    assert_eq!(document["asyncapi"], "3.0.0");
    for (channel, event_type) in [
        ("nodeVersionCommitted", "drive.node.version.committed.v1"),
        ("nodePathChanged", "drive.node.path.changed.v1"),
        (
            "nodeEligibilityChanged",
            "drive.node.eligibility.changed.v1",
        ),
        ("nodeDeleted", "drive.node.deleted.v1"),
    ] {
        assert_eq!(document["channels"][channel]["address"], event_type);
    }

    let data = &document["components"]["schemas"]["DriveNodeVersionCommittedV1Data"];
    let required = data["required"]
        .as_array()
        .expect("event data required fields")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    for field in [
        "driveVersionId",
        "versionNo",
        "spaceRelativePath",
        "rootScopes",
    ] {
        assert!(required.contains(&field), "missing required field {field}");
    }

    let path_data = &document["components"]["schemas"]["DriveNodePathChangedV1Data"];
    let path_required = path_data["required"]
        .as_array()
        .expect("path event data required fields")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    for field in [
        "oldSpaceRelativePath",
        "newSpaceRelativePath",
        "oldRootScopes",
        "newRootScopes",
    ] {
        assert!(
            path_required.contains(&field),
            "missing path event field {field}"
        );
    }

    let eligibility_data = &document["components"]["schemas"]["DriveNodeEligibilityChangedV1Data"];
    assert_eq!(
        eligibility_data["properties"]["rootScopes"]["$ref"],
        "#/components/schemas/DriveRootScopeEffects"
    );
    let deleted_data = &document["components"]["schemas"]["DriveNodeDeletedV1Data"];
    assert_eq!(
        deleted_data["properties"]["rootScopes"]["$ref"],
        "#/components/schemas/DriveRootScopeEffects"
    );

    let source = ASYNC_API.to_ascii_lowercase();
    for forbidden in ["objectkey", "bucketname", "presignedurl", "credential"] {
        assert!(
            !source.contains(forbidden),
            "AsyncAPI must not expose provider field {forbidden}"
        );
    }
}
