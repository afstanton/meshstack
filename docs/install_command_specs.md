### `install` Command Functional Specifications

The `meshstack install` command is responsible for deploying infrastructure components into a Kubernetes cluster using Helm.

#### 1. `--component <name>`

*   **Purpose**: Specifies a single infrastructure component to install.
*   **Input**: A string representing the name of a predefined component (e.g., `istio`, `prometheus`, `vault`).
*   **Behavior**:
    *   If this option is provided, `meshstack` will attempt to install *only* the specified component.
    *   If this option is omitted, `meshstack` will install a default set of core infrastructure components: `istio`, `prometheus`, `grafana`, `cert-manager`, and `nginx-ingress`.
*   **Output**:
    *   On success: A message confirming the installation of the specified component(s).
    *   On failure: An error message indicating if the component name is unrecognized or if the installation failed.
*   **Error Conditions**:
    *   `InvalidComponent`: The provided `<name>` does not correspond to a known or supported component.

#### 2. `--profile <dev|prod|custom>`

*   **Purpose**: Selects a predefined configuration profile for the installation, influencing resource allocation, replica counts, and other environment-specific settings.
*   **Input**: A string, one of `dev`, `prod`, or `custom`.
*   **Behavior**:
    *   `dev`: Applies a development-optimized profile (e.g., minimal resource requests, single replicas, simplified configurations).
    *   `prod`: Applies a production-optimized profile (e.g., higher resource requests, multiple replicas, high-availability settings).
    *   `custom`: (Future) This profile will imply loading configuration from a local file or a more advanced mechanism. For initial implementation, it can be treated as a placeholder or an error.
    *   This profile will translate into specific Helm `values.yaml` overrides or selection of different chart versions.
*   **Output**:
    *   A message indicating which profile is being applied.
*   **Error Conditions**:
    *   `InvalidProfile`: The provided profile is not `dev`, `prod`, or `custom`.

#### 3. `--dry-run`

*   **Purpose**: Simulates the installation process without making any actual changes to the Kubernetes cluster.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   When present, `meshstack` will execute the underlying Helm command with the `--dry-run` flag.
    *   Instead of applying resources, the command will print the rendered Kubernetes manifests to standard output.
    *   No persistent changes will be made to the cluster or local state.
*   **Output**:
    *   The full YAML output of the rendered Kubernetes manifests.
    *   Any warnings or errors that would occur during a real installation (e.g., invalid chart, syntax errors in values).
*   **Error Conditions**:
    *   Errors related to chart rendering or validation should still be reported.

#### 4. `--context <kube-context>`

*   **Purpose**: Specifies the Kubernetes context to target for the installation.
*   **Input**: A string representing the name of a valid Kubernetes context configured in the user's `kubeconfig` file.
*   **Behavior**:
    *   If this option is provided, `meshstack` will instruct Helm to perform the installation against the specified Kubernetes context.
    *   If this option is omitted, `meshstack` will use the currently active Kubernetes context as determined by `kubectl`.
*   **Output**:
    *   A message confirming the Kubernetes context being targeted.
*   **Error Conditions**:
    *   `ContextNotFound`: The specified Kubernetes context does not exist in the `kubeconfig`.
    *   `ContextInaccessible`: The specified Kubernetes context exists but is currently inaccessible (e.g., cluster is down, authentication failure).
