mod error;
mod hooks;
mod manifest;
mod manager;
mod registry;
mod types;

pub use error::{PluginError, PluginManifestValidationError};
pub use hooks::{HookEvent, HookRunResult, HookRunner};
pub use manifest::{
    PluginCommandManifest, PluginHooks, PluginLifecycle, PluginManifest, PluginToolManifest,
    load_plugin_from_directory,
};
pub use manager::{InstallOutcome, PluginManager, PluginManagerConfig, UpdateOutcome};
pub use registry::{
    InstalledPluginRecord, InstalledPluginRegistry, PluginRegistry, PluginSummary, RegisteredPlugin,
};
pub use types::{
    BuiltinPlugin, BundledPlugin, ExternalPlugin, Plugin, PluginDefinition, PluginInstallSource,
    PluginKind, PluginMetadata, PluginPermission, PluginTool, PluginToolDefinition,
    PluginToolPermission, builtin_plugins,
};
