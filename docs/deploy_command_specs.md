### `deploy` Command Functional Specifications

The `meshstack deploy` command is responsible for building and deploying one or more services to the current Kubernetes context.

#### 1. `--service <name>`

*   **Purpose**: Specifies a single service to deploy.
*   **Input**: A string representing the name of a service defined within the `services/` directory.
*   **Behavior**:
    *   If this option is provided, `meshstack` will only build and deploy the specified service.
    *   If this option is omitted, `meshstack` will attempt to deploy all services found in the `services/` directory.
*   **Output**:
    *   Messages indicating the build and deployment status of the service(s).
*   **Error Conditions**:
    *   `ServiceNotFound`: The specified service name does not correspond to an existing service directory.
    *   `BuildFailure`: The Docker image build process failed for a service.
    *   `DeploymentFailure`: The Kubernetes deployment (e.g., Helm install/upgrade) failed.

#### 2. `--env <dev|prod|staging>`

*   **Purpose**: Targets a specific environment profile for deployment.
*   **Input**: A string, one of `dev`, `prod`, or `staging`.
*   **Behavior**:
    *   This option will influence the values used during deployment (e.g., resource limits, replica counts, ingress rules).
    *   It will typically load environment-specific `values.yaml` files for Helm charts or apply kustomize overlays.
*   **Output**:
    *   A message indicating which environment profile is being applied.
*   **Error Conditions**:
    *   `InvalidEnvironment`: The provided environment is not recognized or supported.
    *   `ConfigurationError`: Unable to load or apply environment-specific configurations.

#### 3. `--build`

*   **Purpose**: Forces a rebuild of the Docker image(s) before deployment.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   When present, `meshstack` will execute the Docker build process for the service(s) even if an image with the current tag already exists locally.
    *   If omitted, `meshstack` might skip the build if a suitable image is found locally.
*   **Output**:
    *   Docker build output.
*   **Error Conditions**:
    *   `DockerNotFound`: Docker daemon is not running or Docker CLI is not in PATH.
    *   `BuildFailure`: The Docker build command failed.

#### 4. `--push`

*   **Purpose**: Pushes the built Docker image(s) to a configured container registry.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   When present, `meshstack` will execute `docker push` after a successful build.
    *   The target registry should be configurable (e.g., via `meshstack.yaml`).
*   **Output**:
    *   Docker push output.
*   **Error Conditions**:
    *   `RegistryAuthenticationFailure`: Unable to authenticate with the container registry.
    *   `PushFailure`: The Docker push command failed.

#### 5. `--context <kube-context>`

*   **Purpose**: Overrides the default Kubernetes context for deployment.
*   **Input**: A string representing the name of a valid Kubernetes context.
*   **Behavior**:
    *   Similar to the `install` command, this option directs the deployment to a specific Kubernetes cluster context.
*   **Output**:
    *   A message confirming the Kubernetes context being targeted.
*   **Error Conditions**:
    *   `ContextNotFound`: The specified Kubernetes context does not exist.
    *   `ContextInaccessible`: The specified Kubernetes context exists but is inaccessible.
