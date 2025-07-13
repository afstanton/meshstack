### `status` Command Functional Specifications

The `meshstack status` command provides an overview of `meshstack`-managed resources and their current versions or states.

#### 1. `--components`

*   **Purpose**: Shows a list of installed infrastructure components and their versions.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will query the Kubernetes cluster (via Helm or direct API calls) to identify installed components (e.g., Istio, Prometheus, Grafana).
    *   It will report the name, installed version, and potentially the chart version or namespace.
*   **Output**:
    *   A formatted table or list of installed components with their details.
*   **Error Conditions**:
    *   `ConnectionError`: Unable to connect to the Kubernetes cluster.
    *   `PermissionDenied`: Insufficient permissions to list installed components.

#### 2. `--services`

*   **Purpose**: Shows a list of running application services deployed by `meshstack`.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will query the Kubernetes cluster to list pods, deployments, or services associated with the application services defined in the local project.
    *   It will report the service name, status (e.g., running, pending, failed), and potentially the number of replicas.
*   **Output**:
    *   A formatted table or list of deployed services with their status.
*   **Error Conditions**:
    *   `ConnectionError`: Unable to connect to the Kubernetes cluster.
    *   `PermissionDenied`: Insufficient permissions to list services.

#### 3. `--lockfile`

*   **Purpose**: Compares the current deployed state with the `meshstack.lock` file (if it exists).
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will read the `meshstack.lock` file, which is intended to store the exact versions and configurations of deployed components.
    *   It will then compare this recorded state with the actual state in the Kubernetes cluster.
    *   It will highlight any discrepancies (e.g., version mismatches, components missing from the cluster but present in lockfile, or vice-versa).
*   **Output**:
    *   A report detailing any drift between the `meshstack.lock` file and the cluster's actual state.
*   **Error Conditions**:
    *   `FileNotFound`: `meshstack.lock` file does not exist.
    *   `InvalidFormat`: `meshstack.lock` file is corrupted or has an invalid format.

#### 4. `--context <kube-context>`

*   **Purpose**: Shows the status of resources within a specific Kubernetes context.
*   **Input**: A string representing the name of a valid Kubernetes context.
*   **Behavior**:
    *   This option allows the user to query the status of components and services in a Kubernetes context other than the currently active one.
    *   It will apply to all other `--components`, `--services`, and `--lockfile` flags when used together.
*   **Output**:
    *   Status information scoped to the specified Kubernetes context.
*   **Error Conditions**:
    *   `ContextNotFound`: The specified Kubernetes context does not exist.
    *   `ContextInaccessible`: The specified Kubernetes context exists but is inaccessible.
