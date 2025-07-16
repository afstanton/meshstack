use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

mod utils;
use utils::CommandUnderTest;

#[test]
fn test_init_command()
{
    let temp_dir = tempdir().unwrap();
    CommandUnderTest::new(temp_dir.path())
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
fn test_init_command_already_initialized()
{
    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_path_buf();
    CommandUnderTest::new(temp_dir.path())
        .current_dir(&temp_dir_path)
        .arg("init")
        .assert()
        .success();

    // Run init again in the same directory
    CommandUnderTest::new(temp_dir.path())
        .current_dir(&temp_dir_path)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created meshstack.yaml")); // Should still say it created it, as it overwrites
}

#[test]
fn test_init_command_with_config_file()
{
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");
    let config_content = "project_name: my-test-app\nlanguage: generic\nservice_mesh: linkerd\nci_cd: argo";
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
    assert!(predicate::str::contains("language: generic").eval(&meshstack_yaml_content));
    assert!(predicate::str::contains("service_mesh: linkerd").eval(&meshstack_yaml_content));
    assert!(predicate::str::contains("ci_cd: argo").eval(&meshstack_yaml_content));
}

#[test]
fn test_init_command_with_name()
{
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("--name")
        .arg("my-named-app")
        .assert()
        .success();

    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let meshstack_yaml_content = fs::read_to_string(meshstack_yaml_path).unwrap();

    assert!(predicate::str::contains("project_name: my-named-app").eval(&meshstack_yaml_content));
    assert!(predicate::str::contains("language: generic").eval(&meshstack_yaml_content)); // Default language
    assert!(predicate::str::contains("service_mesh: istio").eval(&meshstack_yaml_content)); // Default service mesh
    assert!(predicate::str::contains("ci_cd: github").eval(&meshstack_yaml_content)); // Default CI/CD
}



#[test]
fn test_init_command_with_mesh()
{
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("--mesh")
        .arg("linkerd")
        .assert()
        .success();

    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let meshstack_yaml_content = fs::read_to_string(meshstack_yaml_path).unwrap();

    assert!(predicate::str::contains("project_name: my-app").eval(&meshstack_yaml_content)); // Default project name
    assert!(predicate::str::contains("language: generic").eval(&meshstack_yaml_content)); // Default language
    assert!(predicate::str::contains("service_mesh: linkerd").eval(&meshstack_yaml_content));
    assert!(predicate::str::contains("ci_cd: github").eval(&meshstack_yaml_content)); // Default CI/CD
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
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install nginx-ingress ingress-nginx/ingress-nginx --values prod-values.yaml"));
}

#[test]
fn test_install_command_with_dev_profile()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .arg("--profile")
        .arg("dev")
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing components..."))
        .stdout(predicate::str::contains("No component specified, installing default set."))
        .stdout(predicate::str::contains("Applying profile: dev"))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install istio istio/istio --values dev-values.yaml"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install prometheus prometheus-community/prometheus --values dev-values.yaml"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install grafana grafana/grafana --values dev-values.yaml"#))
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install cert-manager cert-manager/cert-manager --values dev-values.yaml"#))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install nginx-ingress ingress-nginx/ingress-nginx --values dev-values.yaml"));
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
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install nginx-ingress ingress-nginx/ingress-nginx --kube-context my-kube-context"));
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
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github: invalid_line"; // Invalid YAML
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
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
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
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
        .arg("validate")
        .arg("--cluster")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: kubectl cluster-info command failed:"))
        .stderr(predicate::str::contains("Unable to connect to the server: dial tcp 127.0.0.1:8080: connect: connection refused"));
}

#[test]
fn test_validate_ci_command()
{
    let temp_dir = tempdir().unwrap();
    let github_workflows_path = temp_dir.path().join(".github").join("workflows");
    fs::create_dir_all(&github_workflows_path).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("validate")
        .arg("--ci")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating CI/CD manifests..."))
        .stdout(predicate::str::contains("GitHub Actions workflows directory found."))
        .stdout(predicate::str::contains("ArgoCD manifests validation (placeholder): No issues found."));
}

