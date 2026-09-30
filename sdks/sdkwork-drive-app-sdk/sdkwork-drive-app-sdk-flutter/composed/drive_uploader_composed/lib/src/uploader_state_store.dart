/// Resumable upload state. Drive server state stays authoritative and is
/// re-queried during prepare; the store only saves local round-trips for parts
/// the Drive prepare response has not yet confirmed.
abstract class DriveUploaderStateStore {
  /// Marks one part as uploaded for the task. Idempotent per (taskId, partNo).
  Future<void> markPartUploaded(
    String taskId,
    int partNo,
    String etag, {
    required int offsetBytes,
    required int sizeBytes,
  });

  /// Returns the etags already known for the task, keyed by part number.
  Future<Map<int, String>> completedParts(String taskId);

  /// Clears the task state after completion or abort.
  Future<void> clear(String taskId);
}

class InMemoryDriveUploaderStateStore implements DriveUploaderStateStore {
  final Map<String, Map<int, _StoredPart>> _tasks = <String, Map<int, _StoredPart>>{};

  @override
  Future<void> markPartUploaded(
    String taskId,
    int partNo,
    String etag, {
    required int offsetBytes,
    required int sizeBytes,
  }) async {
    final parts = _tasks.putIfAbsent(taskId, () => <int, _StoredPart>{});
    parts[partNo] = _StoredPart(
      etag: etag,
      offsetBytes: offsetBytes,
      sizeBytes: sizeBytes,
    );
  }

  @override
  Future<Map<int, String>> completedParts(String taskId) async =>
      (_tasks[taskId] ?? const <int, _StoredPart>{})
          .map((partNo, part) => MapEntry(partNo, part.etag));

  @override
  Future<void> clear(String taskId) async {
    _tasks.remove(taskId);
  }
}

class _StoredPart {
  const _StoredPart({
    required this.etag,
    required this.offsetBytes,
    required this.sizeBytes,
  });

  final String etag;
  final int offsetBytes;
  final int sizeBytes;
}
