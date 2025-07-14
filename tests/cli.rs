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
    assert!(predicate::str::contains("language: rust").eval(&meshstack_yaml_content)); // Default language
    assert!(predicate::str::contains("service_mesh: istio").eval(&meshstack_yaml_content)); // Default service mesh
    assert!(predicate::str::contains("ci_cd: github").eval(&meshstack_yaml_content)); // Default CI/CD
}

#[test]
fn test_init_command_with_language()
{
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("--language")
        .arg("go")
        .assert()
        .success();

    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let meshstack_yaml_content = fs::read_to_string(meshstack_yaml_path).unwrap();

    assert!(predicate::str::contains("project_name: my-app").eval(&meshstack_yaml_content)); // Default project name
    assert!(predicate::str::contains("language: go").eval(&meshstack_yaml_content));
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
    assert!(predicate::str::contains("language: rust").eval(&meshstack_yaml_content)); // Default language
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
        .stdout(predicate::str::contains(r#"DRY RUN: Would execute helm command: helm install nginx-ingress ingress-nginx/ingress-nginx --values dev-values.yaml"#));
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
        .stdout(predicate::str::contains("GitHub Actions workflows directory not found. Skipping GitHub Actions validation."))
        .stdout(predicate::str::contains("ArgoCD manifests validation (placeholder): No issues found."));
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
        .stdout(predicate::str::contains("Deploying all services."))
        .stdout(predicate::str::contains("--- Deploying service: my-service ---"))
        .stdout(predicate::str::contains("Building Docker image for my-service (language: rust)..."))
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
        .stdout(predicate::str::contains("Deploying specific service: my-specific-service"))
        .stdout(predicate::str::contains("Building Docker image for my-specific-service (language: rust)..."))
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
        .stdout(predicate::str::contains("Building Docker image for my-service (language: rust)..."))
        .stdout(predicate::str::contains("Mock Docker build success"))
        .stdout(predicate::str::contains("Deploying Helm chart for service: my-service..."))
        .stdout(predicate::str::contains("Successfully deployed service: my-service"));
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
        .stderr(predicate::str::contains("Error: No such file or directory (os error 2)"));
}

#[test]
fn test_deploy_command_dockerfile_not_found()
{
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
    fs::write(&meshstack_yaml_path, config_content).unwrap();

    let service_dir = temp_dir.path().join("services").join("my-service");
    fs::create_dir_all(&service_dir).unwrap();
    fs::write(service_dir.join("Dockerfile"), "FROM alpine\nCMD echo \"Hello from Docker!\"").unwrap();

    // Create mock docker executable that always fails
    let mock_docker_path = temp_dir.path().join("docker");
    fs::write(&mock_docker_path, "#!/bin/bash\necho \"Mock Docker build failure\" >&2; exit 1\n").unwrap();
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
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
fn test_destroy_command_with_confirmation() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("destroy")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("Destroying project..."))
        .stdout(predicate::str::contains("Confirmation received. Proceeding with destruction."));
}




#[test]
fn test_destroy_command_with_service() {
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
fn test_destroy_command_with_component() {
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
fn test_destroy_command_with_full() {
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
fn test_destroy_command_with_context() {
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
fn test_update_command() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("update")
        .arg("--check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Checking for available updates..."));
}

#[test]
fn test_status_command() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."));
}

#[test]
fn test_status_command_components() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("status")
        .arg("--components")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("Showing installed infrastructure and versions..."));
}

#[test]
fn test_status_command_services() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("status")
        .arg("--services")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("Showing running app services..."));
}

#[test]
fn test_status_command_lockfile() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("status")
        .arg("--lockfile")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("Comparing current state with meshstack.lock..."));
}

#[test]
fn test_status_command_context() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("status")
        .arg("--context")
        .arg("my-kube-context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing project status..."))
        .stdout(predicate::str::contains("Showing per-kube-context state for: my-kube-context"));
}



#[test]
fn test_destroy_command_all() {
    let temp_dir = tempdir().unwrap();
    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let config_content = "project_name: my-app\nlanguage: rust\nservice_mesh: istio\nci_cd: github";
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
        .arg("--all")
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
fn test_update_command_apply() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("update")
        .arg("--apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Applying all updates automatically..."));
}

#[test]
fn test_init_command_with_ci()
{
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("--ci")
        .arg("argo")
        .assert()
        .success();

    let meshstack_yaml_path = temp_dir.path().join("meshstack.yaml");
    let meshstack_yaml_content = fs::read_to_string(meshstack_yaml_path).unwrap();

    assert!(predicate::str::contains("project_name: my-app").eval(&meshstack_yaml_content)); // Default project name
    assert!(predicate::str::contains("language: rust").eval(&meshstack_yaml_content)); // Default language
    assert!(predicate::str::contains("service_mesh: istio").eval(&meshstack_yaml_content)); // Default service mesh
    assert!(predicate::str::contains("ci_cd: argo").eval(&meshstack_yaml_content));
}

#[test]
fn test_install_command_dry_run()
{
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.env("MESHSTACK_TEST_DRY_RUN_HELM", "1")
        .arg("install")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing components..."))
        .stdout(predicate::str::contains("DRY RUN: Would execute helm command: helm install istio istio/istio --dry-run"));
}

#[test]
fn test_update_command_with_component() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("update")
        .arg("--component")
        .arg("istio")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Updating component: istio"));
}

#[test]
fn test_update_command_template() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("update")
        .arg("--template")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Updating project templates..."));
}

#[test]
fn test_update_command_infra() {
    let mut cmd = Command::cargo_bin("meshstack").unwrap();
    cmd.arg("update")
        .arg("--infra")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating project..."))
        .stdout(predicate::str::contains("Updating infra charts..."));
}
