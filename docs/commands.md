# Meshstack CLI Commands

This document outlines the core CLI commands available in Meshstack v0.1. Each command is structured to manage the lifecycle of a distributed mesh app across dev and production environments.

---

## 🧱 1. `init`

**Purpose**: Create a new mesh app project with config and template structure.

**Options**:

| Flag | Description |
|------|-------------|
| `--name <project>` | Name of the project (default: current directory) |
| `--language <lang>` | App language (`rust`, `go`, `python`, `node`) |
| `--mesh <istio|linkerd>` | Choose service mesh (default: `istio`) |
| `--ci <github|argo>` | CI/CD preference |
| `--config <path>` | Use preexisting meshstack.yaml config |

**Output**:
- Creates `meshstack.yaml`
- Initializes scaffold directories: `services/`, `provision/`, etc.

---

## ⚙️ 2. `install`

**Purpose**: Install infrastructure components into current Kubernetes cluster.

**Options**:

| Flag | Description |
|------|-------------|
| `--component <name>` | Specific component (e.g. `istio`, `prometheus`, `vault`) |
| `--profile <dev|prod|custom>` | Install resource-tuned versions |
| `--dry-run` | Print manifests instead of applying |
| `--context <kube-context>` | Target a specific cluster context |

**Output**:
- Applies Helm charts or kustomize overlays
- Tracks installed components (e.g., via `meshstack.lock`)

---

## 🔁 3. `update`

**Purpose**: Update installed components or generated files.

**Options**:

| Flag | Description |
|------|-------------|
| `--check` | Show available updates |
| `--apply` | Apply all updates automatically |
| `--component <name>` | Target a specific component |
| `--template` | Update project templates (Dockerfile, Helm, etc.) |
| `--infra` | Update infra charts (e.g. mesh version bump) |

**Output**:
- In-place update of Helm versions or CLI templates
- Warns about breaking changes, offers diff preview

---

## 🚀 4. `deploy`

**Purpose**: Deploy one or more services to current Kubernetes context.

**Options**:

| Flag | Description |
|------|-------------|
| `--service <name>` | Deploy a single service (or all if omitted) |
| `--env <dev|prod|staging>` | Target a specific env profile |
| `--build` | Rebuild Docker image before deploy |
| `--push` | Push container to registry (configurable) |
| `--context` | Kube context override |

**Output**:
- Builds, tags, and deploys containers via Helm or kubectl
- Optionally syncs to ArgoCD repo if GitOps is active

---

## 💥 5. `destroy`

**Purpose**: Tear down one or more components or the full stack.

**Options**:

| Flag | Description |
|------|-------------|
| `--component <name>` | Destroy just a specific part |
| `--confirm` | Bypass confirmation prompt |
| `--all` | Nuke from orbit (dev/test use only) |

**Output**:
- Removes Helm releases, CRDs, or k3d clusters
- Optionally removes meshstack-generated files

---

## 🧪 6. `validate`

**Purpose**: Validate config, manifests, and cluster readiness.

**Options**:

| Flag | Description |
|------|-------------|
| `--config` | Validate `meshstack.yaml` against schema |
| `--cluster` | Check connectivity to kube context |
| `--ci` | Validate GitHub Actions or ArgoCD manifests |
| `--full` | Run all validators |

**Output**:
- Prints results + optional warnings about drift or version mismatches

---

## 📜 7. `status`

**Purpose**: Show meshstack-managed resources and current versions.

**Options**:

| Flag | Description |
|------|-------------|
| `--components` | Show installed infrastructure and versions |
| `--services` | Show running app services |
| `--lockfile` | Compare current state with `meshstack.lock` |
| `--context` | Show per-kube-context state |

---

## 🛠️ Future Commands (planned)

- `bootstrap` – full local cluster and infra setup (dev-only) ([specs](bootstrap_command_specs.md))
- `generate` – re-generate scaffolds based on config ([specs](generate_command_specs.md))
- `plan` – dry-run of what would be installed/deployed/changed ([specs](plan_command_specs.md))
