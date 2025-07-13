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
