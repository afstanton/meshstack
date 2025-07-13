use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_init_command()
{
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing new meshstack project..."))
        .stdout(predicate::str::contains("Created meshstack.yaml"));

    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    assert!(meshstack_yaml_path.exists());

    let services_path = temp_dir.path().join("services");
    assert!(services_path.exists());

    let provision_path = temp_dir.path().join("provision");
    assert!(provision_path.exists());
}

#[test]
fn test_init_command_with_config_file()
{
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");
    let config_content = "project_name: my-test-app\nlanguage: go\nservice_mesh: linkerd\nci_cd: argo";
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("--config")
        .arg(config_path)
        .assert()
        .success();

    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let meshstack_yaml_content = fs::read_to_string(meshstack_yaml_path).unwrap();

    assert!(predicate::str::contains("project_name: my-test-app").eval(&meshstack_yaml_content));
    assert!(predicate::str::contains("language: go").eval(&meshstack_yaml_content));
    assert!(predicate::str::contains("service_mesh: linkerd").eval(&meshstack_yaml_content));
    assert!(predicate::str::contains("ci_cd: argo").eval(&meshstack_yaml_content));
}

#[test]
fn test_install_command()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing components..."))
        .stdout(predicate::str::contains("No component specified, installing default set."))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install istio istio/istio"))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install prometheus prometheus-community/prometheus"))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install grafana grafana/grafana"))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install cert-manager cert-manager/cert-manager"))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install nginx-ingress ingress-nginx/ingress-nginx"));
}

#[test]
fn test_install_command_with_component()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .arg("--component")
        .arg("istio")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"Installing components..."#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install istio istio/istio"#));
}

#[test]
fn test_install_command_with_invalid_component()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .arg("--component")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown component: nonexistent. Valid components are: istio, prometheus, grafana, cert-manager, nginx-ingress, vault"));
}

#[test]
fn test_install_command_with_profile()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .arg("--profile")
        .arg("prod")
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing components..."))
        .stdout(predicate::str::contains("No component specified, installing default set."))
        .stdout(predicate::str::contains("Applying profile: prod"))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install istio istio/istio --values prod-values.yaml"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install prometheus prometheus-community/prometheus --values prod-values.yaml"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install grafana grafana/grafana --values prod-values.yaml"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install cert-manager cert-manager/cert-manager --values prod-values.yaml"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install nginx-ingress ingress-nginx/ingress-nginx --values prod-values.yaml"#));
}

#[test]
fn test_install_command_with_context()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .arg("--context")
        .arg("my-kube-context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing components..."))
        .stdout(predicate::str::contains("No component specified, installing default set."))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install istio istio/istio --kube-context my-kube-context"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install prometheus prometheus-community/prometheus --kube-context my-kube-context"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install grafana grafana/grafana --kube-context my-kube-context"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install cert-manager cert-manager/cert-manager --kube-context my-kube-context"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install nginx-ingress ingress-nginx/ingress-nginx --kube-context my-kube-context"#));
}

#[test]
fn test_install_command_helm_not_found()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("PATH", "/nonexistent/path") // Set PATH to a directory that doesn't contain helm
        .arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Helm is not installed or not found in PATH. Please install Helm to proceed. Refer to https://helm.sh/docs/intro/install/ for instructions."));
}

#[test]
fn test_install_command_with_custom_profile()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .arg("--profile")
        .arg("custom")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Custom profile not yet implemented."));
}

#[test]
fn test_validate_config_command_success()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("validate")
        .arg("--config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating project..."))
        .stdout(predicate::str::contains("Validating meshstack.yaml..."))
        .stdout(predicate::str::contains("meshstack.yaml is valid."));
}

#[test]
fn test_validate_config_command_file_not_found()
{
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("meshstack.yaml not found."));
}

#[test]
fn test_validate_config_command_invalid_yaml()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github: invalid_line"; // Invalid YAML
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: mapping values are not allowed in this context"));
}

#[test]
fn test_validate_cluster_command_success()
{
    let temp_dir = tempdir().unwrap();
    // Create a mock kubectl that always succeeds
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Kubernetes master is running at https://127.0.0.1:8080'\nexit 0").unwrap();
    // Make it executable
    Command::new("chmod").arg("+x").arg(&mock_kubectl_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
        .arg("validate")
        .arg("--cluster")
        .assert()
        .success()
        .stdout(predicate::str::contains("Checking Kubernetes cluster connectivity..."))
        .stdout(predicate::str::contains("Connected to Kubernetes cluster successfully."));
}

#[test]
fn test_validate_cluster_command_failure()
{
    let temp_dir = tempdir().unwrap();
    // Create a mock kubectl that always fails
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Unable to connect to the server: dial tcp 127.0.0.1:8080: connect: connection refused' >&2\nexit 1").unwrap();
    // Make it executable
    Command::new("chmod").arg("+x").arg(&mock_kubectl_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
        .arg("validate")
        .arg("--cluster")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to connect to Kubernetes cluster"));
}

#[test]
fn test_validate_ci_command()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("validate")
        .arg("--ci")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating CI/CD manifests..."))
        .stdout(predicate::str::contains("CI/CD manifests validation (placeholder): No issues found."));
}

#[test]
fn test_validate_full_command_success()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create a mock kubectl that always succeeds
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Kubernetes master is running at https://127.0.0.1:8080'\nexit 0").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_kubectl_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
        .arg("validate")
        .arg("--full")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating project..."))
        .stdout(predicate::str::contains("meshstack.yaml is valid."))
        .stdout(predicate::str::contains("Connected to Kubernetes cluster successfully."))
        .stdout(predicate::str::contains("CI/CD manifests validation (placeholder): No issues found."));
}

