mod error;
mod hooks;
mod plugins;
mod run;
mod skills;
mod tools;
mod types;

pub use error::{AgentError, Result};
pub use hooks::{load_hooks_from_dir, run_hooks, Hook, HookContext, HookEvent};
pub use plugins::{load_plugins_from_dir, PluginManifest, PluginTool};
pub use run::{run_agent, AgentRunContext};
pub use skills::load_skills_from_dir;
pub use types::{default_profiles, AgentProfile, AgentResponse, Skill, ToolCallRecord};
