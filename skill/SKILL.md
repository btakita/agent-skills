# skill-harness

Manage AI agent skills — install, check, uninstall, and list skills across environments.

## Invocation

```
skill-harness install <name> --file <SKILL.md>
skill-harness check <name> --file <SKILL.md>
skill-harness uninstall <name>
skill-harness list
```

## When to use

- When asked to install a skill into a project
- When checking if a skill is up to date
- When removing a skill from the project
- When listing all installed skills

## Commands

### install

Install a skill to the appropriate environment-specific path.

```bash
skill-harness install email --file .agent/skills/email/SKILL.md
```

The target path depends on the detected environment:
- Claude Code: `.claude/skills/<name>/SKILL.md`
- OpenCode: `.opencode/skills/<name>/SKILL.md`
- Cursor: `.cursor/rules/<name>.md`
- Generic: `.agent/skills/<name>/SKILL.md`

### check

Verify if an installed skill matches the source content.

```bash
skill-harness check email --file .agent/skills/email/SKILL.md
```

Returns exit code 0 if up to date, 1 if outdated or not installed.

### uninstall

Remove a skill from the current environment.

```bash
skill-harness uninstall email
```

Removes the skill file and cleans up empty parent directories.

### list

Show all installed skills across known locations.

```bash
skill-harness list
```

Scans `.agent/skills/` and environment-specific skill directories.

## Runbooks

- `install skill` — [runbooks/install-skill.md](runbooks/install-skill.md)
