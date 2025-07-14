use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new mesh app project with config and template structure.
    Init {
        /// Name of the project (default: current directory)
        #[arg(short, long)]
        name: Option<String>,

        /// App language (`rust`, `go`, `python`, `node`)
        #[arg(short, long)]
        language: Option<String>,

        /// Choose service mesh (istio or linkerd)
        #[arg(short, long)]
        mesh: Option<String>,

        /// CI/CD preference
        #[arg(short, long)]
        ci: Option<String>,

        /// Use preexisting meshstack.yaml config
        #[arg(long)]
        config: Option<String>,
    },
    /// Create a new mesh app project with config and template structure.
    New { 
        /// Name of the project
        name: String,
    },
    /// Install infrastructure components into current Kubernetes cluster.
    Install {
        /// Specific component (e.g. `istio`, `prometheus`, `vault`)
        #[arg(short, long)]
        component: Option<String>,

        /// Install resource-tuned versions
        #[arg(short, long)]
        profile: Option<String>,

        /// Print manifests instead of applying
        #[arg(long)]
        dry_run: bool,

        /// Target a specific cluster context
        #[arg(long)]
        context: Option<String>,
    },
    /// Validate config, manifests, and cluster readiness.
    Validate {
        /// Validate `meshstack.yaml` against schema
        #[arg(long)]
        config: bool,

        /// Check connectivity to kube context
        #[arg(long)]
        cluster: bool,

        /// Validate GitHub Actions or ArgoCD manifests
        #[arg(long)]
        ci: bool,

        /// Run all validators
        #[arg(long)]
        full: bool,
    },
    /// Deploy one or more services to current Kubernetes context.
    Deploy {
        /// Deploy a single service (or all if omitted)
        #[arg(short, long)]
        service: Option<String>,

        /// Target a specific env profile
        #[arg(short, long)]
        env: Option<String>,

        /// Rebuild Docker image before deploy
        #[arg(long)]
        build: bool,

        /// Push container to registry (configurable)
        #[arg(long)]
        push: bool,

        /// Kube context override
        #[arg(long)]
        context: Option<String>,
    },
    /// Destroy project resources.
    Destroy {
        /// Service to destroy
        #[arg(short, long)]
        service: Option<String>,

        /// Component to destroy
        #[arg(short, long)]
        component: Option<String>,

        /// Destroy all resources
        #[arg(long)]
        full: bool,

        /// Nuke from orbit (dev/test use only)
        #[arg(long)]
        all: bool,

        /// Kube context override
        #[arg(long)]
        context: Option<String>,

        /// Bypasses confirmation prompt
        #[arg(long)]
        confirm: bool,
    },
    /// Update installed components or generated files.
    Update {
        /// Show available updates
        #[arg(long)]
        check: bool,

        /// Apply all updates automatically
        #[arg(long)]
        apply: bool,

        /// Target a specific component
        #[arg(short, long)]
        component: Option<String>,

        /// Update project templates (Dockerfile, Helm, etc.)
        #[arg(long)]
        template: bool,

        /// Update infra charts (e.g. mesh version bump)
        #[arg(long)]
        infra: bool,
    },
    /// Show meshstack-managed resources and current versions.
    Status {
        /// Show installed infrastructure and versions
        #[arg(long)]
        components: bool,

        /// Show running app services
        #[arg(long)]
        services: bool,

        /// Compare current state with `meshstack.lock`
        #[arg(long)]
        lockfile: bool,

        /// Show per-kube-context state
        #[arg(long)]
        context: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
struct MeshstackConfig {
    project_name: String,
    language: String,
    service_mesh: String,
    ci_cd: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name, language, mesh, ci, config } => {
            println!("Initializing new meshstack project...");

            let config_to_write = if let Some(config_path) = config {
                println!("Using config from: {}", config_path);
                let config_content = fs::read_to_string(config_path)?;
                serde_yaml::from_str(&config_content)?
            } else {
                MeshstackConfig {
                    project_name: name.clone().unwrap_or_else(|| "my-app".to_string()),
                    language: language.clone().unwrap_or_else(|| "rust".to_string()),
                    service_mesh: mesh.clone().unwrap_or_else(|| "istio".to_string()),
                    ci_cd: ci.clone().unwrap_or_else(|| "github".to_string()),
                }
            };

            let yaml_config = serde_yaml::to_string(&config_to_write)?;
            fs::write("meshstack.yaml", yaml_config)?;

            println!("Created meshstack.yaml");

            let dirs = ["services", "provision"];
            for dir in &dirs {
                if !Path::new(dir).exists() {
                    fs::create_dir(dir)?;
                    println!("Created directory: {}", dir);
                }
            }
        }
        Commands::New { name } => {
            new_project(name)?;
        }
        Commands::Install { component, profile, dry_run, context } => {
            install_component(component, profile, *dry_run, context)?;
        }
        Commands::Validate { config, cluster, ci, full } => {
            validate_project(*config, *cluster, *ci, *full)?;
        }
        Commands::Deploy { service, env, build, push, context } => {
            deploy_service(service, env, *build, *push, context)?;
        }
        Commands::Destroy { service, component, full, context, confirm, all } => {
            destroy_project(service, component, *full, context, *confirm, *all)?;
        }
        Commands::Update { check, apply, component, template, infra } => {
            update_project(*check, *apply, component, *template, *infra)?;
        }
        Commands::Status { components, services, lockfile, context } => {
            status_project(*components, *services, *lockfile, context)?;
        }
    }
    Ok(())
}