#[test]
fn test_validate_ci_command_no_workflows_dir()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("validate")
        .arg("--ci")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating CI/CD manifests..."))
        .stdout(predicate::str::contains("GitHub Actions workflows directory not found. Skipping GitHub Actions validation."));
}

#[test]
fn test_validate_full_command_success()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create a mock kubectl that always succeeds
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Kubernetes master is running at https://127.0.0.1:8080'\nexit 0").unwrap();
    // Make it executable
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
        .stdout(predicate::str::contains("GitHub Actions workflows directory not found. Skipping GitHub Actions validation."))
        .stdout(predicate::str::contains("ArgoCD manifests validation (placeholder): No issues found."));
}

#[test]
fn test_validate_full_command_failure_config()
{
    let temp_dir = tempdir().unwrap();
    // No meshstack.yaml to simulate config validation failure

    // Create a mock kubectl that always succeeds
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Kubernetes master is running at https://127.0.0.1:8080'\nexit 0").unwrap();
    // Make it executable
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
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create a mock kubectl that always fails
    let mock_kubectl_path = temp_dir.path().join("kubectl");
    fs::write(&mock_kubectl_path, "#!/bin/bash\necho 'Unable to connect to the server: dial tcp 127.0.0.1:8080: connect: connection refused' >&2\nexit 1").unwrap();
    // Make it executable
    Command::new("chmod").arg("+x").arg(&mock_kubectl_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock kubectl to PATH
        .arg("validate")
        .arg("--full")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: kubectl cluster-info command failed:"))
        .stderr(predicate::str::contains("Unable to connect to the server: dial tcp 127.0.0.1:8080: connect: connection refused"));
}

#[test]
fn test_deploy_command_all_services()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"\nexit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"\nexit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm install success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker to PATH
        .arg("deploy")
        .arg("--build")
        .arg("--push")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Building Docker image for my-service (language: generic)..."))
        .stdout(predicate::str::contains("Successfully built Docker image: meshstack/my-service:latest"))
        .stdout(predicate::str::contains("Pushing Docker image for my-service to registry..."))
        .stdout(predicate::str::contains("Successfully pushed Docker image: meshstack/my-service:latest"))
        .stdout(predicate::str::contains("Deploying Helm chart for service: my-service..."))
        .stdout(predicate::str::contains("Successfully deployed service: my-service"))
        .stdout(predicate::str::contains("Deployment process completed."));
}

#[test]
fn test_deploy_command_specific_service()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-specific-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-specific-service\nversion: 0.1.0").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"\nexit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"\nexit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm install success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

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
        .stdout(predicate::str::contains("Building Docker image for my-specific-service (language: generic)..."))
        .stdout(predicate::str::contains("Successfully built Docker image: meshstack/my-specific-service:latest"))
        .stdout(predicate::str::contains("Pushing Docker image for my-specific-service to registry..."))
        .stdout(predicate::str::contains("Successfully pushed Docker image: meshstack/my-specific-service:latest"))
        .stdout(predicate::str::contains("Deploying Helm chart for service: my-specific-service..."))
        .stdout(predicate::str::contains("Successfully deployed service: my-specific-service"))
        .stdout(predicate::str::contains("Deployment process completed."));
}

#[test]
fn test_deploy_command_with_env()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm install success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path())
        .arg("deploy")
        .arg("--env")
        .arg("prod")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Applying environment profile: prod"))
        .stdout(predicate::str::contains("--- Deploying service: my-service ---"))
        .stdout(predicate::str::contains("Deploying Helm chart for service: my-service..."))
        .stdout(predicate::str::contains("Successfully deployed service: my-service"));
}

#[test]
fn test_deploy_command_deployment_fails()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock helm executable that always fails
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm install failure\" >&2\nexit 1\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock helm to PATH
        .arg("deploy")
        .arg("--service")
        .arg("my-service")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: helm upgrade --install meshstack-my-service command failed:"))
        .stderr(predicate::str::contains("Mock Helm install failure"));
}

