use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skill-harness", about = "Lifecycle management for AI agent skills")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a skill to the project
    Install {
        /// Skill name
        name: String,
        /// Skill content file path
        #[arg(short, long)]
        file: PathBuf,
        /// Project root (default: CWD)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
    /// Check if a skill is installed and up to date
    Check {
        /// Skill name
        name: String,
        /// Skill content file path
        #[arg(short, long)]
        file: PathBuf,
        /// Project root (default: CWD)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
    /// Uninstall a skill from the project
    Uninstall {
        /// Skill name
        name: String,
        /// Project root (default: CWD)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
    /// List installed skills
    List {
        /// Project root (default: CWD)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { name, file, root } => {
            let content = std::fs::read_to_string(&file)?;
            let config = make_config(&name, &content);
            config.install(root.as_deref())?;
        }
        Commands::Check { name, file, root } => {
            let content = std::fs::read_to_string(&file)?;
            let config = make_config(&name, &content);
            let ok = config.check(root.as_deref())?;
            if !ok {
                std::process::exit(1);
            }
        }
        Commands::Uninstall { name, root } => {
            let config = make_config(&name, "");
            config.uninstall(root.as_deref())?;
        }
        Commands::List { root } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            list_skills(&root);
        }
    }

    Ok(())
}

fn make_config(name: &str, content: &str) -> skill_harness::manage::SkillConfig {
    #[cfg(feature = "detect")]
    {
        skill_harness::manage::skill_for_environment(name, content, env!("CARGO_PKG_VERSION"))
    }
    #[cfg(not(feature = "detect"))]
    {
        skill_harness::manage::SkillConfig::generic(name, content, env!("CARGO_PKG_VERSION"))
    }
}

fn list_skills(root: &std::path::Path) {
    let patterns = [
        ".agent/skills/*/SKILL.md",
        ".claude/skills/*/SKILL.md",
    ];

    let mut found = false;
    for pattern in &patterns {
        let full = root.join(pattern).to_string_lossy().to_string();
        if let Ok(entries) = glob::glob(&full) {
            for entry in entries.flatten() {
                let name = entry
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let rel = entry.strip_prefix(root).unwrap_or(&entry);
                println!("  {} → {}", name, rel.display());
                found = true;
            }
        }
    }

    if !found {
        println!("No skills installed.");
    }
}