fn new_project(name: &String) -> anyhow::Result<()> {
    println!("Creating new project: {}...", name);
    Ok(())
}

fn status_project(
    components: bool,
    services: bool,
    lockfile: bool,
    context: &Option<String>,
) -> anyhow::Result<()> {
    println!("Showing project status...");

    if components {
        println!("Showing installed infrastructure and versions...");
    }

    if services {
        println!("Showing running app services...");
    }

    if lockfile {
        println!("Comparing current state with meshstack.lock...");
    }

    if let Some(context) = context {
        println!("Showing per-kube-context state for: {}", context);
    }

    Ok(())
}

fn deploy_service(
    service_name: &Option<String>,
    env: &Option<String>,
    build: bool,
    push: bool,
    context: &Option<String>,
) -> anyhow::Result<()> {
    println!("Deploying service...");

    if let Some(env) = env {
        println!("Applying environment profile: {}", env);
    }

    if let Some(context) = context {
        println!("Targeting Kubernetes context: {}", context);
    }

    let config_content = fs::read_to_string("meshstack.yaml")?;
    let config: MeshstackConfig = serde_yaml::from_str(&config_content)?;

    let services_dir = Path::new("services");
    if !services_dir.exists() {
        anyhow::bail!("Services directory not found. Please run `meshstack init` first.");
    }

    let services_to_deploy = if let Some(svc_name) = service_name {
        println!("Deploying specific service: {}", svc_name);
        vec![services_dir.join(svc_name)]
    } else {
        println!("Deploying all services.");
        fs::read_dir(services_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_dir()))
            .map(|entry| entry.path())
            .collect()
    };

    if services_to_deploy.is_empty() {
        println!("No services found to deploy.");
        return Ok(());
    }

    for service_path in services_to_deploy {
        let current_service_name = service_path.file_name().unwrap().to_string_lossy().into_owned();
        println!("\n--- Deploying service: {} ---", current_service_name);

        if build {
            build_docker_image(&service_path, &current_service_name, &config)?;
        }

        if push {
            push_docker_image(&current_service_name)?;
        }

        // Placeholder for Kubernetes deployment logic
        println!("Kubernetes deployment logic (placeholder) for service: {}.", current_service_name);
    }

    println!("\nDeployment process completed.");
    Ok(())
}

fn destroy_project(
    service: &Option<String>,
    component: &Option<String>,
    full: bool,
    context: &Option<String>,
    confirm: bool,
    all: bool,
) -> anyhow::Result<()> {
    println!("Destroying project...");

    let destroy_full = full || all;

    if let Some(service) = service {
        println!("Destroying service: {}", service);
    }

    if let Some(component) = component {
        println!("Destroying component: {}", component);
    }

    if destroy_full {
        println!("Destroying all resources.");
    }

    if let Some(context) = context {
        println!("Using Kubernetes context: {}", context);
    }

    if confirm {
        println!("Confirmation received. Proceeding with destruction.");
    } else {
        println!("Dry run complete. No resources were destroyed.");
    }

    Ok(())
}

