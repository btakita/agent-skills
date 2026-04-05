//! Skill management — install/check/uninstall SKILL.md files for agent environments.
//!
//! CLI tools bundle a SKILL.md via `include_str!` and use this module to install
//! it to the appropriate location for the active agent environment.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Configuration for a skill to be managed.
pub struct SkillConfig {
    /// The tool name (e.g., "agent-doc", "webmaster").
    pub name: String,
    /// The bundled SKILL.md content (typically from `include_str!`).
    pub content: String,
    /// The tool version (typically from `env!("CARGO_PKG_VERSION")`).
    pub version: String,
    /// The relative path resolver for the target environment.
    pub path_resolver: Box<dyn Fn(&str) -> PathBuf + Send + Sync>,
}

impl SkillConfig {
    /// Create a new skill config with a custom path resolver.
    pub fn new(
        name: impl Into<String>,
        content: impl Into<String>,
        version: impl Into<String>,
        path_resolver: impl Fn(&str) -> PathBuf + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            version: version.into(),
            path_resolver: Box::new(path_resolver),
        }
    }

    /// Create a skill config that installs to `.agent/skills/<name>/SKILL.md`.
    pub fn generic(
        name: impl Into<String>,
        content: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self::new(name, content, version, |name| {
            PathBuf::from(format!(".agent/skills/{name}/SKILL.md"))
        })
    }

    /// Resolve the skill file path under the given root (or CWD if None).
    pub fn skill_path(&self, root: Option<&Path>) -> PathBuf {
        let rel = (self.path_resolver)(&self.name);
        match root {
            Some(r) => r.join(rel),
            None => rel,
        }
    }

    /// Install the bundled SKILL.md to the project.
    pub fn install(&self, root: Option<&Path>) -> Result<()> {
        let path = self.skill_path(root);

        if path.exists() {
            let existing = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            if existing == self.content {
                eprintln!("Skill already up to date (v{}).", self.version);
                return Ok(());
            }
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        std::fs::write(&path, &self.content)
            .with_context(|| format!("failed to write {}", path.display()))?;
        eprintln!("Installed skill v{} → {}", self.version, path.display());

        Ok(())
    }

    /// Check if the installed skill matches the bundled version.
    pub fn check(&self, root: Option<&Path>) -> Result<bool> {
        let path = self.skill_path(root);

        if !path.exists() {
            eprintln!("Not installed. Run `{} skill install` to install.", self.name);
            return Ok(false);
        }

        let existing = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        if existing == self.content {
            eprintln!("Up to date (v{}).", self.version);
            Ok(true)
        } else {
            eprintln!(
                "Outdated. Run `{} skill install` to update to v{}.",
                self.name, self.version
            );
            Ok(false)
        }
    }

    /// Uninstall the skill file and its parent directory (if empty).
    pub fn uninstall(&self, root: Option<&Path>) -> Result<()> {
        let path = self.skill_path(root);

        if !path.exists() {
            eprintln!("Skill not installed.");
            return Ok(());
        }

        std::fs::remove_file(&path)
            .with_context(|| format!("failed to remove {}", path.display()))?;

        if let Some(parent) = path.parent()
            && parent.read_dir().is_ok_and(|mut d| d.next().is_none())
        {
            let _ = std::fs::remove_dir(parent);
        }

        eprintln!("Uninstalled skill from {}", path.display());
        Ok(())
    }
}

/// Create a SkillConfig that uses agent-kit Environment for path resolution.
#[cfg(feature = "detect")]
pub fn skill_for_environment(
    name: impl Into<String>,
    content: impl Into<String>,
    version: impl Into<String>,
) -> SkillConfig {
    let env = agent_kit::detect::Environment::detect();
    let name_str = name.into();
    let name_clone = name_str.clone();
    SkillConfig {
        name: name_str,
        content: content.into(),
        version: version.into(),
        path_resolver: Box::new(move |_| env.skill_rel_path(&name_clone)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SkillConfig {
        SkillConfig::new(
            "test-tool",
            "# Test Skill\n\nSome content.\n",
            "1.0.0",
            |name| PathBuf::from(format!(".claude/skills/{name}/SKILL.md")),
        )
    }

    #[test]
    fn skill_path_with_root() {
        let config = test_config();
        let path = config.skill_path(Some(Path::new("/project")));
        assert_eq!(
            path,
            PathBuf::from("/project/.claude/skills/test-tool/SKILL.md")
        );
    }

    #[test]
    fn skill_path_without_root() {
        let config = test_config();
        let path = config.skill_path(None);
        assert_eq!(
            path,
            PathBuf::from(".claude/skills/test-tool/SKILL.md")
        );
    }

    #[test]
    fn generic_skill_path() {
        let config = SkillConfig::generic("my-tool", "content", "1.0.0");
        let path = config.skill_path(None);
        assert_eq!(
            path,
            PathBuf::from(".agent/skills/my-tool/SKILL.md")
        );
    }

    #[test]
    fn install_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config();
        config.install(Some(dir.path())).unwrap();

        let path = dir.path().join(".claude/skills/test-tool/SKILL.md");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, config.content);
    }

    #[test]
    fn install_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config();
        config.install(Some(dir.path())).unwrap();
        config.install(Some(dir.path())).unwrap();

        let path = dir.path().join(".claude/skills/test-tool/SKILL.md");
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, config.content);
    }

    #[test]
    fn check_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config();
        assert!(!config.check(Some(dir.path())).unwrap());
    }

    #[test]
    fn check_up_to_date() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config();
        config.install(Some(dir.path())).unwrap();
        assert!(config.check(Some(dir.path())).unwrap());
    }

    #[test]
    fn uninstall_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config();
        config.install(Some(dir.path())).unwrap();
        config.uninstall(Some(dir.path())).unwrap();

        let path = dir.path().join(".claude/skills/test-tool/SKILL.md");
        assert!(!path.exists());
    }

    #[test]
    fn uninstall_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config();
        config.uninstall(Some(dir.path())).unwrap();
    }
}
