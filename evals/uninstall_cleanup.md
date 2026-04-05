# uninstall_cleanup

**Contract:** Uninstall

## Scenario

Install a skill, then uninstall it. Observe the filesystem state after uninstall.

## Expected Behavior

1. The skill file is removed from the target path.
2. Empty parent directories created during install are removed (up to the environment root).
3. Non-empty parent directories (containing other skills or user files) are preserved.
4. Sibling skills and unrelated files are untouched.

## Failure Modes

- Skill file remains on disk after uninstall.
- Parent directory with other skills is deleted.
- Unrelated files in the skill directory tree are removed.
- Non-empty directories are removed, causing collateral data loss.
