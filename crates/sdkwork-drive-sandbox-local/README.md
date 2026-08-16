# Drive Sandbox Local Provider

Private server-side file-system access for operator-configured Drive sandbox roots.

The provider:

- accepts an authorization-projected private root reference and canonical logical relative paths;
- uses capability-based directory handles to contain reads and writes beneath the configured root;
- excludes symbolic links from explorer results and rejects symbolic-link escape navigation;
- returns stable, bounded cursor pages ordered by entry name;
- validates entry names against the portable Windows, macOS, and Linux subset;
- supports bounded UTF-8 and binary reads, atomic file create/update, move/rename, and explicit
  recursive deletion with optimistic revisions; and
- never serializes or includes the physical root in entry identities, revisions, errors, or debug output.