#[test]
fn test_deploy_command_invalid_env()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("deploy")
        .arg("--env")
        .arg("invalid-env")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown environment: invalid-env. Valid environments are: dev, prod, staging"));
}

#[test]
fn test_deploy_command_with_build()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"\nexit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"\nexit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm install success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker and helm to PATH
        .arg("deploy")
        .arg("--build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Building Docker image for my-service (language: generic)..."))
        .stdout(predicate::str::contains("Mock Docker build success"))
        .stdout(predicate::str::contains("Deploying Helm chart for service: my-service..."))
        .stdout(predicate::str::contains("Successfully deployed service: my-service"));
}

#[test]
fn test_deploy_command_with_push()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock docker executable
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"\nexit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push success\"\nexit 0; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm install success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker and helm to PATH
        .arg("deploy")
        .arg("--push")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Pushing Docker image for my-service to registry..."))
        .stdout(predicate::str::contains("Mock Docker push success"))
        .stdout(predicate::str::contains("Deploying Helm chart for service: my-service..."))
        .stdout(predicate::str::contains("Successfully deployed service: my-service"));
}

#[test]
fn test_deploy_command_with_context()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm install success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock helm to PATH
        .arg("deploy")
        .arg("--context")
        .arg("my-kube-context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("Targeting Kubernetes context: my-kube-context"))
        .stdout(predicate::str::contains("--- Deploying service: my-service ---"))
        .stdout(predicate::str::contains("Deploying Helm chart for service: my-service..."))
        .stdout(predicate::str::contains("Successfully deployed service: my-service"));
}

#[test]
fn test_build_docker_image_dry_run()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_DOCKER", "1")
        .arg("deploy")
        .arg("--build")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN: Would execute docker command: docker build -t meshstack/my-service:latest"));
}

#[test]
fn test_push_docker_image_dry_run()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_DOCKER", "1")
        .arg("deploy")
        .arg("--push")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN: Would execute docker command: docker push meshstack/my-service:latest"));
}

#[test]
fn test_validate_cluster_dry_run()
{
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_KUBECTL", "1")
        .arg("validate")
        .arg("--cluster")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN: Would execute kubectl command: kubectl cluster-info --context current-context"));
}

#[test]
fn test_deploy_command_no_services_found()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("deploy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploying service..."))
        .stdout(predicate::str::contains("No services found to deploy."));
}

#[test]
fn test_deploy_command_services_dir_not_found()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("deploy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Services directory not found. Please run `meshstack init` first."));
}

#[test]
fn test_deploy_command_meshstack_yaml_not_found()
{
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("deploy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: meshstack.yaml not found or invalid. Run 'meshstack init' first."));
}

#[test]
fn test_deploy_command_dockerfile_not_found()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    // No Dockerfile

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("deploy")
        .arg("--build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Dockerfile not found"));
}

#[test]
fn test_deploy_command_docker_build_fails()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    // Create mock docker executable that always fails
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\necho \"Mock Docker build failure\" >&2\nexit 1\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker to PATH
        .arg("deploy")
        .arg("--build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: docker build command failed:"))
        .stderr(predicate::str::contains("Mock Docker build failure"));
}

#[test]
fn test_deploy_command_docker_push_fails()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    // Create mock docker executable that fails on push
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\nif [ \"$1\" = \"build\" ]; then echo \"Mock Docker build success\"\nexit 0; fi\nif [ \"$1\" = \"push\" ]; then echo \"Mock Docker push failure\" >&2; exit 1; fi\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_docker_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock docker to PATH
        .arg("deploy")
        .arg("--build")
        .arg("--push")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: docker push command failed:"))
        .stderr(predicate::str::contains("Mock Docker push failure"));
}

#[test]
fn test_destroy_command_with_confirmation()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("destroy")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroying project..."))
        .stdout(predicate::str::contains("Confirmation received. Proceeding with destruction."));
}




