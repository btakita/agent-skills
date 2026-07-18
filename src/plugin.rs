//! Plugin boundary for harness-specific skill integration.
//!
//! Downstream tools can implement [`SkillHarnessPlugin`] and register it with a
//! [`PluginRegistry`] instead of depending on skill-harness internals.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::manage::{HarnessTarget, SkillConfig};

/// Environment lookup used by plugin detection.
pub trait EnvironmentProvider: Send + Sync {
    fn var_os(&self, key: &str) -> Option<OsString>;
}

/// Environment provider backed by the current process.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessEnvironment;

impl EnvironmentProvider for ProcessEnvironment {
    fn var_os(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }
}

/// Context passed to plugins while resolving the active integration.
pub struct PluginContext<'a> {
    environment: &'a dyn EnvironmentProvider,
}

impl<'a> PluginContext<'a> {
    pub fn new(environment: &'a dyn EnvironmentProvider) -> Self {
        Self { environment }
    }

    pub fn process() -> PluginContext<'static> {
        static PROCESS_ENVIRONMENT: ProcessEnvironment = ProcessEnvironment;
        PluginContext::new(&PROCESS_ENVIRONMENT)
    }

    pub fn env_var(&self, key: &str) -> Option<OsString> {
        self.environment.var_os(key)
    }

    pub fn has_env_var(&self, key: &str) -> bool {
        self.env_var(key).is_some()
    }
}

/// A tool-specific skill integration.
pub trait SkillHarnessPlugin: Send + Sync {
    /// Stable integration id, such as `codex` or `agent-doc`.
    fn id(&self) -> &'static str;

    /// Higher-priority matching plugins win; equal priorities keep registration order.
    fn priority(&self) -> i32 {
        0
    }

    /// Whether this plugin should handle the current context.
    fn detect(&self, context: &PluginContext<'_>) -> bool;

    /// Relative install path for a skill in this integration.
    fn skill_rel_path(&self, skill_name: &str) -> PathBuf;
}

/// Built-in plugin for one explicit harness target.
#[derive(Debug, Clone, Copy)]
pub struct HarnessPlugin {
    id: &'static str,
    target: HarnessTarget,
    env_vars: &'static [&'static str],
    fallback: bool,
}

impl HarnessPlugin {
    pub const fn new(
        id: &'static str,
        target: HarnessTarget,
        env_vars: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            target,
            env_vars,
            fallback: false,
        }
    }

    pub const fn fallback(id: &'static str, target: HarnessTarget) -> Self {
        Self {
            id,
            target,
            env_vars: &[],
            fallback: true,
        }
    }
}

impl SkillHarnessPlugin for HarnessPlugin {
    fn id(&self) -> &'static str {
        self.id
    }

    fn priority(&self) -> i32 {
        if self.fallback { -100 } else { 0 }
    }

    fn detect(&self, context: &PluginContext<'_>) -> bool {
        self.fallback || self.env_vars.iter().any(|key| context.has_env_var(key))
    }

    fn skill_rel_path(&self, skill_name: &str) -> PathBuf {
        self.target.skill_rel_path(skill_name)
    }
}

/// Ordered registry of available skill integrations.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn SkillHarnessPlugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::with_default_plugins()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn with_default_plugins() -> Self {
        let mut registry = Self::new();
        registry.register(HarnessPlugin::new(
            "claude",
            HarnessTarget::ClaudeCode,
            &["CLAUDE_CODE", "CLAUDE_CODE_ENTRYPOINT"],
        ));
        registry.register(HarnessPlugin::new(
            "opencode",
            HarnessTarget::OpenCode,
            &["OPENCODE"],
        ));
        registry.register(HarnessPlugin::new(
            "codex",
            HarnessTarget::Codex,
            &["CODEX_CLI", "CODEX"],
        ));
        registry.register(HarnessPlugin::new(
            "cursor",
            HarnessTarget::Cursor,
            &["CURSOR_SESSION_ID", "CURSOR"],
        ));
        registry.register(HarnessPlugin::fallback("generic", HarnessTarget::Generic));
        registry
    }

    pub fn register(&mut self, plugin: impl SkillHarnessPlugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    pub fn detect<'a>(&'a self, context: &PluginContext<'_>) -> Option<&'a dyn SkillHarnessPlugin> {
        let mut selected: Option<&dyn SkillHarnessPlugin> = None;
        for plugin in &self.plugins {
            let plugin = plugin.as_ref();
            if !plugin.detect(context) {
                continue;
            }
            if selected.is_none_or(|current| plugin.priority() > current.priority()) {
                selected = Some(plugin);
            }
        }
        selected
    }

    pub fn skill_config(
        &self,
        name: impl Into<String>,
        content: impl Into<String>,
        version: impl Into<String>,
        context: &PluginContext<'_>,
    ) -> Option<SkillConfig> {
        let name = name.into();
        let plugin = self.detect(context)?;
        let rel_path = plugin.skill_rel_path(&name);
        Some(SkillConfig::new(name, content, version, move |_| {
            rel_path.clone()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct TestEnvironment {
        vars: BTreeMap<String, OsString>,
    }

    impl TestEnvironment {
        fn with(mut self, key: &str, value: &str) -> Self {
            self.vars.insert(key.to_string(), OsString::from(value));
            self
        }
    }

    impl EnvironmentProvider for TestEnvironment {
        fn var_os(&self, key: &str) -> Option<OsString> {
            self.vars.get(key).cloned()
        }
    }

    struct AgentDocPlugin;

    impl SkillHarnessPlugin for AgentDocPlugin {
        fn id(&self) -> &'static str {
            "agent-doc"
        }

        fn priority(&self) -> i32 {
            100
        }

        fn detect(&self, context: &PluginContext<'_>) -> bool {
            context.has_env_var("AGENT_DOC_SESSION")
        }

        fn skill_rel_path(&self, skill_name: &str) -> PathBuf {
            PathBuf::from(format!(".agent-doc/plugins/{skill_name}/SKILL.md"))
        }
    }

    #[test]
    fn defaults_detect_codex_with_documented_skill_path() {
        let env = TestEnvironment::default().with("CODEX_CLI", "1");
        let context = PluginContext::new(&env);
        let registry = PluginRegistry::with_default_plugins();

        let config = registry
            .skill_config("compose-skills", "content", "1.0.0", &context)
            .unwrap();

        assert_eq!(
            config.skill_path(None),
            PathBuf::from(".codex/skills/compose-skills/SKILL.md")
        );
    }

    #[test]
    fn defaults_fall_back_to_generic_skill_path() {
        let env = TestEnvironment::default();
        let context = PluginContext::new(&env);
        let registry = PluginRegistry::with_default_plugins();

        let plugin = registry.detect(&context).unwrap();

        assert_eq!(plugin.id(), "generic");
        assert_eq!(
            plugin.skill_rel_path("deploy"),
            PathBuf::from(".agent/skills/deploy/SKILL.md")
        );
    }

    #[test]
    fn custom_plugin_can_override_default_detection() {
        let env = TestEnvironment::default()
            .with("CODEX_CLI", "1")
            .with("AGENT_DOC_SESSION", "1");
        let context = PluginContext::new(&env);
        let mut registry = PluginRegistry::with_default_plugins();
        registry.register(AgentDocPlugin);

        let config = registry
            .skill_config("agent-doc", "content", "1.0.0", &context)
            .unwrap();

        assert_eq!(
            config.skill_path(None),
            PathBuf::from(".agent-doc/plugins/agent-doc/SKILL.md")
        );
    }
}
