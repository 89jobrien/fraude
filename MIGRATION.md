# Migration Guide

## claw → fraude rename

This project was renamed from **Claw Code** to **Fraude**. If you have an existing
installation, the following changes may require manual steps.

### Environment variables

All `CLAW_*` environment variables have been renamed to `FRAUDE_*`.

| Old name                       | New name                         |
| ------------------------------ | -------------------------------- |
| `CLAW_CONFIG_HOME`             | `FRAUDE_CONFIG_HOME`             |
| `CLAW_CODE_REMOTE`             | `FRAUDE_REMOTE`                  |
| `CLAW_CODE_REMOTE_SESSION_ID`  | `FRAUDE_REMOTE_SESSION_ID`       |
| `CLAW_CODE_UPSTREAM`           | `FRAUDE_UPSTREAM`                |
| `CLAW_MODEL`                   | `FRAUDE_MODEL`                   |
| `CLAW_PLUGIN_ID`               | `FRAUDE_PLUGIN_ID`               |
| `CLAW_PLUGIN_NAME`             | `FRAUDE_PLUGIN_NAME`             |
| `CLAW_PLUGIN_ROOT`             | `FRAUDE_PLUGIN_ROOT`             |
| `CLAW_TOOL_NAME`               | `FRAUDE_TOOL_NAME`               |
| `CLAW_TOOL_INPUT`              | `FRAUDE_TOOL_INPUT`              |
| `CLAW_PERMISSION_MODE`         | `FRAUDE_PERMISSION_MODE`         |
| `CLAW_SANDBOX_FILESYSTEM_MODE` | `FRAUDE_SANDBOX_FILESYSTEM_MODE` |
| `CLAW_SANDBOX_ALLOWED_MOUNTS`  | `FRAUDE_SANDBOX_ALLOWED_MOUNTS`  |
| `CLAW_WEB_SEARCH_BASE_URL`     | `FRAUDE_WEB_SEARCH_BASE_URL`     |
| `CLAW_TODO_STORE`              | `FRAUDE_TODO_STORE`              |
| `CLAW_AGENT_STORE`             | `FRAUDE_AGENT_STORE`             |

Update any `.env` files, CI workflows, Docker Compose configs, and shell scripts
that export these variables.

### Configuration files

Config files and directories have been renamed. The runtime reads both old and new
locations during the transition period (new location takes priority).

| Old path                    | New path                      |
| --------------------------- | ----------------------------- |
| `.claw.json`                | `.fraude.json`                |
| `.claw/settings.json`       | `.fraude/settings.json`       |
| `.claw/settings.local.json` | `.fraude/settings.local.json` |
| `~/.claw/`                  | `~/.fraude/`                  |

To migrate: copy your existing config to the new location.

```sh
cp .claw.json .fraude.json
cp -r .claw/ .fraude/
cp -r ~/.claw/ ~/.fraude/
```

### Task and agent store files

| Old path           | New path             |
| ------------------ | -------------------- |
| `.claw-todos.json` | `.fraude-todos.json` |
| `.claw-agents/`    | `.fraude-agents/`    |

The runtime falls back to the old path if the new path does not yet exist.

### Plugin directories

Plugin manifests previously lived in `.claw-plugin/plugin.json` inside the plugin
source directory. They now live in `.fraude-plugin/plugin.json`.

The runtime falls back to `.claw-plugin/` if `.fraude-plugin/` is absent, so
**existing installed plugins continue to work**. To migrate a plugin permanently:

```sh
mv .claw-plugin .fraude-plugin
```

### Plugin subprocess environment variables

Plugin scripts that read `$CLAW_TOOL_INPUT`, `$CLAW_PLUGIN_NAME`, etc. will receive
both old (`CLAW_*`) and new (`FRAUDE_*`) names during the transition window so that
existing scripts keep working without modification.

Update plugin scripts to read `$FRAUDE_TOOL_INPUT`, `$FRAUDE_PLUGIN_NAME`, etc.
Support for the old names will be removed in a future release.

### OAuth scope

The OAuth scope changed from `user:sessions:claw_code` to `user:sessions:fraude`.
Both scopes are accepted during the transition period. Existing tokens remain valid.

### Public API

The following identifiers changed in the `runtime` crate public API:

| Old name                        | New name                          |
| ------------------------------- | --------------------------------- |
| `BootstrapPlan::claw_default()` | `BootstrapPlan::fraude_default()` |
| `CLAW_SETTINGS_SCHEMA_NAME`     | `FRAUDE_SETTINGS_SCHEMA_NAME`     |
