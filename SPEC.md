# agent-skills spec

Format specification for contextually-activated instruction bundles. Skills package rules, runbooks, and examples into self-contained directories with activation metadata.

## Skill Directory Structure

```
.agent/skills/<name>/
├── SKILL.md           # required: instruction content + activation metadata
├── runbooks/          # optional: on-demand procedures
│   ├── deploy.md
│   └── migrate.md
└── examples/          # optional: reference material
    ├── config.yaml
    └── template.ts
```

The skill name is the directory name. It should be a lowercase slug (e.g., `testing`, `deploy`, `api-client`).

## SKILL.md Format

Each SKILL.md is a markdown file with optional YAML frontmatter:

```markdown
---
description: "One-line description for contextual activation"
globs: ["**/*.test.ts", "**/*.spec.ts"]
alwaysApply: false
---

# Skill Name

Instruction content here. This is the body that gets loaded when the skill activates.

## Rules

- Convention or constraint specific to this capability.

## Runbooks

- **Deploy**: Follow `runbooks/deploy.md` when deploying.
- **Migrate**: Follow `runbooks/migrate.md` for database migrations.
```

### Frontmatter Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `description` | string | yes | -- | One-line summary used for agent-requested activation |
| `globs` | string[] | no | `[]` | File patterns that trigger activation |
| `alwaysApply` | boolean | no | `false` | Whether to load on every interaction |

### Body Content

The markdown body contains the skill's instruction content. It can include:

- **Rules** -- declarative policy (conventions, constraints, architecture decisions)
- **Runbook references** -- pointers to procedures in the `runbooks/` directory
- **Inline guidance** -- any other instruction content relevant to this capability

## Activation Modes

Skills activate based on their frontmatter configuration:

### Always

```yaml
---
description: "Project-wide coding conventions"
alwaysApply: true
---
```

Loaded on every interaction. Use sparingly -- this adds to base context cost.

### File-pattern

```yaml
---
description: "Testing conventions and utilities"
globs: ["**/*.test.ts", "**/*.spec.ts", "tests/**"]
---
```

Loaded when the agent is working with files matching the glob patterns. The matching semantics follow gitignore-style globs.

### Agent-requested

```yaml
---
description: "Database migration procedures and conventions"
---
```

The agent reads the `description` and decides whether to load the skill based on the current task. No `globs` or `alwaysApply` -- activation is the agent's judgment call.

### Manual

No frontmatter needed. The user explicitly tells the agent to use the skill:

> "Use the deploy skill for this."

Or references it directly:

> "Follow `.agent/skills/deploy/SKILL.md`."

## Runbooks Directory

Skills can include runbooks in a `runbooks/` subdirectory. These follow the [agent-runbooks](https://github.com/btakita/agent-runbooks) convention:

```
.agent/skills/deploy/
├── SKILL.md
└── runbooks/
    ├── production.md
    └── staging.md
```

Reference runbooks from SKILL.md:

```markdown
## Runbooks

- **Production deploy**: Follow `runbooks/production.md` for production releases.
- **Staging deploy**: Follow `runbooks/staging.md` for staging environments.
```

The trigger phrase (e.g., "Production deploy") tells the agent when to load the referenced runbook.

## Examples Directory

Skills can include reference material in an `examples/` subdirectory:

```
.agent/skills/api-client/
├── SKILL.md
└── examples/
    ├── basic-usage.ts
    └── error-handling.ts
```

Examples are not loaded by default -- the agent reads them when it needs concrete reference material.

## Install Semantics

To install a skill into a project:

1. Copy or symlink the skill directory into `.agent/skills/<name>/`
2. The skill is immediately available -- no registration step required
3. For tool-specific formats, generate the native representation:

**Claude Code:**
```bash
ln -s ../../.agent/skills/deploy .claude/skills/deploy
```

**Cursor (generate .mdc):**
```
---
description: "Deploy procedures and conventions"
globs:
alwaysApply: false
---
# Deploy
...
```

**Copilot (merge into scoped instructions):**
```markdown
## Deploy (from .agent/skills/deploy)
...
```

## Validation Rules

When auditing skills:

1. **SKILL.md required**: Every skill directory must contain a `SKILL.md` file
2. **Description required**: Frontmatter must include a `description` field
3. **Runbooks exist**: Any runbook referenced in SKILL.md must exist in the `runbooks/` directory
4. **No machine-local paths**: Same context invariant as other instruction files -- no `~/`, `/home/user/`, or absolute paths that won't resolve on other machines
5. **Valid frontmatter**: If YAML frontmatter is present, it must parse without errors
6. **Unique names**: No duplicate skill directory names within a project

## Relationship to Other Specs

| Spec | Role in Skills |
|------|---------------|
| [agent-rules](https://github.com/btakita/agent-rules) | Rule content appears inline in SKILL.md body |
| [agent-runbooks](https://github.com/btakita/agent-runbooks) | Procedures live in the skill's `runbooks/` directory |
| [agent-memories](https://github.com/btakita/agent-memories) | Skills may generate memories during use; memories reference the skill scope |