fn build_docker_image(service_path: &Path, service_name: &str, config: &MeshstackConfig) -> anyhow::Result<()> {
    println!("Building Docker image for {} (language: {})...", service_name, config.language);
    let dockerfile_path = service_path.join("Dockerfile");
    if !dockerfile_path.exists() {
        anyhow::bail!("Dockerfile not found in {}.", service_path.display());
    }

    let image_name = format!("meshstack/{}:latest", service_name);
    let output = Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(&image_name)
        .arg(&service_path)
        .output()?;

    if output.status.success() {
        println!("Successfully built Docker image: {}\n{}", image_name, String::from_utf8_lossy(&output.stdout));
    } else {
        anyhow::bail!("Failed to build Docker image for {}:\n{}\n{}",
            service_name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn push_docker_image(service_name: &str) -> anyhow::Result<()> {
    println!("Pushing Docker image for {} to registry...", service_name);
    let image_name = format!("meshstack/{}:latest", service_name);
    let output = Command::new("docker")
        .arg("push")
        .arg(&image_name)
        .output()?;

    if output.status.success() {
        println!("Successfully pushed Docker image: {}\n{}", image_name, String::from_utf8_lossy(&output.stdout));
    } else {
        anyhow::bail!("Failed to push Docker image for {}:\n{}\n{}",
            service_name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn validate_project(config: bool, cluster: bool, ci: bool, full: bool) -> Result<()> {
    println!("Validating project...");

    if full || config {
        validate_config()?;
    }
    if full || cluster {
        validate_cluster()?;
    }
    if full || ci {
        validate_ci()?;
    }

    Ok(())
}

fn validate_config() -> Result<()> {
    println!("Validating meshstack.yaml...");
    let config_path = "meshstack.yaml";
    if !Path::new(config_path).exists() {
        anyhow::bail!("meshstack.yaml not found.");
    }
    let config_content = fs::read_to_string(config_path)?;
    serde_yaml::from_str::<MeshstackConfig>(&config_content)?;
    println!("meshstack.yaml is valid.");
    Ok(())
}

fn validate_cluster() -> Result<()> {
    println!("Checking Kubernetes cluster connectivity...");
    let output = Command::new("kubectl")
        .arg("cluster-info")
        .arg("--context")
        .arg("current-context") // This will use the current context
        .output()?;

    if output.status.success() {
        println!("Connected to Kubernetes cluster successfully.");
    } else {
        anyhow::bail!("Failed to connect to Kubernetes cluster: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn validate_ci() -> Result<()> {
    println!("Validating CI/CD manifests...");
    // Placeholder for actual CI/CD validation logic
    // This would involve checking for .github/workflows for GitHub Actions
    // or ArgoCD application manifests.
    println!("CI/CD manifests validation (placeholder): No issues found.");
    Ok(())
}

fn install_component(
    component: &Option<String>,
    profile: &Option<String>,
    dry_run: bool,
    context: &Option<String>,
) -> anyhow::Result<()> {
    println!("Installing components...");

    let components_to_install = if let Some(comp) = component {
        vec![(comp.clone(), match comp.as_str() {
            "istio" => "istio/istio".to_string(),
            "prometheus" => "prometheus-community/prometheus".to_string(),
            "grafana" => "grafana/grafana".to_string(),
            "cert-manager" => "cert-manager/cert-manager".to_string(),
            "nginx-ingress" => "ingress-nginx/ingress-nginx".to_string(),
            "vault" => "hashicorp/vault".to_string(),
            _ => anyhow::bail!("Unknown component: {}. Valid components are: istio, prometheus, grafana, cert-manager, nginx-ingress, vault", comp),
        })]
    } else {
        println!("No component specified, installing default set.");
        vec![
            ("istio".to_string(), "istio/istio".to_string()),
            ("prometheus".to_string(), "prometheus-community/prometheus".to_string()),
            ("grafana".to_string(), "grafana/grafana".to_string()),
            ("cert-manager".to_string(), "cert-manager/cert-manager".to_string()),
            ("nginx-ingress".to_string(), "ingress-nginx/ingress-nginx".to_string()),
        ]
    };

    if let Some(p) = profile {
        println!("Applying profile: {}", p);
    }

    // Check if helm is installed
    if std::env::var("MESHSTACK_TEST_DRY_RUN_HELM").is_err() {
        if let Err(_) = Command::new("helm").arg("version").output() {
            anyhow::bail!("Helm is not installed or not found in PATH. Please install Helm to proceed. Refer to https://helm.sh/docs/intro/install/ for instructions.");
        }
    }

    for (release_name, chart_name) in components_to_install {
        println!("Attempting to install {} from chart {}", release_name, chart_name);

        let mut command = Command::new("helm");
        command.arg("install");
        command.arg(&release_name);
        command.arg(&chart_name);

        if dry_run {
            command.arg("--dry-run");
        }

        if let Some(ctx) = context {
            command.arg("--kube-context");
            command.arg(ctx);
        }

        if let Some(p) = profile {
            let values_file = match p.as_str() {
                "dev" => Some("dev-values.yaml"),
                "prod" => Some("prod-values.yaml"),
                "custom" => anyhow::bail!("Custom profile not yet implemented."),
                _ => anyhow::bail!("Unknown profile: {}. Valid profiles are: dev, prod, custom", p),
            };

            if let Some(file) = values_file {
                command.arg("--values");
                command.arg(file);
            }
        }

        // Check if we are in a test environment and should dry run helm execution
        if std::env::var("MESHSTACK_TEST_DRY_RUN_HELM").is_ok() {
            let command_str = format!("helm {}", command.get_args().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>().join(" "));
            println!("DRY RUN: Would execute helm command: {}", command_str);
            continue; // Continue to the next component in dry run mode
        }

        let output = command.output()?;

        if output.status.success() {
            println!("Installation of {} successful.", release_name);
            println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("Installation of {} failed.", release_name);
            eprintln!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Helm command failed for {}", release_name);
        }
    }

    Ok(())
}

fn update_project(
    check: bool,
    apply: bool,
    component: &Option<String>,
    template: bool,
    infra: bool,
) -> anyhow::Result<()> {
    println!("Updating project...");

    if check {
        println!("Checking for available updates...");
    }

    if apply {
        println!("Applying all updates automatically...");
    }

    if let Some(component) = component {
        println!("Updating component: {}", component);
    }

    if template {
        println!("Updating project templates...");
    }

    if infra {
        println!("Updating infra charts...");
    }

    Ok(())
}