#[test]
fn test_destroy_command_with_service()
{
    let temp_dir = tempdir().unwrap();
    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm uninstall success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock helm to PATH
        .arg("destroy")
        .arg("--service")
        .arg("my-service")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroying project..."))
        .stdout(predicate::str::contains("Destroying service: my-service"))
        .stdout(predicate::str::contains("Uninstalling Helm release: meshstack-my-service..."))
        .stdout(predicate::str::contains("Successfully uninstalled Helm release: meshstack-my-service"));
}

#[test]
fn test_destroy_command_with_component()
{
    let temp_dir = tempdir().unwrap();
    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm uninstall success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock helm to PATH
        .arg("destroy")
        .arg("--component")
        .arg("istio")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroying project..."))
        .stdout(predicate::str::contains("Destroying component: istio"))
        .stdout(predicate::str::contains("Uninstalling Helm release: istio..."))
        .stdout(predicate::str::contains("Successfully uninstalled Helm release: istio"));
}

#[test]
fn test_destroy_command_with_full()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Chart.yaml"), "apiVersion: v2\nname: my-service\nversion: 0.1.0").unwrap();

    // Create mock helm executable
    let mock_helm_path = temp_dir.path().join("helm");
    fs::write(&mock_helm_path, "#!/bin/bash\necho \"Mock Helm uninstall success\"\nexit 0\n").unwrap();
    Command::new("chmod").arg("+x").arg(&mock_helm_path).status().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", temp_dir.path()) // Prepend mock helm to PATH
        .arg("destroy")
        .arg("--full")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroying project..."))
        .stdout(predicate::str::contains("Destroying all resources."))
        .stdout(predicate::str::contains("Uninstalling infrastructure component: istio"))
        .stdout(predicate::str::contains("Uninstalling infrastructure component: prometheus"))
        .stdout(predicate::str::contains("Uninstalling infrastructure component: grafana"))
        .stdout(predicate::str::contains("Uninstalling infrastructure component: cert-manager"))
        .stdout(predicate::str::contains("Uninstalling infrastructure component: nginx-ingress"))
        .stdout(predicate::str::contains("Uninstalling infrastructure component: vault"))
        .stdout(predicate::str::contains("Uninstalling service: my-service"))
        .stdout(predicate::str::contains("Successfully uninstalled Helm release: meshstack-my-service"));
}

#[test]
fn test_destroy_command_with_context()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("destroy")
        .arg("--context")
        .arg("my-kube-context")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroying project..."))
        .stdout(predicate::str::contains("Confirmation received. Proceeding with destruction."));
}

#[test]
fn test_update_command_check()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("update")
        .arg("--check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Checking for available updates..."))
        .stdout(predicate::str::contains("Available Updates:"));
}

#[test]
fn test_update_command_apply()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("update")
        .arg("--apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Checking for available updates..."))
        .stdout(predicate::str::contains("Applying updates..."))
        .stdout(predicate::str::contains("All updates applied successfully!"));
}

#[test]
fn test_update_command_specific_component()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("update")
        .arg("--component")
        .arg("istio")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Updating component: istio"))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm upgrade istio istio/istio --version 1.1.0"));
}

#[test]
fn test_update_command_invalid_component()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("update")
        .arg("--component")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown component: nonexistent. Valid components are: istio, prometheus, grafana, cert-manager, nginx-ingress, vault"));
}

#[test]
fn test_update_command_template()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("update")
        .arg("--template")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Updating project templates..."))
        .stdout(predicate::str::contains("Successfully updated base templates"));
}

#[test]
fn test_update_command_infra()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("update")
        .arg("--infra")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Updating infrastructure charts..."));
}

#[test]
fn test_update_command_no_config()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("update")
        .arg("--check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("meshstack.yaml not found or invalid. Run 'meshstack init' first."));
}

#[test]
fn test_status_command()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    CommandUnderTest::new(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."));
}

#[test]
fn test_status_command_components()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    CommandUnderTest::new(temp_dir.path())
        .arg("status")
        .arg("--components")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("--- Installed Infrastructure Components ---"))
        .stdout(predicate::str::contains("Service Mesh: istio"))
        .stdout(predicate::str::contains("Other components (placeholder): Prometheus, Grafana, Cert-Manager"));
}

