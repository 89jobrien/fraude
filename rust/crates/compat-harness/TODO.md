# TODO

- **Version gating**: check the upstream `fraude` version before extraction and emit
  a warning (or refuse) when the version is outside the tested range.
- **Schema versioning**: add a `schema_version` field to `ExtractedManifest` so consumers
  can detect format drift without trial-and-error deserialization.
- **Cached extraction**: optionally write the extracted manifest to a sidecar file and
  skip re-extraction when the upstream mtime has not changed.
- **Test fixtures**: add a mock upstream directory tree to the test suite so extraction
  logic can be exercised without a real `fraude` installation.
