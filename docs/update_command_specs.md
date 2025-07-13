### `update` Command Functional Specifications

The `meshstack update` command is responsible for updating installed infrastructure components or generated project files.

#### 1. `--check`

*   **Purpose**: Shows available updates without applying them.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   When present, `meshstack` will scan for newer versions of installed components (e.g., Helm chart versions) and project templates.
    *   It will compare current versions against available upstream versions.
    *   No changes will be applied to the cluster or local files.
*   **Output**:
    *   A list of components/templates with available updates, including current and new versions.
    *   Warnings about potential breaking changes if known.
*   **Error Conditions**:
    *   `ConnectivityError`: Unable to reach Helm repositories or template sources.

#### 2. `--apply`

*   **Purpose**: Applies all detected updates automatically.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   When present, `meshstack` will proceed to download and apply all available updates for components and templates.
    *   This option should ideally be used after reviewing changes with `--check`.
*   **Output**:
    *   Confirmation messages for each successful update.
    *   Error messages for any failed updates.
*   **Error Conditions**:
    *   `UpdateFailure`: An update operation failed for a specific component or template.
    *   `PermissionDenied`: Insufficient permissions to apply updates.

#### 3. `--component <name>`

*   **Purpose**: Targets a specific infrastructure component for update.
*   **Input**: A string representing the name of an installed component (e.g., `istio`, `prometheus`).
*   **Behavior**:
    *   If provided, `meshstack` will only check for and apply updates for the specified component.
    *   This option can be combined with `--check` or `--apply`.
*   **Output**:
    *   Update status specific to the targeted component.
*   **Error Conditions**:
    *   `ComponentNotFound`: The specified component is not installed or recognized.
    *   `UpdateFailure`: Update failed for the specific component.

#### 4. `--template`

*   **Purpose**: Updates project templates (e.g., Dockerfile, Helm charts, CI/CD workflows) within the local project.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will compare the project's current templates with the latest versions provided by the CLI.
    *   If combined with `--apply`, it will overwrite or merge updated templates.
    *   If combined with `--check`, it will show a diff preview of changes.
*   **Output**:
    *   Messages indicating which templates were updated or what changes are available.
    *   Diff output if `--check` is used.
*   **Error Conditions**:
    *   `TemplateNotFound`: Expected template files are missing.
    *   `MergeConflict`: Unable to automatically merge template changes (requires manual intervention).

#### 5. `--infra`

*   **Purpose**: Updates infrastructure-related configurations, such as service mesh versions or base Helm chart versions.
*   **Input**: A boolean flag (its mere presence implies `true`).
*   **Behavior**:
    *   `meshstack` will update the versions of core infrastructure components (e.g., Istio, Linkerd) referenced in the project's configuration.
    *   This might involve updating `meshstack.yaml` or other internal configuration files.
    *   If combined with `--apply`, it will modify the configuration.
    *   If combined with `--check`, it will show proposed changes.
*   **Output**:
    *   Messages indicating which infrastructure versions were updated or what changes are available.
*   **Error Conditions**:
    *   `VersionMismatch`: Incompatible infrastructure versions detected.
    *   `ConfigurationError`: Unable to update infrastructure configuration.
