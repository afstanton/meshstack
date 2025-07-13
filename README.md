


# Meshstack

**Meshstack** is a CLI tool scaffold designed to help developers spin up resilient, scalable, cloud-agnostic microservice architectures using modern open infrastructure. It creates a fully configurable mesh app project, complete with service templates, deployment scripts, and a self-healing runtime configuration.

## Features

- Generates a production-ready project skeleton
- Supports Kubernetes (k8s), Nomad, and container-based deployments
- Includes setup scripts for local development
- Infrastructure-as-code templates (Terraform, Helm, etc.)
- Modular service definitions and shared configuration
- CLI interactive or flag-driven project generation

## Getting Started

1. Install the CLI:
   ```bash
   brew tap afstanton/meshstack
   brew install meshstack
   ```

2. Generate a new project:
   ```bash
   meshstack new my-app
   ```

## License

MIT License. See `LICENSE` file for details.

## Releasing a New Version

To release a new version of `meshstack` to both crates.io and Homebrew, follow these steps:

1. **Update the version number** in `Cargo.toml`.
2. **Create a new git tag** for the version (e.g., `v0.1.1`).
3. **Push the tag to GitHub**. This will trigger the release workflow.

```bash
# 1. Update version in Cargo.toml
# (Manually edit the file)

# 2. Create a new git tag
git tag v0.1.1

# 3. Push the tag to GitHub
git push origin v0.1.1
```

The GitHub Actions workflow will then automatically build the release, publish it to crates.io, and update the Homebrew tap.
