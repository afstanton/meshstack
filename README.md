


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
