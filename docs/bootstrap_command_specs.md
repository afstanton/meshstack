# `bootstrap` Command Specification

## Purpose

Set up a full local Kubernetes cluster and install all necessary infrastructure components for development.

## Options

| Flag | Description |
|------|-------------|
| `--kind` | Use Kind for local cluster provisioning (default) |
| `--k3d` | Use k3d for local cluster provisioning |
| `--skip-install` | Skip installation of infrastructure components |

## Output

- Provisions a local Kubernetes cluster.
- Installs default infrastructure components (e.g., Istio, Prometheus, Grafana).
- Configures local kubeconfig to connect to the new cluster.