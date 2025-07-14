# `generate` Command Specification

## Purpose

Re-generate scaffolds and configuration files based on the `meshstack.yaml` configuration.

## Options

| Flag | Description |
|------|-------------|
| `--service <name>` | Generate scaffold for a specific service |
| `--all` | Re-generate all project scaffolds and configurations |
| `--force` | Overwrite existing files without prompt |

## Output

- Updates or creates service directories and Dockerfiles.
- Re-generates Kubernetes manifests or Helm charts based on `meshstack.yaml`.
- Provides a summary of generated or updated files.