#[test]
fn test_status_command_services()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    fs::create_dir_all(services_dir.join("my-service")).unwrap();

    CommandUnderTest::new(temp_dir.path())
        .arg("status")
        .arg("--services")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("--- Running App Services ---"))
        .stdout(predicate::str::contains("Service: my-service (Status: Running - placeholder)"));
}

#[test]
fn test_status_command_lockfile()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let lockfile_path = temp_dir.path().join("meshstack.lock");
    fs::write(&lockfile_path, "locked_component: v1.0").unwrap();

    CommandUnderTest::new(temp_dir.path())
        .arg("status")
        .arg("--lockfile")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("--- meshstack.lock Status ---"))
        .stdout(predicate::str::contains("Content of meshstack.lock:\nlocked_component: v1.0"));
}

#[test]
fn test_status_command_context()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .arg("--context")
        .arg("my-kube-context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("--- Kubernetes Context Status ---"))
        .stdout(predicate::str::contains("Targeting Kubernetes context: my-kube-context"))
        .stdout(predicate::str::contains("Kubernetes context status (placeholder): Connected"));
}

#[test]
fn test_status_command_all_flags() {
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    fs::create_dir_all(services_dir.join("my-service")).unwrap();

    let lockfile_path = temp_dir.path().join("meshstack.lock");
    fs::write(&lockfile_path, "locked_component: v1.0").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .arg("--components")
        .arg("--services")
        .arg("--lockfile")
        .arg("--context")
        .arg("my-kube-context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("--- Installed Infrastructure Components ---"))
        .stdout(predicate::str::contains("Service Mesh: istio"))
        .stdout(predicate::str::contains("Other components (placeholder): Prometheus, Grafana, Cert-Manager"))
        .stdout(predicate::str::contains("--- Running App Services ---"))
        .stdout(predicate::str::contains("Service: my-service (Status: Running - placeholder)"))
        .stdout(predicate::str::contains("--- meshstack.lock Status ---"))
        .stdout(predicate::str::contains("Content of meshstack.lock:\nlocked_component: v1.0"))
        .stdout(predicate::str::contains("--- Kubernetes Context Status ---"))
        .stdout(predicate::str::contains("Targeting Kubernetes context: my-kube-context"))
        .stdout(predicate::str::contains("Kubernetes context status (placeholder): Connected"));
}
#[test]
fn test_bootstrap_command_kind()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_CLUSTER", "1")
        .env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .env("MESHSTACK_TEST_DRY_RUN_KUBECTL", "1")
        .arg("bootstrap")
        .arg("--kind")
        .arg("--name")
        .arg("test-cluster")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bootstrapping local Kubernetes cluster..."))
        .stdout(predicate::str::contains("Using kind for local cluster provisioning"))
        .stdout(predicate::str::contains("DRY RUN: Would check if kind is installed"))
        .stdout(predicate::str::contains("DRY RUN: Would check if cluster 'test-cluster' exists"))
        .stdout(predicate::str::contains("DRY RUN: Would create cluster 'test-cluster' using kind"))
        .stdout(predicate::str::contains("DRY RUN: Would set kubectl context to 'kind-test-cluster'"))
        .stdout(predicate::str::contains("Installing infrastructure components..."))
        .stdout(predicate::str::contains("Local cluster bootstrap completed!"));
}

#[test]
fn test_bootstrap_command_k3d()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_CLUSTER", "1")
        .env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .env("MESHSTACK_TEST_DRY_RUN_KUBECTL", "1")
        .arg("bootstrap")
        .arg("--k3d")
        .arg("--name")
        .arg("test-cluster")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bootstrapping local Kubernetes cluster..."))
        .stdout(predicate::str::contains("Using k3d for local cluster provisioning"))
        .stdout(predicate::str::contains("DRY RUN: Would check if k3d is installed"))
        .stdout(predicate::str::contains("DRY RUN: Would check if cluster 'test-cluster' exists"))
        .stdout(predicate::str::contains("DRY RUN: Would create cluster 'test-cluster' using k3d"))
        .stdout(predicate::str::contains("DRY RUN: Would set kubectl context to 'k3d-test-cluster'"))
        .stdout(predicate::str::contains("Installing infrastructure components..."))
        .stdout(predicate::str::contains("Local cluster bootstrap completed!"));
}

