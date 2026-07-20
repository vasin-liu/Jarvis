mod chat_resolver;
mod embed_resolver;
mod error;
mod hooks;
mod orchestrate;
mod plugins;
mod router;
mod run;
mod skills;
mod tool_parse;
mod tools;
mod types;

pub use chat_resolver::ChatResolver;
pub use embed_resolver::EmbedResolver;
pub use error::{AgentError, Result};
pub use hooks::{load_hooks_from_dir, run_hooks, Hook, HookContext, HookEvent};
pub use orchestrate::{resolve_pipeline_agent_ids, run_orchestrated};
pub use plugins::{
    load_plugins_from_dir, plugin_has_permissions, PluginManifest, PluginTool,
    PERMISSION_SHELL_EXEC,
};
pub use router::run_routed;
pub use run::{run_agent, AgentRunContext};
pub use skills::load_skills_from_dir;
pub use types::{
    default_profiles, AgentProfile, AgentResponse, OrchestrationStep, Skill, ToolCallRecord,
};
