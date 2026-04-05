# install_idempotent

**Contract:** Install

## Scenario

Install a skill into a target environment. Then install the same skill again without changes.

## Expected Behavior

1. First install succeeds and reports the installed path.
2. Second install detects existing content, compares it, and reports "up-to-date" (or equivalent).
3. The skill file on disk is byte-identical after both installs.
4. No duplicate entries, no errors, no unnecessary writes.

## Failure Modes

- Second install overwrites the file (unnecessary I/O, potential timestamp churn).
- Second install creates a duplicate (e.g., `SKILL (1).md`).
- Second install errors out instead of recognizing the existing installation.
