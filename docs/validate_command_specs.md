### `validate` Command Functional Specifications

The `meshstack validate` command is responsible for checking the correctness and readiness of configurations, manifests, and the Kubernetes cluster.

#### 1. `--config`

*   **Purpose**: Validates the `meshstack.yaml` configuration file against a defined schema.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will parse `meshstack.yaml` and check its structure, data types, and values against an internal or external schema definition.
    *   It will ensure that all required fields are present and that values conform to expected formats (e.g., valid mesh types).
*   **Output**:
    *   "`meshstack.yaml` is valid." on success.
    *   Detailed error messages indicating schema violations, missing fields, or invalid values on failure.
*   **Error Conditions**:
    *   `InvalidSchema`: The `meshstack.yaml` file does not conform to the expected schema.
    *   `FileNotFound`: `meshstack.yaml` is not found in the current directory.

#### 2. `--cluster`

*   **Purpose**: Checks connectivity to the configured Kubernetes cluster and its readiness.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will attempt to connect to the Kubernetes API server using the current or specified context.
    *   It will perform basic checks like verifying API server reachability, authentication, and authorization.
    *   Optionally, it might check for the presence of core components (e.g., `kube-proxy`, `coredns`).
*   **Output**:
    *   "Connected to Kubernetes cluster successfully." on success.
    *   Error messages indicating connection issues, authentication failures, or cluster unreadiness.
*   **Error Conditions**:
    *   `ConnectionError`: Unable to connect to the Kubernetes API server.
    *   `AuthenticationError`: Failed to authenticate with the cluster.
    *   `AuthorizationError`: Insufficient permissions to perform checks.

#### 3. `--ci`

*   **Purpose**: Validates CI/CD manifests (e.g., GitHub Actions workflows, ArgoCD Application manifests).
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will parse and validate the syntax and structure of CI/CD related files.
    *   For GitHub Actions, it might check for valid YAML syntax and basic workflow structure.
    *   For ArgoCD, it might validate Application manifests against Kubernetes API conventions.
*   **Output**:
    *   "CI/CD manifests are valid." on success.
    *   Detailed error messages for syntax errors or structural issues in CI/CD files.
*   **Error Conditions**:
    *   `InvalidManifest`: CI/CD manifest files contain syntax errors or are structurally incorrect.
    *   `FileNotFound`: Expected CI/CD files are missing.

#### 4. `--full`

*   **Purpose**: Runs all available validation checks (`--config`, `--cluster`, `--ci`).
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   This flag acts as a convenience to execute all validation routines sequentially.
    *   It will report on the success or failure of each individual validation step.
*   **Output**:
    *   A summary of all validation checks, indicating which passed and which failed.
*   **Error Conditions**:
    *   Any error condition from `--config`, `--cluster`, or `--ci` will be reported.