#[test]
fn test_bootstrap_command_default_kind()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_CLUSTER", "1")
        .env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .env("MESHSTACK_TEST_DRY_RUN_KUBECTL", "1")
        .arg("bootstrap")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bootstrapping local Kubernetes cluster..."))
        .stdout(predicate::str::contains("Using kind for local cluster provisioning"))
        .stdout(predicate::str::contains("DRY RUN: Would check if kind is installed"))
        .stdout(predicate::str::contains("DRY RUN: Would check if cluster 'meshstack-dev' exists"))
        .stdout(predicate::str::contains("DRY RUN: Would create cluster 'meshstack-dev' using kind"))
        .stdout(predicate::str::contains("DRY RUN: Would set kubectl context to 'kind-meshstack-dev'"))
        .stdout(predicate::str::contains("Installing infrastructure components..."))
        .stdout(predicate::str::contains("Local cluster bootstrap completed!"));
}

#[test]
fn test_bootstrap_command_skip_install()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("MESHSTACK_TEST_DRY_RUN_CLUSTER", "1")
        .env("MESHSTACK_TEST_DRY_RUN_KUBECTL", "1")
        .arg("bootstrap")
        .arg("--skip-install")
        .arg("--name")
        .arg("test-cluster")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bootstrapping local Kubernetes cluster..."))
        .stdout(predicate::str::contains("Using kind for local cluster provisioning"))
        .stdout(predicate::str::contains("DRY RUN: Would check if kind is installed"))
        .stdout(predicate::str::contains("DRY RUN: Would check if cluster 'test-cluster' exists"))
        .stdout(predicate::str::contains("DRY RUN: Would create cluster 'test-cluster' using kind"))
        .stdout(predicate::str::contains("DRY RUN: Would set kubectl context to 'kind-test-cluster'"))
        .stdout(predicate::str::contains("Skipping infrastructure component installation"))
        .stdout(predicate::str::contains("Local cluster bootstrap completed!"));
}

#[test]
fn test_bootstrap_command_tool_not_found()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("PATH", "/nonexistent/path") // Set PATH to a directory that doesn't contain kind/k3d
        .arg("bootstrap")
        .arg("--kind")
        .assert()
        .failure()
        .stderr(predicate::str::contains("kind is not installed or not found in PATH. Please install kind to proceed."))
        .stderr(predicate::str::contains("Installation instructions:"));
}
#[test]
fn test_generate_command_specific_service()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("generate")
        .arg("--service")
        .arg("my-service")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generating scaffolds and configuration files..."))
        .stdout(predicate::str::contains("Generating scaffold for service: my-service"))
        .stdout(predicate::str::contains("Created service directory:"))
        .stdout(predicate::str::contains("Successfully generated"));

    // Verify files were created
    let service_dir = temp_dir.path().join("services").join("my-service");
    assert!(service_dir.exists());
    assert!(service_dir.join("Dockerfile").exists());
    assert!(service_dir.join("Chart.yaml").exists());
    assert!(service_dir.join("Cargo.toml").exists());
    assert!(service_dir.join("src").join("main.rs").exists());
}

#[test]
fn test_generate_command_all()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: node\nservice_mesh: linkerd\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create an existing service to test regeneration
    let existing_service_dir = temp_dir.path().join("services").join("existing-service");
    fs::create_dir_all(&existing_service_dir).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("generate")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generating scaffolds and configuration files..."))
        .stdout(predicate::str::contains("Re-generating all project scaffolds and configurations..."))
        .stdout(predicate::str::contains("Re-generating scaffold for existing service: existing-service"))
        .stdout(predicate::str::contains("Successfully generated"));

    // Verify project structure was created
    assert!(temp_dir.path().join("services").exists());
    assert!(temp_dir.path().join("provision").exists());
    assert!(temp_dir.path().join(".github").join("workflows").join("meshstack.yml").exists());
    assert!(temp_dir.path().join("dev-values.yaml").exists());
    assert!(temp_dir.path().join("prod-values.yaml").exists());

    // Verify existing service was regenerated
    assert!(existing_service_dir.join("Dockerfile").exists());
    assert!(existing_service_dir.join("package.json").exists());
}

