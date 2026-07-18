//! Agent Skills — management for contextually-activated instruction bundles.
//!
//! Provides skill install/check/uninstall for AI agent environments.
//! Auto-detection is provided through the plugin registry.

pub mod compose;
pub mod manage;
pub mod okf;
pub mod plugin;

pub use manage::SkillConfig;
pub use plugin::{
    EnvironmentProvider, HarnessPlugin, PluginContext, PluginRegistry, ProcessEnvironment,
    SkillHarnessPlugin,
};
