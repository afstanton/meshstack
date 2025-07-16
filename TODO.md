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

- [ ] Build helper assertions for CLI tests:
  ```rust
  assert_cli("meshstack init")
      .succeeds()
      .prints("Project initialized")
      .creates_file("meshstack.yaml");
  ```
  - Current tests use `assert_cmd` crate with comprehensive assertions
  - Could be refactored to use more fluent helper methods

## 🔍 Things to Watch / Improve

- [x] ~~Error handling:~~
  ✅ **COMPLETED**: Implemented shared `run_command()` utility function that centralizes error output formatting and reduces duplication across all external command calls.

- [ ] **Repetition:**
  The service/component/context repetition across commands could be refactored into utility structs or helper functions.
  - Consider creating a `MeshstackContext` struct to encapsulate common parameters
  - Extract common validation logic into shared functions

- [ ] **Config loading:**
  `MeshstackConfig` is currently read during Init, Deploy, and Status commands.
  - Consider implementing a shared config loader function
  - Add config validation and caching for better performance
  - Standardize config access patterns across all commands

- [x] ~~Language-specific features:~~
  ✅ **COMPLETED**: Removed `--language` option and language-specific code generation features
  - Meshstack now focuses purely on mesh infrastructure management
  - Default language set to "generic" for language-agnostic approach

- [ ] **Placeholder logic:**
  Several areas still need full implementation:
  - `validate_ci`: Basic GitHub Actions detection, but no deep validation
  - `update_project`: All functionality is stubbed out
  - `bootstrap` command: Planned but not yet implemented
  - `generate` command: Planned but not yet implemented
  - `plan` command: Planned but not yet implemented

## 📊 Current Project Status

- **Tests**: 52 comprehensive tests covering all major functionality ✅
- **Commands**: 7 core commands implemented (`init`, `install`, `validate`, `deploy`, `destroy`, `update`, `status`) ✅
- **Mocking**: Full test mocking for external dependencies (helm, kubectl, docker) ✅
- **Error Handling**: Centralized error handling with proper formatting ✅
- **Documentation**: Complete command specifications and usage docs ✅
- **Code Quality**: All compiler warnings fixed, clean codebase ✅

## 🎯 Next Priority Items

1. **Implement missing commands:**
   - `bootstrap` - Set up local Kubernetes cluster and install infrastructure
   - `plan` - Dry-run preview of changes before applying them

2. **Enhance existing functionality:**
   - Complete `update` command implementation
   - Improve `validate_ci` with deeper GitHub Actions validation
   - Add more sophisticated config validation and caching

3. **Code organization improvements:**
   - Refactor common parameter patterns into utility structs
   - Create shared config loading utilities
   - Consider splitting large functions into smaller, focused ones

4. **User experience enhancements:**
   - Add progress indicators for long-running operations
   - Improve error messages with actionable suggestions
   - Add colored output for better readability
