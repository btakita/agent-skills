# environment_path_resolution

**Contract:** Install

## Scenario

Install the same skill targeting each supported environment: Claude Code, Cursor, Windsurf, Copilot, Gemini CLI, Codex, Aider.

## Expected Behavior

1. Each environment resolves to its documented native path:
   - Claude Code: `.claude/skills/<name>/SKILL.md`
   - Cursor: `.cursor/rules/<name>.mdc`
   - Windsurf: `.windsurf/rules/<name>.md`
   - Copilot: `.github/copilot-instructions.md` (merged)
   - Gemini CLI: `GEMINI.md` with `@file` reference
   - Codex: skill directory with `AGENTS.md`
   - Aider: `.aider.conf.yml` read list entry
2. No environment gets another environment's path.
3. Unknown environments produce a clear error, not a silent wrong path.

## Failure Modes

- Environment detection picks the wrong target (e.g., installs Cursor format into Claude Code path).
- Path uses hardcoded home directory instead of project-relative resolution.
- Unsupported environment silently falls back to a default without informing the user.
