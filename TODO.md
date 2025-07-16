## CLI Improvements

### CLI Improvements

- [x] Add tests for negative and edge cases (covered for `init`, `install`, `validate`, `deploy`, `destroy`):

  - `meshstack init` in an already-initialized directory.
  - `meshstack deploy` with missing config or malformed YAML.
  - `meshstack destroy` with no services defined.
  - Unknown flags or bad arguments should print clear errors.

- [x] Mock external commands (`helm`, `kubectl`, `docker`) in CLI tests.

  - ✅ Implemented using environment variables: `MESHSTACK_TEST_DRY_RUN_HELM`, `MESHSTACK_TEST_DRY_RUN_DOCKER`, `MESHSTACK_TEST_DRY_RUN_KUBECTL`

- [x] Add fixture-based tests:

  - Verify config file parsing and merging.
  - Validate Helm chart scaffolding (if applicable).
  - Ensure template rendering logic works if present.

- [x] Test expected side effects:

  - ✅ Files written or deleted by each command (52 comprehensive tests)
  - ✅ Workspace changes and structure
  - ✅ Error formatting and handling (YAML parse failure, network errors, etc.)

- [x] Ensure tests are CI-compatible:

  - ✅ No reliance on local tools unless explicitly mocked
  - ✅ Use temporary directories for workspace context
  - ✅ All external tool interactions are properly mocked

- [x] Build helper assertions for CLI tests:
  ```rust
  assert_cli("meshstack init")
      .succeeds()
      .prints("Project initialized")
      .creates_file("meshstack.yaml");
  ```
  - ✅ **COMPLETED**: Implemented `CommandUnderTest` utility in `tests/utils.rs`
  - ✅ Provides fluent interface for test assertions with automatic template copying
  - ✅ All 52 tests use comprehensive helper methods for clean, readable test code

## 🔍 Things to Watch / Improve

- [x] ~~Error handling:~~
      ✅ **COMPLETED**: Implemented shared `run_command()` utility function that centralizes error output formatting and reduces duplication across all external command calls.

- [x] ~~Repetition:~~
      ✅ **COMPLETED**: Refactored common parameter patterns into `MeshstackContext` utility struct

  - ✅ Created `MeshstackContext` struct to encapsulate common parameters (config, kube_context, dry_run)
  - ✅ Eliminated repetitive context parameter passing across all commands
  - ✅ Added helper methods for consistent Kubernetes context handling

- [x] ~~Config loading:~~
      ✅ **COMPLETED**: Implemented centralized config loading with proper error handling

  - ✅ Added shared `load_config()` and `require_config()` methods
  - ✅ Standardized config access patterns across all commands
  - ✅ Improved error messages with user-friendly feedback

- [x] ~~Language-specific features:~~
      ✅ **COMPLETED**: Removed `--language` option and language-specific code generation features

  - Meshstack now focuses purely on mesh infrastructure management
  - Default language set to "generic" for language-agnostic approach

- [ ] **Placeholder logic:**
      Several areas still need full implementation:
  - `validate_ci`: Only checks for `.github/workflows` directory existence, no YAML validation or workflow analysis
  - ✅ ~~`update_project`: All functionality prints placeholder messages - no actual update logic implemented~~
    - ✅ **COMPLETED**: Implemented comprehensive update system with version checking and application
    - ✅ `--check`: Queries for available component/template updates with detailed output
    - ✅ `--apply`: Performs actual updates to Helm charts and templates
    - ✅ `--component`: Updates specific infrastructure components with version management
    - ✅ `--template`: Regenerates project templates from latest versions
    - ✅ `--infra`: Updates infrastructure chart versions with proper Helm integration
    - ✅ Added 7 comprehensive tests covering all update scenarios
  - ✅ ~~`bootstrap` command: Documented in specs but not implemented in CLI~~
    - ✅ **COMPLETED**: Implemented comprehensive bootstrap functionality for local development
    - ✅ Supports both Kind and k3d cluster provisioning tools
    - ✅ Automatic cluster creation with development-friendly configuration
    - ✅ Automatic infrastructure component installation with dev profile
    - ✅ Proper kubectl context management and cluster naming
    - ✅ Added 5 comprehensive tests covering all bootstrap scenarios
  - ✅ ~~`generate` command: Documented in specs but not implemented in CLI~~
    - ✅ **COMPLETED**: Implemented comprehensive scaffold generation functionality
    - ✅ Supports service-specific scaffold generation with `--service` flag
    - ✅ Full project regeneration with `--all` flag for complete project scaffolds
    - ✅ Language-specific file generation (Rust, Node.js, Python, Go, generic)
    - ✅ Helm chart generation with proper templates and values files
    - ✅ CI/CD workflow generation (GitHub Actions, ArgoCD)
    - ✅ Environment-specific values files (dev, prod, staging)
    - ✅ Force overwrite functionality with `--force` flag
    - ✅ Added 7 comprehensive tests covering all generation scenarios
  - ✅ ~~`plan` command: Documented in specs but not implemented in CLI~~
    - ✅ **COMPLETED**: Implemented comprehensive dry-run planning functionality
    - ✅ Supports planning for all major commands (install, deploy, destroy, update, bootstrap, generate)
    - ✅ Detailed argument parsing and command-specific planning logic
    - ✅ Verbose mode for detailed execution plans and prerequisites
    - ✅ Clear output showing what would be executed without side effects
    - ✅ Helpful command suggestions for executing planned changes
    - ✅ Added 8 comprehensive tests covering all planning scenarios

## 📊 Current Project Status (v0.1.16)

- **Tests**: 78 comprehensive tests covering all major functionality ✅
- **Commands**: 10 core commands implemented (`init`, `bootstrap`, `generate`, `plan`, `install`, `validate`, `deploy`, `destroy`, `update`, `status`) ✅
- **Bootstrap System**: Full local cluster provisioning with Kind/k3d support and automatic infrastructure setup ✅
- **Generate System**: Comprehensive scaffold generation with language-specific templates and CI/CD workflows ✅
- **Update System**: Comprehensive update functionality with version checking and Helm chart management ✅
- **Mocking**: Full test mocking for external dependencies (helm, kubectl, docker, cluster tools) ✅
- **Error Handling**: Centralized error handling with proper formatting ✅
- **Documentation**: Complete command specifications and usage docs ✅
- **Code Quality**: All compiler warnings fixed, clean codebase ✅
- **Test Infrastructure**: Robust `CommandUnderTest` utility with automatic template management ✅
- **CI/CD**: GitHub Actions workflow for automated releases to crates.io and Homebrew ✅

## 🎯 Next Priority Items

### High Priority (Enhanced Functionality)

3. **Improve validation capabilities:**
   - Enhance `validate_ci` with actual GitHub Actions YAML parsing and validation
   - Add schema validation for `meshstack.yaml` beyond basic YAML parsing
   - Implement drift detection between actual cluster state and expected configuration

4. **User experience enhancements:**
   - Add progress indicators for long-running operations (Helm installs, Docker builds)
   - Implement colored output for better readability
   - Add interactive prompts for destructive operations
   - Improve error messages with actionable suggestions and troubleshooting links

### Low Priority (Nice to Have)

5. **Advanced features:**
   - Add configuration caching and performance optimizations
   - Implement plugin system for custom components
   - Add support for additional service meshes beyond Istio/Linkerd
   - Consider splitting large functions into smaller, focused ones (current code is already well-structured)
