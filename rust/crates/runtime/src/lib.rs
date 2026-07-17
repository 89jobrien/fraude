mod bash;
mod bootstrap;
mod compact;
mod config;
mod conversation;
mod file_ops;
mod hooks;
mod json;
mod mcp;
mod mcp_client;
mod mcp_stdio;
mod oauth;
mod permissions;
mod prompt;
mod remote;
pub mod sandbox;
mod session;
mod usage;

pub use bash::{BashCommandInput, BashCommandOutput, execute_bash};
pub use bootstrap::{BootstrapPhase, BootstrapPlan};
pub use compact::{
    CompactionConfig, CompactionResult, compact_session, estimate_session_tokens,
    format_compact_summary, get_compact_continuation_message, should_compact,
};
pub use config::{
    ConfigEntry, ConfigError, ConfigLoader, ConfigSource, FRAUDE_SETTINGS_SCHEMA_NAME,
    McpConfigCollection, McpManagedProxyServerConfig, McpOAuthConfig, McpRemoteServerConfig,
    McpSdkServerConfig, McpServerConfig, McpStdioServerConfig, McpTransport,
    McpWebSocketServerConfig, OAuthConfig, ResolvedPermissionMode, RuntimeConfig,
    RuntimeFeatureConfig, RuntimeHookConfig, RuntimePluginConfig, ScopedMcpServerConfig,
};
pub use conversation::{
    ApiClient, ApiRequest, AssistantEvent, ConversationRuntime, RuntimeError, StaticToolExecutor,
    ToolError, ToolExecutor, TurnSummary,
};
pub use file_ops::{
    EditFileOutput, GlobSearchOutput, GrepSearchInput, GrepSearchOutput, ReadFileOutput,
    StructuredPatchHunk, TextFilePayload, WriteFileOutput, edit_file, glob_search, grep_search,
    read_file, write_file,
};
pub use hooks::{HookEvent, HookRunResult, HookRunner};
pub use lsp::{
    FileDiagnostics, LspContextEnrichment, LspError, LspManager, LspServerConfig, SymbolLocation,
    WorkspaceDiagnostics,
};
pub use mcp::{
    mcp_server_signature, mcp_tool_name, mcp_tool_prefix, normalize_name_for_mcp,
    scoped_mcp_config_hash, unwrap_ccr_proxy_url,
};
pub use mcp_client::{
    McpClientAuth, McpClientBootstrap, McpClientTransport, McpManagedProxyTransport,
    McpRemoteTransport, McpSdkTransport, McpStdioTransport,
};
pub use mcp_stdio::{
    JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse, ManagedMcpTool,
    McpInitializeClientInfo, McpInitializeParams, McpInitializeResult, McpInitializeServerInfo,
    McpListResourcesParams, McpListResourcesResult, McpListToolsParams, McpListToolsResult,
    McpReadResourceParams, McpReadResourceResult, McpResource, McpResourceContents,
    McpServerManager, McpServerManagerError, McpStdioProcess, McpTool, McpToolCallContent,
    McpToolCallParams, McpToolCallResult, UnsupportedMcpServer, spawn_mcp_stdio_process,
};
#[cfg(feature = "fuzz")]
pub use oauth::fuzz_helpers as oauth_fuzz;
pub use oauth::{
    OAuthAuthorizationRequest, OAuthCallbackParams, OAuthRefreshRequest, OAuthTokenExchangeRequest,
    OAuthTokenSet, PkceChallengeMethod, PkceCodePair, clear_oauth_credentials, code_challenge_s256,
    credentials_path, generate_pkce_pair, generate_state, load_oauth_credentials,
    loopback_redirect_uri, parse_oauth_callback_query, parse_oauth_callback_request_target,
    save_oauth_credentials,
};
pub use permissions::{
    PermissionMode, PermissionOutcome, PermissionPolicy, PermissionPromptDecision,
    PermissionPrompter, PermissionRequest,
};
pub use prompt::{
    ContextFile, FRONTIER_MODEL_NAME, ProjectContext, PromptBuildError,
    SYSTEM_PROMPT_DYNAMIC_BOUNDARY, SystemPromptBuilder, load_system_prompt, prepend_bullets,
};
pub use remote::{
    DEFAULT_REMOTE_BASE_URL, DEFAULT_SESSION_TOKEN_PATH, DEFAULT_SYSTEM_CA_BUNDLE, NO_PROXY_HOSTS,
    RemoteSessionContext, UPSTREAM_PROXY_ENV_KEYS, UpstreamProxyBootstrap, UpstreamProxyState,
    inherited_upstream_proxy_env, no_proxy_list, read_token, upstream_proxy_ws_url,
};
pub use session::{ContentBlock, ConversationMessage, MessageRole, Session, SessionError};
pub use usage::{
    ModelPricing, TokenUsage, UsageCostEstimate, UsageTracker, format_usd, pricing_for_model,
};

#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
