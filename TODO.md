## CLI Improvements

### CLI Improvements

- [ ] Add tests for negative and edge cases:
  - `meshstack init` in an already-initialized directory.
  - `meshstack deploy` with missing config or malformed YAML.
  - `meshstack destroy` with no services defined.
  - Unknown flags or bad arguments should print clear errors.

- [ ] Mock external commands (`helm`, `kubectl`, `docker`) in CLI tests.
  - Use environment variable like `MESHSTACK_TEST_MODE=true` to short-circuit calls.

- [ ] Add fixture-based tests:
  - Verify config file parsing and merging.
  - Validate Helm chart scaffolding (if applicable).
  - Ensure template rendering logic works if present.

- [ ] Test expected side effects:
  - Files written or deleted by each command.
  - Workspace changes and structure.
  - Error formatting and handling (YAML parse failure, network errors, etc.)

- [ ] Build helper assertions for CLI tests:
  ```rust
  assert_cli("meshstack init")
      .succeeds()
      .prints("Project initialized")
      .creates_file("meshstack.yaml");
  ```

- [ ] Ensure tests are CI-compatible:
  - No reliance on local tools unless explicitly mocked.
  - Use temporary directories for workspace context.
  - Mark any long-running/external tool tests with `#[ignore]` for now.

## 🔍 Things to Watch / Improve

- Error handling:
  Some places (like `Command::output()`) use `?` without parsing the output unless failure is detected.
  Consider a shared `run_command()` utility to reduce duplication and centralize error output formatting.

- Repetition:
  The service/component/context repetition across commands could be refactored into utility structs or helper functions.

- Config loading:
  `MeshstackConfig` is currently only read during Init and Deploy.
  If the project grows, you may want a shared loader function or wrapper to validate + cache it.

- Placeholder logic:
  `validate_ci`, `deploy_service` (K8s deploy step), and others are stubs.
  It’s good that they’re scaffolded — just worth noting how much remains unimplemented.