#[test]
fn test_generate_command_default()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: generic\nservice_mesh: istio\nci_cd: argo";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generating scaffolds and configuration files..."))
        .stdout(predicate::str::contains("Re-generating project-level configurations..."))
        .stdout(predicate::str::contains("Successfully generated"));

    // Verify project-level files were created
    assert!(temp_dir.path().join("services").exists());
    assert!(temp_dir.path().join("provision").exists());
    assert!(temp_dir.path().join("argocd").join("application.yaml").exists());
    assert!(temp_dir.path().join("dev-values.yaml").exists());
}

#[test]
fn test_generate_command_force_overwrite()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: python\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create existing file
    let service_dir = temp_dir.path().join("services").join("test-service");
    fs::create_dir_all(&service_dir).unwrap();
    let dockerfile_path = service_dir.join("Dockerfile");
    fs::write(&dockerfile_path, "# Existing content").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("generate")
        .arg("--service")
        .arg("test-service")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generating scaffolds and configuration files..."))
        .stdout(predicate::str::contains("Generating scaffold for service: test-service"))
        .stdout(predicate::str::contains("Successfully generated"));

    // Verify file was overwritten (should contain Python-specific content)
    let dockerfile_content = fs::read_to_string(&dockerfile_path).unwrap();
    assert!(dockerfile_content.contains("FROM python:3.11-slim"));
}

#[test]
fn test_generate_command_no_force_existing_files()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create existing files
    let service_dir = temp_dir.path().join("services").join("test-service");
    fs::create_dir_all(&service_dir).unwrap();
    let dockerfile_path = service_dir.join("Dockerfile");
    fs::write(&dockerfile_path, "# Existing content").unwrap();
    let chart_path = service_dir.join("Chart.yaml");
    fs::write(&chart_path, "# Existing chart").unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("generate")
        .arg("--service")
        .arg("test-service")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generating scaffolds and configuration files..."))
        .stdout(predicate::str::contains("Generating scaffold for service: test-service"));

    // Verify files were not overwritten
    let dockerfile_content = fs::read_to_string(&dockerfile_path).unwrap();
    assert_eq!(dockerfile_content, "# Existing content");
    let chart_content = fs::read_to_string(&chart_path).unwrap();
    assert_eq!(chart_content, "# Existing chart");
}

#[test]
fn test_generate_command_no_config()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("generate")
        .arg("--service")
        .arg("test-service")
        .assert()
        .failure()
        .stderr(predicate::str::contains("meshstack.yaml not found or invalid. Run 'meshstack init' first."));
}

#[test]
fn test_generate_command_different_languages()
{
    let temp_dir = tempdir().unwrap();

    // Test Rust generation
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("generate")
        .arg("--service")
        .arg("rust-service")
        .assert()
        .success();

    let rust_service_dir = temp_dir.path().join("services").join("rust-service");
    assert!(rust_service_dir.join("Cargo.toml").exists());
    assert!(rust_service_dir.join("src").join("main.rs").exists());

    let dockerfile_content = fs::read_to_string(rust_service_dir.join("Dockerfile")).unwrap();
    assert!(dockerfile_content.contains("FROM rust:1.70 as builder"));
}
#[test]
fn test_plan_command_install()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("install")
        .arg("--component")
        .arg("istio")
        .assert()
        .success()
        .stdout(predicate::str::contains("Planning execution of 'install' command..."))
        .stdout(predicate::str::contains("Planning 'install' command execution:"))
        .stdout(predicate::str::contains("Components that would be installed:"))
        .stdout(predicate::str::contains("• istio (from chart: istio/istio)"))
        .stdout(predicate::str::contains("Prerequisites:"))
        .stdout(predicate::str::contains("Planning completed successfully!"));
}