#[test]
fn test_validate_full_command_failure_config()
{
    let temp_dir = tempdir().unwrap();
    // No meshstack.yaml to simulate config validation failure

    // Create a mock kubectl that always succeeds
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Kubernetes master is running at https://127.0.0.1:8080'\nexit 0").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_kubectl_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
        .arg("validate")
        .arg("--full")
        .assert()
        .failure()
        .stderr(predicate::str::contains("meshstack.yaml not found."));
}

#[test]
fn test_validate_full_command_failure_cluster()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create a mock kubectl that always fails
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Unable to connect to the server: dial tcp 127.0.0.1:8080: connect: connection refused' >&2\nexit 1").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_kubectl_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
        .arg("validate")
        .arg("--full")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to connect to Kubernetes cluster"));
}

#[test]
fn test_deploy_command_all_services()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"; exit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"; exit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker to PATH
        .arg("deploy")
        .arg("--build")
        .arg("--push")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Deploying all services."))
        .stdout(predicate::str::contains("--- Deploying service: my-service ---"))
        .stdout(predicate::str::contains("Building Docker image for my-service (language: rust)..."))
        .stdout(predicate::str::contains("Successfully built Docker image: meshstack/my-service:latest"))
        .stdout(predicate::str::contains("Pushing Docker image for my-service to registry..."))
        .stdout(predicate::str::contains("Successfully pushed Docker image: meshstack/my-service:latest"))
        .stdout(predicate::str::contains("Kubernetes deployment logic (placeholder) for service: my-service."))
        .stdout(predicate::str::contains("Deployment process completed."));
}

#[test]
fn test_deploy_command_specific_service()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-specific-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"; exit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"; exit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker to PATH
        .arg("deploy")
        .arg("--service")
        .arg("my-specific-service")
        .arg("--build")
        .arg("--push")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Deploying specific service: my-specific-service"))
        .stdout(predicate::str::contains("Building Docker image for my-specific-service (language: rust)..."))
        .stdout(predicate::str::contains("Successfully built Docker image: meshstack/my-specific-service:latest"))
        .stdout(predicate::str::contains("Pushing Docker image for my-specific-service to registry..."))
        .stdout(predicate::str::contains("Successfully pushed Docker image: meshstack/my-specific-service:latest"))
        .stdout(predicate::str::contains("Kubernetes deployment logic (placeholder) for service: my-specific-service."))
        .stdout(predicate::str::contains("Deployment process completed."));
}

#[test]
fn test_deploy_command_with_env()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine
CMD echo \"Hello from Docker!\"").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("deploy")
        .arg("--env")
        .arg("prod")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Applying environment profile: prod"))
        .stdout(predicate::str::contains("--- Deploying service: my-service ---"))
        .stdout(predicate::str::contains("Kubernetes deployment logic (placeholder) for service: my-service."))
        .stdout(predicate::str::contains("Deployment process completed."));
}

#[test]
fn test_deploy_command_with_build()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"; exit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"; exit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker to PATH
        .arg("deploy")
        .arg("--build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Building Docker image for my-service (language: rust)..."))
        .stdout(predicate::str::contains("Mock Docker build success"))
        .stdout(predicate::str::contains("Kubernetes deployment logic (placeholder) for service: my-service."));
}

#[test]
fn test_deploy_command_with_push()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"; exit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"; exit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker to PATH
        .arg("deploy")
        .arg("--push")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Pushing Docker image for my-service to registry..."))
        .stdout(predicate::str::contains("Mock Docker push success"))
        .stdout(predicate::str::contains("Kubernetes deployment logic (placeholder) for service: my-service."));
}

#[test]
fn test_deploy_command_with_context()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("deploy")
        .arg("--context")
        .arg("my-kube-context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Targeting Kubernetes context: my-kube-context"))
        .stdout(predicate::str::contains("--- Deploying service: my-service ---"))
        .stdout(predicate::str::contains("Kubernetes deployment logic (placeholder) for service: my-service."))
        .stdout(predicate::str::contains("Deployment process completed."));
}
