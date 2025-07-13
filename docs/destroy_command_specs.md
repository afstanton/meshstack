### `destroy` Command Functional Specifications

The `meshstack destroy` command is responsible for tearing down deployed components or the entire mesh app stack from a Kubernetes cluster.

#### 1. `--component <name>`

*   **Purpose**: Specifies a single component to destroy.
*   **Input**: A string representing the name of a deployed infrastructure component (e.g., `istio`, `prometheus`) or a service.
*   **Behavior**:
    *   If provided, `meshstack` will only attempt to uninstall or delete the specified component or service.
    *   This typically involves Helm uninstallation or `kubectl delete` operations.
*   **Output**:
    *   Confirmation messages for the successful destruction of the specified component.
*   **Error Conditions**:
    *   `ComponentNotFound`: The specified component or service is not recognized or found as deployed.
    *   `DeletionFailure`: The uninstallation or deletion process failed for the component.

#### 2. `--confirm`

*   **Purpose**: Bypasses the interactive confirmation prompt before destruction.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   When present, `meshstack` will proceed with the destruction operation without asking for user confirmation.
    *   **Caution**: This flag should be used with extreme care, especially in production environments, as it can lead to irreversible data loss.
*   **Output**:
    *   No confirmation prompt will be displayed.
*   **Error Conditions**:
    *   None directly related to the flag itself, but errors from the destruction process will still be reported.

#### 3. `--all`

*   **Purpose**: Destroys all `meshstack`-managed resources and optionally the local project files.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   When present, `meshstack` will initiate a comprehensive teardown, removing all deployed infrastructure components and services.
    *   This option is primarily intended for development and testing environments.
    *   It should prompt for confirmation unless `--confirm` is also used.
    *   Optionally, it might remove `meshstack.yaml`, `services/`, `provision/`, and other generated files from the local project directory.
*   **Output**:
    *   Messages detailing the resources being destroyed.
*   **Error Conditions**:
    *   `DeletionFailure`: Failure to remove one or more resources.
    *   `PermissionDenied`: Insufficient permissions to delete resources.