#[test]
fn test_plan_command_deploy()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    // Create a service directory
    let service_dir = temp_dir.path().join("services").join("test-service");
    fs::create_dir_all(&service_dir).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("deploy")
        .arg("--service")
        .arg("test-service")
        .arg("--build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Planning execution of 'deploy' command..."))
        .stdout(predicate::str::contains("Planning 'deploy' command execution:"))
        .stdout(predicate::str::contains("Services that would be deployed:"))
        .stdout(predicate::str::contains("• test-service"))
        .stdout(predicate::str::contains("Deployment steps that would be executed:"))
        .stdout(predicate::str::contains("1. Build Docker images for services"))
        .stdout(predicate::str::contains("Planning completed successfully!"));
}

#[test]
fn test_plan_command_destroy()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("destroy")
        .arg("--service")
        .arg("my-service")
        .assert()
        .success()
        .stdout(predicate::str::contains("Planning execution of 'destroy' command..."))
        .stdout(predicate::str::contains("Planning 'destroy' command execution:"))
        .stdout(predicate::str::contains("Resources that would be destroyed:"))
        .stdout(predicate::str::contains("• Service: my-service (Helm release: meshstack-my-service)"))
        .stdout(predicate::str::contains("DANGER ZONE:"))
        .stdout(predicate::str::contains("Planning completed successfully!"));
}

#[test]
fn test_plan_command_update()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("update")
        .arg("--check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Planning execution of 'update' command..."))
        .stdout(predicate::str::contains("Planning 'update' command execution:"))
        .stdout(predicate::str::contains("Update operations that would be performed:"))
        .stdout(predicate::str::contains("• Check for available updates"))
        .stdout(predicate::str::contains("Prerequisites:"))
        .stdout(predicate::str::contains("Planning completed successfully!"));
}

#[test]
fn test_plan_command_bootstrap()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("bootstrap")
        .arg("--k3d")
        .arg("--name")
        .arg("test-cluster")
        .assert()
        .success()
        .stdout(predicate::str::contains("Planning execution of 'bootstrap' command..."))
        .stdout(predicate::str::contains("Planning 'bootstrap' command execution:"))
        .stdout(predicate::str::contains("Bootstrap operations that would be performed:"))
        .stdout(predicate::str::contains("• Create local Kubernetes cluster using k3d"))
        .stdout(predicate::str::contains("• Cluster name: test-cluster"))
        .stdout(predicate::str::contains("• Install infrastructure components (dev profile):"))
        .stdout(predicate::str::contains("Prerequisites:"))
        .stdout(predicate::str::contains("Planning completed successfully!"));
}

#[test]
fn test_plan_command_generate()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("generate")
        .arg("--service")
        .arg("new-service")
        .assert()
        .success()
        .stdout(predicate::str::contains("Planning execution of 'generate' command..."))
        .stdout(predicate::str::contains("Planning 'generate' command execution:"))
        .stdout(predicate::str::contains("Generation operations that would be performed:"))
        .stdout(predicate::str::contains("• Generate scaffold for service: new-service"))
        .stdout(predicate::str::contains("Current project configuration:"))
        .stdout(predicate::str::contains("• Language: rust"))
        .stdout(predicate::str::contains("Planning completed successfully!"));
}

#[test]
fn test_plan_command_verbose()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("install")
        .arg("--verbose")
        .arg("--component")
        .arg("prometheus")
        .assert()
        .success()
        .stdout(predicate::str::contains("Planning execution of 'install' command..."))
        .stdout(predicate::str::contains("Verbose mode enabled - showing detailed planning information"))
        .stdout(predicate::str::contains("- Helm command: helm install prometheus prometheus-community/prometheus"))
        .stdout(predicate::str::contains("Planning completed successfully!"));
}

#[test]
fn test_plan_command_invalid_command()
{
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--command")
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown command 'invalid-command' for planning. Supported commands: install, deploy, destroy, update, bootstrap, generate"));
}
