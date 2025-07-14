# `plan` Command Specification

## Purpose

Perform a dry-run of what would be installed, deployed, or changed by a given command without actually applying any changes.

## Options

| Flag | Description |
|------|-------------|
| `--command <cmd>` | The command to dry-run (e.g., `install`, `deploy`, `destroy`) |
| `--verbose` | Show detailed output of planned changes |

## Output

- Displays a summary of actions that would be taken.
- Shows a diff of proposed changes to infrastructure or services.
- Provides warnings for potential issues without side effects.