# Local Sandbox File-System Provider Specs

Machine-readable component authority: `component.spec.json`.

The provider consumes only an authorization-projected `AuthorizedSandboxMount`, keeps the
operator-configured root reference private, and exposes logical root-relative entries through the
workspace-service `DriveSandboxDirectoryProvider` and `DriveSandboxFileSystemProvider` ports.

The file-system port owns bounded UTF-8/binary reads, atomic file publication and replacement,
move/rename, explicit recursive deletion, optimistic revision checks, and capability-relative path
containment. Symbolic links are excluded from public entries and rejected as navigation or mutation
targets.
