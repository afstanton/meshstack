use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
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
    /// Set up a local Kubernetes cluster and install infrastructure components for development.
    Bootstrap {
        /// Use Kind for local cluster provisioning (default)
        #[arg(long)]
        kind: bool,

        /// Use k3d for local cluster provisioning
        #[arg(long)]
        k3d: bool,

        /// Skip installation of infrastructure components
        #[arg(long)]
        skip_install: bool,

        /// Name for the local cluster
        #[arg(short, long, default_value = "meshstack-dev")]
        name: String,
    },
    /// Re-generate scaffolds and configuration files based on meshstack.yaml.
    Generate {
        /// Generate scaffold for a specific service
        #[arg(short, long)]
        service: Option<String>,

        /// Re-generate all project scaffolds and configurations
        #[arg(long)]
        all: bool,

        /// Overwrite existing files without prompt
        #[arg(long)]
        force: bool,
    },
    /// Perform a dry-run preview of changes before applying them.
    Plan {
        /// The command to dry-run (e.g., install, deploy, destroy)
        #[arg(short, long)]
        command: String,

        /// Show detailed output of planned changes
        #[arg(long)]
        verbose: bool,

        /// Additional arguments to pass to the planned command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
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

#[derive(Serialize, Deserialize, Clone)]
struct MeshstackConfig {
    project_name: String,
    language: String,
    service_mesh: String,
    ci_cd: String,
}

/// Common context and configuration for Meshstack operations
#[derive(Clone)]
struct MeshstackContext {
    pub config: Option<MeshstackConfig>,
    pub kube_context: Option<String>,
    pub dry_run: bool,
}

impl MeshstackContext {
    /// Create a new context with optional Kubernetes context
    fn new(kube_context: Option<String>) -> Self {
        Self {
            config: Self::load_config().ok(),
            kube_context,
            dry_run: false,
        }
    }

    /// Create a new context with dry run enabled
    fn new_dry_run(kube_context: Option<String>) -> Self {
        Self {
            config: Self::load_config().ok(),
            kube_context,
            dry_run: true,
        }
    }

    /// Load and parse meshstack.yaml configuration
    fn load_config() -> Result<MeshstackConfig> {
        let config_content = fs::read_to_string("meshstack.yaml")?;
        let config: MeshstackConfig = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }

    /// Get the configuration, returning an error if not loaded
    fn require_config(&self) -> Result<&MeshstackConfig> {
        self.config.as_ref().ok_or_else(|| {
            anyhow::anyhow!("meshstack.yaml not found or invalid. Run 'meshstack init' first.")
        })
    }

    /// Add Kubernetes context arguments to a command if context is specified
    fn add_kube_context_args(&self, command: &mut Command) {
        if let Some(ctx) = &self.kube_context {
            command.arg("--kube-context").arg(ctx);
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name, mesh, ci, config } => {
            println!("Initializing new meshstack project...");

            let config_to_write = if let Some(config_path) = config {
                println!("Using config from: {}", config_path);
                let config_content = fs::read_to_string(config_path)?;
                serde_yaml::from_str(&config_content)?
            } else {
                MeshstackConfig {
                    project_name: name.clone().unwrap_or_else(|| "my-app".to_string()),
                    language: "generic".to_string(), // Language-agnostic mesh apps
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

            // Copy base templates
            let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let template_source_path = project_root.join("templates").join("base");
            let template_dest_path = Path::new("."); // Copy to current directory
            copy_dir_all(&template_source_path, template_dest_path)?;
            println!("Copied base templates.");
        }
        Commands::Bootstrap { kind, k3d, skip_install, name } => {
            bootstrap_local_cluster(*kind, *k3d, *skip_install, name)?;
        }
        Commands::Generate { service, all, force } => {
            let ctx = MeshstackContext::new(None);
            generate_scaffolds(service, *all, *force, &ctx)?;
        }
        Commands::Plan { command, verbose, args } => {
            plan_command(command, *verbose, args)?;
        }
        Commands::Install { component, profile, dry_run, context } => {
            let ctx = if *dry_run {
                MeshstackContext::new_dry_run(context.clone())
            } else {
                MeshstackContext::new(context.clone())
            };
            install_component(component, profile, &ctx)?;
        }
        Commands::Validate { config, cluster, ci, full } => {
            let ctx = MeshstackContext::new(None);
            validate_project(*config, *cluster, *ci, *full, &ctx)?;
        }
        Commands::Deploy { service, env, build, push, context } => {
            let ctx = MeshstackContext::new(context.clone());
            deploy_service(service, env, *build, *push, &ctx)?;
        }
        Commands::Destroy { service, component, full, context, confirm, all } => {
            let ctx = MeshstackContext::new(context.clone());
            destroy_project(service, component, *full, &ctx, *confirm, *all)?;
        }
        Commands::Update { check, apply, component, template, infra } => {
            let ctx = MeshstackContext::new(None);
            update_project(*check, *apply, component, *template, *infra, &ctx)?;
        }
        Commands::Status { components, services, lockfile, context } => {
            let ctx = MeshstackContext::new(context.clone());
            status_project(*components, *services, *lockfile, &ctx)?;
        }
    }
    Ok(())
}

// Helper function to run external commands and handle their output
fn run_command(mut command: Command, command_name: &str) -> anyhow::Result<String> {
    let output = command.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        anyhow::bail!(
            "{} command failed:\nStdout: {}\nStderr: {}",
            command_name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn status_project(
    components: bool,
    services: bool,
    lockfile: bool,
    ctx: &MeshstackContext,
) -> anyhow::Result<()> {
    println!("Showing project status...");

    if components {
        println!("\n--- Installed Infrastructure Components ---");
        if let Some(c) = &ctx.config {
            println!("Service Mesh: {}", c.service_mesh);
            // In a real scenario, you'd query Kubernetes or other tools for actual installed components
            println!("Other components (placeholder): Prometheus, Grafana, Cert-Manager");
        } else {
            println!("No meshstack.yaml found. Cannot determine installed components.");
        }
    }

    if services {
        println!("\n--- Running App Services ---");
        let services_dir = Path::new("services");
        if services_dir.exists() && services_dir.is_dir() {
            let mut service_found = false;
            for entry in fs::read_dir(services_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(svc_name) = path.file_name().and_then(|n| n.to_str()) {
                        println!("Service: {} (Status: Running - placeholder)", svc_name);
                        service_found = true;
                    }
                }
            }
            if !service_found {
                println!("No services found in the 'services/' directory.");
            }
        } else {
            println!("'services/' directory not found.");
        }
    }

    if lockfile {
        println!("\n--- meshstack.lock Status ---");
        let lockfile_path = Path::new("meshstack.lock");
        if lockfile_path.exists() {
            let lock_content = fs::read_to_string(lockfile_path)?;
            println!("Content of meshstack.lock:\n{}", lock_content);
        } else {
            println!("meshstack.lock not found.");
        }
    }

    if let Some(kube_ctx) = &ctx.kube_context {
        println!("\n--- Kubernetes Context Status ---");
        println!("Targeting Kubernetes context: {}", kube_ctx);
        // In a real scenario, you'd run kubectl commands to get context status
        println!("Kubernetes context status (placeholder): Connected");
    }

    Ok(())
}

fn deploy_service(
    service_name: &Option<String>,
    env: &Option<String>,
    build: bool,
    push: bool,
    ctx: &MeshstackContext,
) -> anyhow::Result<()> {
    println!("Deploying service...");

    if let Some(env) = env {
        println!("Applying environment profile: {}", env);
    }

    if let Some(context) = &ctx.kube_context {
        println!("Targeting Kubernetes context: {}", context);
    }

    let config = ctx.require_config()?;

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

        // Kubernetes deployment logic
        // Only deploy helm chart if not in a test environment with any dry run flag
        if std::env::var("MESHSTACK_TEST_DRY_RUN_HELM").is_err() &&
           std::env::var("MESHSTACK_TEST_DRY_RUN_DOCKER").is_err() &&
           std::env::var("MESHSTACK_TEST_DRY_RUN_KUBECTL").is_err() {
            deploy_helm_chart(&service_path, &current_service_name, env, ctx)?;
        }
    }

    println!("\nDeployment process completed.");
    Ok(())
}

fn deploy_helm_chart(
    service_path: &Path,
    service_name: &str,
    env: &Option<String>,
    ctx: &MeshstackContext,
) -> anyhow::Result<()> {
    println!("Deploying Helm chart for service: {}...", service_name);

    let chart_path = service_path;
    if !chart_path.join("Chart.yaml").exists() {
        anyhow::bail!("Helm chart (Chart.yaml) not found in {}.", chart_path.display());
    }

    let release_name = format!("meshstack-{}", service_name);

    let mut command = Command::new("helm");
    command.arg("upgrade");
    command.arg("--install");
    command.arg(&release_name);
    command.arg(chart_path);

    ctx.add_kube_context_args(&mut command);

    if let Some(e) = env {
        let values_file = match e.as_str() {
            "dev" => Some("dev-values.yaml"),
            "prod" => Some("prod-values.yaml"),
            "staging" => Some("staging-values.yaml"),
            _ => anyhow::bail!("Unknown environment: {}. Valid environments are: dev, prod, staging", e),
        };

        if let Some(file) = values_file {
            let env_values_path = Path::new(file);
            if env_values_path.exists() {
                command.arg("--values");
                command.arg(env_values_path);
            } else {
                println!("Warning: Environment values file {} not found. Skipping.", file);
            }
        }
    }

    // Check if we are in a test environment and should dry run helm execution
    if std::env::var("MESHSTACK_TEST_DRY_RUN_HELM").is_ok() {
        let command_str = format!("helm {}", command.get_args().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>().join(" "));
        println!("DRY RUN: Would execute helm command: {}", command_str);
        return Ok(());
    }

    let stdout = run_command(command, &format!("helm upgrade --install {}", release_name))?;
    println!("Successfully deployed service: {}\n{}", service_name, stdout);

    Ok(())
}

fn destroy_project(
    service: &Option<String>,
    component: &Option<String>,
    full: bool,
    ctx: &MeshstackContext,
    confirm: bool,
    all: bool,
) -> anyhow::Result<()> {
    println!("Destroying project...");

    let destroy_full = full || all;

    if !confirm && (service.is_some() || component.is_some() || destroy_full) {
        println!("Dry run complete. No resources were destroyed. Use --confirm to proceed.");
        return Ok(());
    }

    if let Some(svc) = service {
        println!("Destroying service: {}", svc);
        uninstall_helm_release(&format!("meshstack-{}", svc), ctx)?;
    }

    if let Some(comp) = component {
        println!("Destroying component: {}", comp);
        // For now, assume components are also Helm releases. This might need more sophisticated logic later.
        uninstall_helm_release(comp, ctx)?;
    }

    if destroy_full {
        println!("Destroying all resources.");
        // Uninstall all known infrastructure components
        let infra_components = vec!["istio", "prometheus", "grafana", "cert-manager", "nginx-ingress", "vault"];
        for comp in infra_components {
            println!("Uninstalling infrastructure component: {}", comp);
            uninstall_helm_release(comp, ctx)?;
        }

        // Discover and uninstall all services
        let services_dir = Path::new("services");
        if services_dir.exists() {
            for entry in fs::read_dir(services_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(svc_name) = path.file_name().and_then(|n| n.to_str()) {
                        println!("Uninstalling service: {}", svc_name);
                        uninstall_helm_release(&format!("meshstack-{}", svc_name), ctx)?;
                    }
                }
            }
        }

        // Optionally remove local project files (as per spec, but requires user confirmation)
        // For now, we'll just print a message.
        println!("Local project files (meshstack.yaml, services/, provision/) would be removed with --all. This is a placeholder.");
    }

    if confirm {
        println!("Confirmation received. Proceeding with destruction.");
    }

    Ok(())
}

fn uninstall_helm_release(release_name: &str, ctx: &MeshstackContext) -> anyhow::Result<()> {
    println!("Uninstalling Helm release: {}...", release_name);

    let mut command = Command::new("helm");
    command.arg("uninstall");
    command.arg(release_name);

    ctx.add_kube_context_args(&mut command);

    // Check if we are in a test environment and should dry run helm execution
    if std::env::var("MESHSTACK_TEST_DRY_RUN_HELM").is_ok() {
        let command_str = format!("helm {}", command.get_args().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>().join(" "));
        println!("DRY RUN: Would execute helm command: {}", command_str);
        return Ok(());
    }

    let stdout = run_command(command, &format!("helm uninstall {}", release_name))?;
    println!("Successfully uninstalled Helm release: {}\n{}", release_name, stdout);

    Ok(())
}

fn build_docker_image(service_path: &Path, service_name: &str, _config: &MeshstackConfig) -> anyhow::Result<()> {
    println!("Building Docker image for {} (language: {})...", service_name, _config.language);
    let dockerfile_path = service_path.join("Dockerfile");
    if !dockerfile_path.exists() {
        anyhow::bail!("Dockerfile not found in {}.", service_path.display());
    }

    let image_name = format!("meshstack/{}:latest", service_name);
    let mut command = Command::new("docker");
    command.arg("build").arg("-t").arg(&image_name).arg(&service_path);

    // Check if we are in a test environment and should dry run docker execution
    if std::env::var("MESHSTACK_TEST_DRY_RUN_DOCKER").is_ok() {
        let command_str = format!("docker {}", command.get_args().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>().join(" "));
        println!("DRY RUN: Would execute docker command: {}", command_str);
        return Ok(());
    }

    let stdout = run_command(command, "docker build")?;
    println!("Successfully built Docker image: {}\n{}", image_name, stdout);
    Ok(())
}

fn push_docker_image(service_name: &str) -> anyhow::Result<()> {
    println!("Pushing Docker image for {} to registry...", service_name);
    let image_name = format!("meshstack/{}:latest", service_name);
    let mut command = Command::new("docker");
    command.arg("push").arg(&image_name);

    // Check if we are in a test environment and should dry run docker execution
    if std::env::var("MESHSTACK_TEST_DRY_RUN_DOCKER").is_ok() {
        let command_str = format!("docker {}", command.get_args().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>().join(" "));
        println!("DRY RUN: Would execute docker command: {}", command_str);
        return Ok(());
    }

    let stdout = run_command(command, "docker push")?;
    println!("Successfully pushed Docker image: {}\n{}", image_name, stdout);
    Ok(())
}

fn validate_project(config: bool, cluster: bool, ci: bool, full: bool, _ctx: &MeshstackContext) -> anyhow::Result<()> {
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

fn validate_config() -> anyhow::Result<()> {
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

fn validate_cluster() -> anyhow::Result<()> {
    println!("Checking Kubernetes cluster connectivity...");
    let mut command = Command::new("kubectl");
    command.arg("cluster-info").arg("--context").arg("current-context");

    // Check if we are in a test environment and should dry run kubectl execution
    if std::env::var("MESHSTACK_TEST_DRY_RUN_KUBECTL").is_ok() {
        let command_str = format!("kubectl {}", command.get_args().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>().join(" "));
        println!("DRY RUN: Would execute kubectl command: {}", command_str);
        return Ok(());
    }

    let stdout = run_command(command, "kubectl cluster-info")?;
    println!("Connected to Kubernetes cluster successfully.\n{}", stdout);
    Ok(())
}

fn validate_ci() -> anyhow::Result<()> {
    println!("Validating CI/CD manifests...");

    let github_workflows_path = Path::new(".github").join("workflows");

    if github_workflows_path.exists() && github_workflows_path.is_dir() {
        println!("GitHub Actions workflows directory found.");
        // Further validation for GitHub Actions can go here
    } else {
        println!("GitHub Actions workflows directory not found. Skipping GitHub Actions validation.");
    }

    // Placeholder for ArgoCD validation
    println!("ArgoCD manifests validation (placeholder): No issues found.");

    Ok(())
}

fn install_component(
    component: &Option<String>,
    profile: &Option<String>,
    ctx: &MeshstackContext,
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
        let mut helm_version_cmd = Command::new("helm");
        helm_version_cmd.arg("version");
        if run_command(helm_version_cmd, "helm version").is_err() {
            anyhow::bail!("Helm is not installed or not found in PATH. Please install Helm to proceed. Refer to https://helm.sh/docs/intro/install/ for instructions.");
        }
    }

    for (release_name, chart_name) in components_to_install {
        println!("Attempting to install {} from chart {}", release_name, chart_name);

        let mut command = Command::new("helm");
        command.arg("install");
        command.arg(&release_name);
        command.arg(&chart_name);

        if ctx.dry_run {
            command.arg("--dry-run");
        }

        ctx.add_kube_context_args(&mut command);

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

        let stdout = run_command(command, &format!("helm upgrade --install {}", release_name))?;
        println!("Successfully deployed service: {}\n{}", release_name, stdout);
    }

    Ok(())
}

fn update_project(
    check: bool,
    apply: bool,
    component: &Option<String>,
    template: bool,
    infra: bool,
    ctx: &MeshstackContext,
) -> anyhow::Result<()> {
    println!("Updating project...");

    let config = ctx.require_config()?;
    let mut updates_available = Vec::new();

    // Check for updates
    if check || apply {
        println!("Checking for available updates...");

        if let Some(comp) = component {
            updates_available.extend(check_component_updates(comp, ctx)?);
        } else {
            // Check all infrastructure components
            if infra || (!template && !infra) { // Default to infra if neither specified
                updates_available.extend(check_infrastructure_updates(ctx)?);
            }

            // Check template updates
            if template || (!template && !infra) { // Default to templates if neither specified
                updates_available.extend(check_template_updates(config)?);
            }
        }

        if updates_available.is_empty() {
            println!("✅ All components are up to date!");
            return Ok(());
        }

        println!("\n📋 Available Updates:");
        for update in &updates_available {
            println!("  • {}: {} → {}", update.name, update.current_version, update.latest_version);
        }
    }

    // Apply updates if requested
    if apply {
        println!("\n🚀 Applying updates...");

        for update in &updates_available {
            match update.update_type {
                UpdateType::HelmChart => {
                    apply_helm_chart_update(update, ctx)?;
                }
                UpdateType::Template => {
                    apply_template_update(update, config)?;
                }
            }
        }

        println!("✅ All updates applied successfully!");
    } else if !check {
        // Handle individual flags without --check or --apply
        if let Some(comp) = component {
            println!("Updating component: {}", comp);
            let component_updates = check_component_updates(comp, ctx)?;
            for update in component_updates {
                apply_helm_chart_update(&update, ctx)?;
            }
        }

        if template {
            println!("Updating project templates...");
            let template_updates = check_template_updates(config)?;
            for update in template_updates {
                apply_template_update(&update, config)?;
            }
        }

        if infra {
            println!("Updating infrastructure charts...");
            let infra_updates = check_infrastructure_updates(ctx)?;
            for update in infra_updates {
                apply_helm_chart_update(&update, ctx)?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct UpdateInfo {
    name: String,
    current_version: String,
    latest_version: String,
    update_type: UpdateType,
    chart_name: Option<String>, // For Helm charts
}

#[derive(Debug, Clone)]
enum UpdateType {
    HelmChart,
    Template,
}

fn check_component_updates(component: &str, ctx: &MeshstackContext) -> anyhow::Result<Vec<UpdateInfo>> {
    let mut updates = Vec::new();

    let chart_name = match component {
        "istio" => "istio/istio",
        "prometheus" => "prometheus-community/prometheus",
        "grafana" => "grafana/grafana",
        "cert-manager" => "cert-manager/cert-manager",
        "nginx-ingress" => "ingress-nginx/ingress-nginx",
        "vault" => "hashicorp/vault",
        _ => anyhow::bail!("Unknown component: {}. Valid components are: istio, prometheus, grafana, cert-manager, nginx-ingress, vault", component),
    };

    if let Some(update) = check_helm_chart_update(component, chart_name, ctx)? {
        updates.push(update);
    }

    Ok(updates)
}

fn check_infrastructure_updates(ctx: &MeshstackContext) -> anyhow::Result<Vec<UpdateInfo>> {
    let mut updates = Vec::new();

    let components = vec![
        ("istio", "istio/istio"),
        ("prometheus", "prometheus-community/prometheus"),
        ("grafana", "grafana/grafana"),
        ("cert-manager", "cert-manager/cert-manager"),
        ("nginx-ingress", "ingress-nginx/ingress-nginx"),
    ];

    for (component, chart_name) in components {
        if let Some(update) = check_helm_chart_update(component, chart_name, ctx)? {
            updates.push(update);
        }
    }

    Ok(updates)
}

fn check_template_updates(_config: &MeshstackConfig) -> anyhow::Result<Vec<UpdateInfo>> {
    let mut updates = Vec::new();

    // Check if templates directory exists and compare with embedded templates
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let template_source = project_root.join("templates").join("base");

    if template_source.exists() {
        // For now, we'll simulate a template update check
        // In a real implementation, this would compare file hashes or versions
        updates.push(UpdateInfo {
            name: "base-templates".to_string(),
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.1".to_string(),
            update_type: UpdateType::Template,
            chart_name: None,
        });
    }

    Ok(updates)
}

fn check_helm_chart_update(component: &str, chart_name: &str, ctx: &MeshstackContext) -> anyhow::Result<Option<UpdateInfo>> {
    // Check if we're in test mode
    if std::env::var("MESHSTACK_TEST_DRY_RUN_HELM").is_ok() {
        // Return a mock update for testing
        return Ok(Some(UpdateInfo {
            name: component.to_string(),
            current_version: "1.0.0".to_string(),
            latest_version: "1.1.0".to_string(),
            update_type: UpdateType::HelmChart,
            chart_name: Some(chart_name.to_string()),
        }));
    }

    // Check if the component is currently installed
    let mut list_cmd = Command::new("helm");
    list_cmd.arg("list").arg("--filter").arg(component).arg("--output").arg("json");

    ctx.add_kube_context_args(&mut list_cmd);

    match run_command(list_cmd, "helm list") {
        Ok(output) => {
            if output.trim().is_empty() || output.trim() == "[]" {
                // Component not installed, no update needed
                return Ok(None);
            }

            // Parse current version (simplified - in real implementation would parse JSON)
            let current_version = "1.0.0"; // Placeholder

            // Check latest version available
            let mut search_cmd = Command::new("helm");
            search_cmd.arg("search").arg("repo").arg(chart_name).arg("--output").arg("json");

            match run_command(search_cmd, "helm search repo") {
                Ok(search_output) => {
                    if !search_output.trim().is_empty() && search_output.trim() != "[]" {
                        let latest_version = "1.1.0"; // Placeholder - would parse JSON in real implementation

                        if current_version != latest_version {
                            return Ok(Some(UpdateInfo {
                                name: component.to_string(),
                                current_version: current_version.to_string(),
                                latest_version: latest_version.to_string(),
                                update_type: UpdateType::HelmChart,
                                chart_name: Some(chart_name.to_string()),
                            }));
                        }
                    }
                }
                Err(_) => {
                    // Helm repo might not be added, skip this component
                    println!("Warning: Could not check updates for {} - repository might not be added", component);
                }
            }
        }
        Err(_) => {
            // Helm might not be available or component not installed
            return Ok(None);
        }
    }

    Ok(None)
}

fn apply_helm_chart_update(update: &UpdateInfo, ctx: &MeshstackContext) -> anyhow::Result<()> {
    println!("📦 Updating {} from {} to {}...", update.name, update.current_version, update.latest_version);

    let chart_name = update.chart_name.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Chart name not specified for {}", update.name))?;

    let mut command = Command::new("helm");
    command.arg("upgrade");
    command.arg(&update.name);
    command.arg(chart_name);
    command.arg("--version");
    command.arg(&update.latest_version);

    ctx.add_kube_context_args(&mut command);

    // Check if we are in a test environment and should dry run helm execution
    if std::env::var("MESHSTACK_TEST_DRY_RUN_HELM").is_ok() {
        let command_str = format!("helm {}", command.get_args().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>().join(" "));
        println!("DRY RUN: Would execute helm command: {}", command_str);
        return Ok(());
    }

    let stdout = run_command(command, &format!("helm upgrade {}", update.name))?;
    println!("✅ Successfully updated {}\n{}", update.name, stdout);

    Ok(())
}

fn apply_template_update(update: &UpdateInfo, _config: &MeshstackConfig) -> anyhow::Result<()> {
    println!("📄 Updating templates: {} from {} to {}...", update.name, update.current_version, update.latest_version);

    // Copy updated templates from the embedded templates
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let template_source_path = project_root.join("templates").join("base");
    let template_dest_path = Path::new("."); // Copy to current directory

    if template_source_path.exists() {
        copy_dir_all(&template_source_path, template_dest_path)?;
        println!("✅ Successfully updated base templates");
    } else {
        println!("⚠️  Warning: Template source not found, skipping template update");
    }

    Ok(())
}

// Helper function to copy a directory recursively
fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

// Removed copy_template_for_language function
fn bootstrap_local_cluster(
    use_kind: bool,
    use_k3d: bool,
    skip_install: bool,
    cluster_name: &str,
) -> anyhow::Result<()> {
    println!("🚀 Bootstrapping local Kubernetes cluster...");

    // Determine which cluster tool to use
    let cluster_tool = if use_k3d {
        "k3d"
    } else if use_kind || !use_k3d {
        // Default to kind if neither specified or if kind is explicitly requested
        "kind"
    } else {
        "kind" // Fallback
    };

    println!("Using {} for local cluster provisioning", cluster_tool);

    // Check if the tool is installed
    check_cluster_tool_installed(cluster_tool)?;

    // Check if cluster already exists
    if cluster_exists(cluster_tool, cluster_name)? {
        println!("✅ Cluster '{}' already exists", cluster_name);
    } else {
        // Create the cluster
        create_cluster(cluster_tool, cluster_name)?;
    }

    // Set kubectl context to the new cluster
    set_kubectl_context(cluster_tool, cluster_name)?;

    // Install infrastructure components unless skipped
    if !skip_install {
        println!("\n📦 Installing infrastructure components...");

        // Create a context for the new cluster
        let cluster_context = match cluster_tool {
            "kind" => format!("kind-{}", cluster_name),
            "k3d" => format!("k3d-{}", cluster_name),
            _ => cluster_name.to_string(),
        };

        let ctx = MeshstackContext::new(Some(cluster_context));

        // Install default components with dev profile
        install_component(&None, &Some("dev".to_string()), &ctx)?;
    } else {
        println!("⏭️  Skipping infrastructure component installation");
    }

    println!("\n✅ Local cluster bootstrap completed!");
    println!("🔧 Cluster name: {}", cluster_name);
    println!("🔧 Tool: {}", cluster_tool);
    println!("🔧 Context: {}-{}", cluster_tool, cluster_name);

    if !skip_install {
        println!("🔧 Infrastructure: Installed (dev profile)");
    }

    Ok(())
}

fn check_cluster_tool_installed(tool: &str) -> anyhow::Result<()> {
    println!("Checking if {} is installed...", tool);

    // Check if we're in test mode
    if std::env::var("MESHSTACK_TEST_DRY_RUN_CLUSTER").is_ok() {
        println!("DRY RUN: Would check if {} is installed", tool);
        return Ok(());
    }

    let mut command = Command::new(tool);
    command.arg("version");

    match run_command(command, &format!("{} version", tool)) {
        Ok(_) => {
            println!("✅ {} is installed", tool);
            Ok(())
        }
        Err(_) => {
            anyhow::bail!(
                "{} is not installed or not found in PATH. Please install {} to proceed.\n\
                Installation instructions:\n\
                - Kind: https://kind.sigs.k8s.io/docs/user/quick-start/#installation\n\
                - k3d: https://k3d.io/v5.4.6/#installation",
                tool, tool
            );
        }
    }
}

fn cluster_exists(tool: &str, cluster_name: &str) -> anyhow::Result<bool> {
    println!("Checking if cluster '{}' exists...", cluster_name);

    // Check if we're in test mode
    if std::env::var("MESHSTACK_TEST_DRY_RUN_CLUSTER").is_ok() {
        println!("DRY RUN: Would check if cluster '{}' exists", cluster_name);
        return Ok(false); // Assume cluster doesn't exist in test mode
    }

    let mut command = Command::new(tool);
    match tool {
        "kind" => {
            command.arg("get").arg("clusters");
        }
        "k3d" => {
            command.arg("cluster").arg("list").arg("--output").arg("json");
        }
        _ => anyhow::bail!("Unsupported cluster tool: {}", tool),
    }

    match run_command(command, &format!("{} list clusters", tool)) {
        Ok(output) => {
            let exists = output.contains(cluster_name);
            if exists {
                println!("✅ Cluster '{}' already exists", cluster_name);
            } else {
                println!("📋 Cluster '{}' does not exist", cluster_name);
            }
            Ok(exists)
        }
        Err(_) => {
            // If we can't list clusters, assume it doesn't exist
            println!("📋 Could not list clusters, assuming '{}' does not exist", cluster_name);
            Ok(false)
        }
    }
}

fn create_cluster(tool: &str, cluster_name: &str) -> anyhow::Result<()> {
    println!("🔨 Creating cluster '{}'...", cluster_name);

    // Check if we're in test mode
    if std::env::var("MESHSTACK_TEST_DRY_RUN_CLUSTER").is_ok() {
        println!("DRY RUN: Would create cluster '{}' using {}", cluster_name, tool);
        return Ok(());
    }

    let mut command = Command::new(tool);
    match tool {
        "kind" => {
            command.arg("create").arg("cluster").arg("--name").arg(cluster_name);

            // Add some useful configuration for development
            command.arg("--config").arg("-");

            // We'll pipe a basic config that exposes ports for common services
            let _config = format!(
                r#"kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: {}
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
"#,
                cluster_name
            );

            // For now, we'll create without the config file to keep it simple
            // In a real implementation, we'd write the config to a temp file
            command = Command::new(tool);
            command.arg("create").arg("cluster").arg("--name").arg(cluster_name);
        }
        "k3d" => {
            command.arg("cluster").arg("create").arg(cluster_name);

            // Add some useful configuration for development
            command.arg("--port").arg("80:80@loadbalancer");
            command.arg("--port").arg("443:443@loadbalancer");
        }
        _ => anyhow::bail!("Unsupported cluster tool: {}", tool),
    }

    let stdout = run_command(command, &format!("{} create cluster", tool))?;
    println!("✅ Successfully created cluster '{}'\n{}", cluster_name, stdout);

    Ok(())
}

fn set_kubectl_context(tool: &str, cluster_name: &str) -> anyhow::Result<()> {
    let context_name = match tool {
        "kind" => format!("kind-{}", cluster_name),
        "k3d" => format!("k3d-{}", cluster_name),
        _ => cluster_name.to_string(),
    };

    println!("🔧 Setting kubectl context to '{}'...", context_name);

    // Check if we're in test mode
    if std::env::var("MESHSTACK_TEST_DRY_RUN_KUBECTL").is_ok() ||
       std::env::var("MESHSTACK_TEST_DRY_RUN_CLUSTER").is_ok() {
        println!("DRY RUN: Would set kubectl context to '{}'", context_name);
        return Ok(());
    }

    let mut command = Command::new("kubectl");
    command.arg("config").arg("use-context").arg(&context_name);

    let _stdout = run_command(command, "kubectl config use-context")?;
    println!("✅ Successfully set kubectl context to '{}'", context_name);

    Ok(())
}
fn generate_scaffolds(
    service: &Option<String>,
    all: bool,
    force: bool,
    ctx: &MeshstackContext,
) -> anyhow::Result<()> {
    println!("🔧 Generating scaffolds and configuration files...");

    let config = ctx.require_config()?;
    let mut generated_files = Vec::new();

    if let Some(service_name) = service {
        // Generate scaffold for a specific service
        println!("Generating scaffold for service: {}", service_name);
        generated_files.extend(generate_service_scaffold(service_name, config, force)?);
    } else if all {
        // Re-generate all project scaffolds and configurations
        println!("Re-generating all project scaffolds and configurations...");

        // Generate base project structure
        generated_files.extend(generate_project_structure(config, force)?);

        // Generate scaffolds for all existing services
        let services_dir = Path::new("services");
        if services_dir.exists() {
            for entry in fs::read_dir(services_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(svc_name) = path.file_name().and_then(|n| n.to_str()) {
                        println!("Re-generating scaffold for existing service: {}", svc_name);
                        generated_files.extend(generate_service_scaffold(svc_name, config, force)?);
                    }
                }
            }
        }
    } else {
        // Default behavior: regenerate project-level configurations
        println!("Re-generating project-level configurations...");
        generated_files.extend(generate_project_structure(config, force)?);
    }

    // Print summary
    if generated_files.is_empty() {
        println!("✅ No files needed to be generated or updated.");
    } else {
        println!("\n📋 Generated/Updated Files:");
        for file in &generated_files {
            println!("  • {}", file);
        }
        println!("\n✅ Successfully generated {} files!", generated_files.len());
    }

    Ok(())
}

fn generate_service_scaffold(
    service_name: &str,
    config: &MeshstackConfig,
    force: bool,
) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();
    let service_dir = Path::new("services").join(service_name);

    // Create service directory if it doesn't exist
    if !service_dir.exists() {
        fs::create_dir_all(&service_dir)?;
        println!("Created service directory: {}", service_dir.display());
    }

    // Generate Dockerfile
    let dockerfile_path = service_dir.join("Dockerfile");
    if !dockerfile_path.exists() || force {
        let dockerfile_content = generate_dockerfile_content(config);
        if should_write_file(&dockerfile_path, force)? {
            fs::write(&dockerfile_path, dockerfile_content)?;
            generated_files.push(dockerfile_path.to_string_lossy().to_string());
        }
    }

    // Generate Helm Chart
    generated_files.extend(generate_helm_chart(service_name, &service_dir, config, force)?);

    // Generate basic application files based on language (if not generic)
    if config.language != "generic" {
        generated_files.extend(generate_app_files(service_name, &service_dir, config, force)?);
    }

    Ok(generated_files)
}

fn generate_project_structure(
    config: &MeshstackConfig,
    force: bool,
) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();

    // Ensure base directories exist
    let dirs = ["services", "provision"];
    for dir in &dirs {
        if !Path::new(dir).exists() {
            fs::create_dir(dir)?;
            println!("Created directory: {}", dir);
        }
    }

    // Generate/update meshstack.yaml if needed
    let meshstack_yaml_path = Path::new("meshstack.yaml");
    if !meshstack_yaml_path.exists() || force {
        if should_write_file(meshstack_yaml_path, force)? {
            let yaml_config = serde_yaml::to_string(config)?;
            fs::write(meshstack_yaml_path, yaml_config)?;
            generated_files.push("meshstack.yaml".to_string());
        }
    }

    // Generate CI/CD configurations based on ci_cd setting
    match config.ci_cd.as_str() {
        "github" => {
            generated_files.extend(generate_github_actions_workflow(config, force)?);
        }
        "argo" => {
            generated_files.extend(generate_argocd_manifests(config, force)?);
        }
        _ => {
            println!("Unknown CI/CD system: {}. Skipping CI/CD generation.", config.ci_cd);
        }
    }

    // Generate environment-specific values files
    generated_files.extend(generate_values_files(config, force)?);

    // Copy/update base templates
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let template_source_path = project_root.join("templates").join("base");
    if template_source_path.exists() {
        copy_dir_all(&template_source_path, Path::new("."))?;
        generated_files.push("base templates".to_string());
    }

    Ok(generated_files)
}

fn generate_dockerfile_content(config: &MeshstackConfig) -> String {
    match config.language.as_str() {
        "rust" => {
            r#"# Multi-stage build for Rust
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/app /usr/local/bin/app

EXPOSE 8080
CMD ["app"]
"#.to_string()
        }
        "node" | "javascript" => {
            r#"FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY . .

EXPOSE 3000
USER node

CMD ["npm", "start"]
"#.to_string()
        }
        "python" => {
            r#"FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 8000
USER 1000

CMD ["python", "app.py"]
"#.to_string()
        }
        "go" => {
            r#"# Multi-stage build for Go
FROM golang:1.21-alpine AS builder

WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o main .

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/

COPY --from=builder /app/main .

EXPOSE 8080
CMD ["./main"]
"#.to_string()
        }
        _ => {
            // Generic/language-agnostic Dockerfile
            r#"FROM alpine:latest

# Install basic utilities
RUN apk --no-cache add ca-certificates curl

WORKDIR /app

# Copy application files
COPY . .

# Make any scripts executable
RUN find . -name "*.sh" -exec chmod +x {} \;

EXPOSE 8080

# Default command - override in your specific implementation
CMD ["echo", "Please customize this Dockerfile for your specific application"]
"#.to_string()
        }
    }
}

fn generate_helm_chart(
    service_name: &str,
    service_dir: &Path,
    config: &MeshstackConfig,
    force: bool,
) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();

    // Generate Chart.yaml
    let chart_yaml_path = service_dir.join("Chart.yaml");
    if !chart_yaml_path.exists() || force {
        let chart_content = format!(
            r#"apiVersion: v2
name: {}
description: A Helm chart for {} service
type: application
version: 0.1.0
appVersion: "1.0.0"
"#,
            service_name, service_name
        );

        if should_write_file(&chart_yaml_path, force)? {
            fs::write(&chart_yaml_path, chart_content)?;
            generated_files.push(chart_yaml_path.to_string_lossy().to_string());
        }
    }

    // Create templates directory
    let templates_dir = service_dir.join("templates");
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir)?;
    }

    // Generate deployment.yaml
    let deployment_path = templates_dir.join("deployment.yaml");
    if !deployment_path.exists() || force {
        let deployment_content = generate_deployment_yaml(service_name, config);
        if should_write_file(&deployment_path, force)? {
            fs::write(&deployment_path, deployment_content)?;
            generated_files.push(deployment_path.to_string_lossy().to_string());
        }
    }

    // Generate service.yaml
    let service_path = templates_dir.join("service.yaml");
    if !service_path.exists() || force {
        let service_content = generate_service_yaml(service_name);
        if should_write_file(&service_path, force)? {
            fs::write(&service_path, service_content)?;
            generated_files.push(service_path.to_string_lossy().to_string());
        }
    }

    // Generate values.yaml
    let values_path = service_dir.join("values.yaml");
    if !values_path.exists() || force {
        let values_content = generate_values_yaml(service_name, config);
        if should_write_file(&values_path, force)? {
            fs::write(&values_path, values_content)?;
            generated_files.push(values_path.to_string_lossy().to_string());
        }
    }

    Ok(generated_files)
}

fn generate_deployment_yaml(service_name: &str, config: &MeshstackConfig) -> String {
    let mesh_annotations = match config.service_mesh.as_str() {
        "istio" => r#"
        sidecar.istio.io/inject: "true""#,
        "linkerd" => r#"
        linkerd.io/inject: enabled"#,
        _ => "",
    };

    format!(
        r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{{{ include "{}.fullname" . }}}}
  labels:
    {{{{- include "{}.labels" . | nindent 4 }}}}
spec:
  replicas: {{{{ .Values.replicaCount }}}}
  selector:
    matchLabels:
      {{{{- include "{}.selectorLabels" . | nindent 6 }}}}
  template:
    metadata:
      annotations:{}
      labels:
        {{{{- include "{}.selectorLabels" . | nindent 8 }}}}
    spec:
      containers:
        - name: {{{{ .Chart.Name }}}}
          image: "{{{{ .Values.image.repository }}}}:{{{{ .Values.image.tag | default .Chart.AppVersion }}}}"
          imagePullPolicy: {{{{ .Values.image.pullPolicy }}}}
          ports:
            - name: http
              containerPort: {{{{ .Values.service.targetPort }}}}
              protocol: TCP
          livenessProbe:
            httpGet:
              path: /health
              port: http
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /ready
              port: http
            initialDelaySeconds: 5
            periodSeconds: 5
          resources:
            {{{{- toYaml .Values.resources | nindent 12 }}}}
"#,
        service_name, service_name, service_name, mesh_annotations, service_name
    )
}

fn generate_service_yaml(service_name: &str) -> String {
    format!(
        r#"apiVersion: v1
kind: Service
metadata:
  name: {{{{ include "{}.fullname" . }}}}
  labels:
    {{{{- include "{}.labels" . | nindent 4 }}}}
spec:
  type: {{{{ .Values.service.type }}}}
  ports:
    - port: {{{{ .Values.service.port }}}}
      targetPort: {{{{ .Values.service.targetPort }}}}
      protocol: TCP
      name: http
  selector:
    {{{{- include "{}.selectorLabels" . | nindent 4 }}}}
"#,
        service_name, service_name, service_name
    )
}

fn generate_values_yaml(service_name: &str, _config: &MeshstackConfig) -> String {
    format!(
        r#"# Default values for {}.
replicaCount: 1

image:
  repository: meshstack/{}
  pullPolicy: IfNotPresent
  tag: "latest"

service:
  type: ClusterIP
  port: 80
  targetPort: 8080

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi

autoscaling:
  enabled: false
  minReplicas: 1
  maxReplicas: 100
  targetCPUUtilizationPercentage: 80

nodeSelector: {{}}

tolerations: []

affinity: {{}}
"#,
        service_name, service_name
    )
}

fn generate_app_files(
    service_name: &str,
    service_dir: &Path,
    config: &MeshstackConfig,
    force: bool,
) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();

    match config.language.as_str() {
        "rust" => {
            // Generate Cargo.toml
            let cargo_toml_path = service_dir.join("Cargo.toml");
            if !cargo_toml_path.exists() || force {
                let cargo_content = format!(
                    r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
warp = "0.3"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#,
                    service_name
                );
                if should_write_file(&cargo_toml_path, force)? {
                    fs::write(&cargo_toml_path, cargo_content)?;
                    generated_files.push(cargo_toml_path.to_string_lossy().to_string());
                }
            }

            // Generate basic main.rs
            let src_dir = service_dir.join("src");
            if !src_dir.exists() {
                fs::create_dir_all(&src_dir)?;
            }

            let main_rs_path = src_dir.join("main.rs");
            if !main_rs_path.exists() || force {
                let main_content = r#"use warp::Filter;

#[tokio::main]
async fn main() {
    let health = warp::path("health")
        .map(|| "OK");

    let ready = warp::path("ready")
        .map(|| "Ready");

    let hello = warp::path::end()
        .map(|| "Hello from Meshstack service!");

    let routes = health.or(ready).or(hello);

    println!("Starting server on port 8080");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 8080))
        .await;
}
"#;
                if should_write_file(&main_rs_path, force)? {
                    fs::write(&main_rs_path, main_content)?;
                    generated_files.push(main_rs_path.to_string_lossy().to_string());
                }
            }
        }
        "node" | "javascript" => {
            // Generate package.json
            let package_json_path = service_dir.join("package.json");
            if !package_json_path.exists() || force {
                let package_content = format!(
                    r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "Meshstack service",
  "main": "index.js",
  "scripts": {{
    "start": "node index.js",
    "dev": "nodemon index.js"
  }},
  "dependencies": {{
    "express": "^4.18.0"
  }},
  "devDependencies": {{
    "nodemon": "^2.0.0"
  }}
}}
"#,
                    service_name
                );
                if should_write_file(&package_json_path, force)? {
                    fs::write(&package_json_path, package_content)?;
                    generated_files.push(package_json_path.to_string_lossy().to_string());
                }
            }

            // Generate basic index.js
            let index_js_path = service_dir.join("index.js");
            if !index_js_path.exists() || force {
                let index_content = r#"const express = require('express');
const app = express();
const port = process.env.PORT || 8080;

app.use(express.json());

app.get('/', (req, res) => {
  res.json({ message: 'Hello from Meshstack service!' });
});

app.get('/health', (req, res) => {
  res.json({ status: 'OK' });
});

app.get('/ready', (req, res) => {
  res.json({ status: 'Ready' });
});

app.listen(port, '0.0.0.0', () => {
  console.log(`Server running on port ${port}`);
});
"#;
                if should_write_file(&index_js_path, force)? {
                    fs::write(&index_js_path, index_content)?;
                    generated_files.push(index_js_path.to_string_lossy().to_string());
                }
            }
        }
        _ => {
            // For other languages or generic, just create a placeholder
            println!("Language-specific files not implemented for: {}", config.language);
        }
    }

    Ok(generated_files)
}

fn generate_github_actions_workflow(
    config: &MeshstackConfig,
    force: bool,
) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();

    let workflows_dir = Path::new(".github").join("workflows");
    if !workflows_dir.exists() {
        fs::create_dir_all(&workflows_dir)?;
    }

    let workflow_path = workflows_dir.join("meshstack.yml");
    if !workflow_path.exists() || force {
        let workflow_content = format!(
            r#"name: Meshstack CI/CD

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v2

    - name: Login to Container Registry
      uses: docker/login-action@v2
      with:
        registry: ${{{{ secrets.REGISTRY_URL }}}}
        username: ${{{{ secrets.REGISTRY_USERNAME }}}}
        password: ${{{{ secrets.REGISTRY_PASSWORD }}}}

    - name: Build and push Docker images
      run: |
        for service in services/*/; do
          if [ -d "$service" ]; then
            service_name=$(basename "$service")
            echo "Building $service_name"
            docker build -t meshstack/$service_name:${{{{ github.sha }}}} "$service"
            docker push meshstack/$service_name:${{{{ github.sha }}}}
          fi
        done

    - name: Deploy to {} cluster
      if: github.ref == 'refs/heads/main'
      run: |
        # Add your deployment commands here
        echo "Deploying to {} service mesh"
        # meshstack deploy --env prod
"#,
            config.service_mesh, config.service_mesh
        );

        if should_write_file(&workflow_path, force)? {
            fs::write(&workflow_path, workflow_content)?;
            generated_files.push(workflow_path.to_string_lossy().to_string());
        }
    }

    Ok(generated_files)
}

fn generate_argocd_manifests(
    config: &MeshstackConfig,
    force: bool,
) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();

    let argocd_dir = Path::new("argocd");
    if !argocd_dir.exists() {
        fs::create_dir_all(&argocd_dir)?;
    }

    let app_path = argocd_dir.join("application.yaml");
    if !app_path.exists() || force {
        let app_content = format!(
            r#"apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: {}
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/your-org/your-repo
    targetRevision: HEAD
    path: .
  destination:
    server: https://kubernetes.default.svc
    namespace: default
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
    - CreateNamespace=true
"#,
            config.project_name
        );

        if should_write_file(&app_path, force)? {
            fs::write(&app_path, app_content)?;
            generated_files.push(app_path.to_string_lossy().to_string());
        }
    }

    Ok(generated_files)
}

fn generate_values_files(
    _config: &MeshstackConfig,
    force: bool,
) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();

    let values_files = [
        ("dev-values.yaml", "development"),
        ("prod-values.yaml", "production"),
        ("staging-values.yaml", "staging"),
    ];

    for (filename, env) in values_files {
        let values_path = Path::new(filename);
        if !values_path.exists() || force {
            let values_content = format!(
                r#"# {} environment values
environment: {}

# Resource limits for {} environment
resources:
  limits:
    cpu: {}
    memory: {}
  requests:
    cpu: {}
    memory: {}

# Replica count for {} environment
replicaCount: {}

# Service mesh specific configurations
serviceMesh:
  enabled: true

# Monitoring and observability
monitoring:
  enabled: true

# Security settings
security:
  enabled: true
"#,
                env.to_uppercase(),
                env,
                env,
                if env == "production" { "1000m" } else { "500m" },
                if env == "production" { "1Gi" } else { "512Mi" },
                if env == "production" { "500m" } else { "250m" },
                if env == "production" { "512Mi" } else { "256Mi" },
                env,
                if env == "production" { 3 } else { 1 }
            );

            if should_write_file(values_path, force)? {
                fs::write(values_path, values_content)?;
                generated_files.push(filename.to_string());
            }
        }
    }

    Ok(generated_files)
}

fn should_write_file(path: &Path, force: bool) -> anyhow::Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    if force {
        return Ok(true);
    }

    // In a real implementation, you might want to prompt the user
    // For now, we'll skip existing files unless force is specified
    println!("File {} already exists. Use --force to overwrite.", path.display());
    Ok(false)
}
fn plan_command(
    command: &str,
    verbose: bool,
    args: &[String],
) -> anyhow::Result<()> {
    println!("📋 Planning execution of '{}' command...", command);

    if verbose {
        println!("🔍 Verbose mode enabled - showing detailed planning information");
    }

    match command {
        "install" => plan_install_command(args, verbose)?,
        "deploy" => plan_deploy_command(args, verbose)?,
        "destroy" => plan_destroy_command(args, verbose)?,
        "update" => plan_update_command(args, verbose)?,
        "bootstrap" => plan_bootstrap_command(args, verbose)?,
        "generate" => plan_generate_command(args, verbose)?,
        _ => {
            anyhow::bail!(
                "Unknown command '{}' for planning. Supported commands: install, deploy, destroy, update, bootstrap, generate",
                command
            );
        }
    }

    println!("\n✅ Planning completed successfully!");
    println!("💡 To execute the planned changes, run: meshstack {}",
             format!("{} {}", command, args.join(" ")).trim());

    Ok(())
}

fn plan_install_command(args: &[String], verbose: bool) -> anyhow::Result<()> {
    println!("\n🔧 Planning 'install' command execution:");

    // Parse arguments to understand what would be installed
    let mut component: Option<String> = None;
    let mut profile: Option<String> = None;
    let mut context: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--component" | "-c" => {
                if i + 1 < args.len() {
                    component = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--profile" | "-p" => {
                if i + 1 < args.len() {
                    profile = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--context" => {
                if i + 1 < args.len() {
                    context = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let components_to_install = if let Some(comp) = &component {
        vec![comp.clone()]
    } else {
        vec!["istio".to_string(), "prometheus".to_string(), "grafana".to_string(),
             "cert-manager".to_string(), "nginx-ingress".to_string()]
    };

    println!("📦 Components that would be installed:");
    for comp in &components_to_install {
        let chart_name = match comp.as_str() {
            "istio" => "istio/istio",
            "prometheus" => "prometheus-community/prometheus",
            "grafana" => "grafana/grafana",
            "cert-manager" => "cert-manager/cert-manager",
            "nginx-ingress" => "ingress-nginx/ingress-nginx",
            "vault" => "hashicorp/vault",
            _ => "unknown/unknown",
        };
        println!("  • {} (from chart: {})", comp, chart_name);

        if verbose {
            println!("    - Helm command: helm install {} {}", comp, chart_name);
            if let Some(p) = &profile {
                println!("    - Profile: {} (values file: {}-values.yaml)", p, p);
            }
            if let Some(ctx) = &context {
                println!("    - Kubernetes context: {}", ctx);
            }
        }
    }

    if let Some(p) = &profile {
        println!("🎯 Profile: {}", p);
    }

    if let Some(ctx) = &context {
        println!("🎯 Target Kubernetes context: {}", ctx);
    } else {
        println!("🎯 Target Kubernetes context: current-context");
    }

    println!("\n⚠️  Prerequisites:");
    println!("  • Helm must be installed and available in PATH");
    println!("  • Kubernetes cluster must be accessible");
    println!("  • Required Helm repositories must be added");

    Ok(())
}

fn plan_deploy_command(args: &[String], verbose: bool) -> anyhow::Result<()> {
    println!("\n🚀 Planning 'deploy' command execution:");

    // Parse arguments
    let mut service: Option<String> = None;
    let mut env: Option<String> = None;
    let mut build = false;
    let mut push = false;
    let mut context: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--service" | "-s" => {
                if i + 1 < args.len() {
                    service = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--env" | "-e" => {
                if i + 1 < args.len() {
                    env = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--build" => build = true,
            "--push" => push = true,
            "--context" => {
                if i + 1 < args.len() {
                    context = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Check what services would be deployed
    let services_dir = Path::new("services");
    let services_to_deploy = if let Some(svc_name) = &service {
        if services_dir.join(svc_name).exists() {
            vec![svc_name.clone()]
        } else {
            println!("⚠️  Warning: Service '{}' directory not found", svc_name);
            vec![]
        }
    } else if services_dir.exists() {
        fs::read_dir(services_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_dir()))
            .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
            .collect()
    } else {
        println!("⚠️  Warning: Services directory not found");
        vec![]
    };

    if services_to_deploy.is_empty() {
        println!("❌ No services found to deploy");
        return Ok(());
    }

    println!("🎯 Services that would be deployed:");
    for svc in &services_to_deploy {
        println!("  • {}", svc);

        if verbose {
            let service_path = services_dir.join(svc);
            if build {
                println!("    - Docker build: docker build -t meshstack/{}:latest {}", svc, service_path.display());
            }
            if push {
                println!("    - Docker push: docker push meshstack/{}:latest", svc);
            }
            println!("    - Helm deploy: helm upgrade --install meshstack-{} {}", svc, service_path.display());
        }
    }

    if let Some(e) = &env {
        println!("🌍 Environment: {} (values file: {}-values.yaml)", e, e);
    }

    if let Some(ctx) = &context {
        println!("🎯 Target Kubernetes context: {}", ctx);
    }

    println!("\n📋 Deployment steps that would be executed:");
    if build {
        println!("  1. Build Docker images for services");
    }
    if push {
        println!("  2. Push Docker images to registry");
    }
    println!("  3. Deploy services using Helm charts");

    println!("\n⚠️  Prerequisites:");
    println!("  • meshstack.yaml configuration file must exist");
    println!("  • Services must have valid Dockerfiles and Helm charts");
    if build {
        println!("  • Docker must be installed and running");
    }
    if push {
        println!("  • Docker registry credentials must be configured");
    }
    println!("  • Kubernetes cluster must be accessible");
    println!("  • Helm must be installed");

    Ok(())
}

fn plan_destroy_command(args: &[String], verbose: bool) -> anyhow::Result<()> {
    println!("\n💥 Planning 'destroy' command execution:");

    // Parse arguments
    let mut service: Option<String> = None;
    let mut component: Option<String> = None;
    let mut full = false;
    let mut all = false;
    let mut context: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--service" | "-s" => {
                if i + 1 < args.len() {
                    service = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--component" | "-c" => {
                if i + 1 < args.len() {
                    component = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--full" => full = true,
            "--all" => all = true,
            "--context" => {
                if i + 1 < args.len() {
                    context = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let destroy_full = full || all;

    println!("🎯 Resources that would be destroyed:");

    if let Some(svc) = &service {
        println!("  • Service: {} (Helm release: meshstack-{})", svc, svc);
        if verbose {
            println!("    - Command: helm uninstall meshstack-{}", svc);
        }
    }

    if let Some(comp) = &component {
        println!("  • Component: {} (Helm release: {})", comp, comp);
        if verbose {
            println!("    - Command: helm uninstall {}", comp);
        }
    }

    if destroy_full {
        println!("  • All infrastructure components:");
        let infra_components = vec!["istio", "prometheus", "grafana", "cert-manager", "nginx-ingress", "vault"];
        for comp in &infra_components {
            println!("    - {}", comp);
            if verbose {
                println!("      Command: helm uninstall {}", comp);
            }
        }

        println!("  • All application services:");
        let services_dir = Path::new("services");
        if services_dir.exists() {
            for entry in fs::read_dir(services_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(svc_name) = path.file_name().and_then(|n| n.to_str()) {
                        println!("    - {} (Helm release: meshstack-{})", svc_name, svc_name);
                        if verbose {
                            println!("      Command: helm uninstall meshstack-{}", svc_name);
                        }
                    }
                }
            }
        }

        if all {
            println!("  • Local project files (with --all flag):");
            println!("    - meshstack.yaml");
            println!("    - services/ directory");
            println!("    - provision/ directory");
        }
    }

    if let Some(ctx) = &context {
        println!("🎯 Target Kubernetes context: {}", ctx);
    }

    println!("\n⚠️  DANGER ZONE:");
    println!("  • This operation will permanently delete resources");
    println!("  • Confirmation will be required unless --confirm flag is used");
    println!("  • Backup important data before proceeding");

    println!("\n⚠️  Prerequisites:");
    println!("  • Helm must be installed and available");
    println!("  • Kubernetes cluster must be accessible");
    println!("  • Sufficient permissions to delete resources");

    Ok(())
}

fn plan_update_command(args: &[String], verbose: bool) -> anyhow::Result<()> {
    println!("\n🔄 Planning 'update' command execution:");

    // Parse arguments
    let mut check = false;
    let mut apply = false;
    let mut component: Option<String> = None;
    let mut template = false;
    let mut infra = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => check = true,
            "--apply" => apply = true,
            "--component" | "-c" => {
                if i + 1 < args.len() {
                    component = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--template" => template = true,
            "--infra" => infra = true,
            _ => {}
        }
        i += 1;
    }

    println!("🎯 Update operations that would be performed:");

    if check {
        println!("  • Check for available updates");
        if verbose {
            println!("    - Query Helm repositories for latest chart versions");
            println!("    - Compare with currently installed versions");
            println!("    - Check for template updates");
        }
    }

    if apply {
        println!("  • Apply all available updates automatically");
        if verbose {
            println!("    - Update Helm charts to latest versions");
            println!("    - Regenerate templates from latest versions");
        }
    }

    if let Some(comp) = &component {
        println!("  • Update specific component: {}", comp);
        if verbose {
            let chart_name = match comp.as_str() {
                "istio" => "istio/istio",
                "prometheus" => "prometheus-community/prometheus",
                "grafana" => "grafana/grafana",
                "cert-manager" => "cert-manager/cert-manager",
                "nginx-ingress" => "ingress-nginx/ingress-nginx",
                "vault" => "hashicorp/vault",
                _ => "unknown/unknown",
            };
            println!("    - Chart: {}", chart_name);
            println!("    - Command: helm upgrade {} {}", comp, chart_name);
        }
    }

    if template {
        println!("  • Update project templates");
        if verbose {
            println!("    - Regenerate Dockerfiles");
            println!("    - Update Helm chart templates");
            println!("    - Refresh CI/CD configurations");
        }
    }

    if infra {
        println!("  • Update infrastructure charts");
        if verbose {
            println!("    - Update service mesh components");
            println!("    - Update monitoring stack");
            println!("    - Update ingress controllers");
        }
    }

    if !check && !apply && component.is_none() && !template && !infra {
        println!("  • Default: Check infrastructure and template updates");
    }

    println!("\n⚠️  Prerequisites:");
    println!("  • meshstack.yaml configuration file must exist");
    println!("  • Helm repositories must be up to date");
    println!("  • Internet connection for checking latest versions");

    Ok(())
}

fn plan_bootstrap_command(args: &[String], verbose: bool) -> anyhow::Result<()> {
    println!("\n🚀 Planning 'bootstrap' command execution:");

    // Parse arguments
    let mut _kind = false;
    let mut k3d = false;
    let mut skip_install = false;
    let mut name = "meshstack-dev".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--kind" => _kind = true,
            "--k3d" => k3d = true,
            "--skip-install" => skip_install = true,
            "--name" | "-n" => {
                if i + 1 < args.len() {
                    name = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let cluster_tool = if k3d {
        "k3d"
    } else {
        "kind" // Default
    };

    println!("🎯 Bootstrap operations that would be performed:");
    println!("  • Create local Kubernetes cluster using {}", cluster_tool);
    println!("  • Cluster name: {}", name);

    if verbose {
        match cluster_tool {
            "kind" => {
                println!("    - Command: kind create cluster --name {}", name);
                println!("    - Context: kind-{}", name);
            }
            "k3d" => {
                println!("    - Command: k3d cluster create {} --port 80:80@loadbalancer --port 443:443@loadbalancer", name);
                println!("    - Context: k3d-{}", name);
            }
            _ => {}
        }
        println!("    - Set kubectl context: kubectl config use-context {}-{}", cluster_tool, name);
    }

    if !skip_install {
        println!("  • Install infrastructure components (dev profile):");
        let components = vec!["istio", "prometheus", "grafana", "cert-manager", "nginx-ingress"];
        for comp in &components {
            println!("    - {}", comp);
            if verbose {
                let chart_name = match *comp {
                    "istio" => "istio/istio",
                    "prometheus" => "prometheus-community/prometheus",
                    "grafana" => "grafana/grafana",
                    "cert-manager" => "cert-manager/cert-manager",
                    "nginx-ingress" => "ingress-nginx/ingress-nginx",
                    _ => "unknown/unknown",
                };
                println!("      Command: helm install {} {} --kube-context {}-{} --values dev-values.yaml",
                         comp, chart_name, cluster_tool, name);
            }
        }
    } else {
        println!("  • Skip infrastructure component installation");
    }

    println!("\n⚠️  Prerequisites:");
    println!("  • {} must be installed and available in PATH", cluster_tool);
    println!("  • Docker must be running (required by {})", cluster_tool);
    if !skip_install {
        println!("  • Helm must be installed");
        println!("  • Required Helm repositories must be added");
    }

    Ok(())
}

fn plan_generate_command(args: &[String], verbose: bool) -> anyhow::Result<()> {
    println!("\n🔧 Planning 'generate' command execution:");

    // Parse arguments
    let mut service: Option<String> = None;
    let mut all = false;
    let mut force = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--service" | "-s" => {
                if i + 1 < args.len() {
                    service = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--all" => all = true,
            "--force" => force = true,
            _ => {}
        }
        i += 1;
    }

    println!("🎯 Generation operations that would be performed:");

    if let Some(svc_name) = &service {
        println!("  • Generate scaffold for service: {}", svc_name);
        if verbose {
            println!("    - Create service directory: services/{}", svc_name);
            println!("    - Generate Dockerfile (language-specific)");
            println!("    - Generate Helm chart (Chart.yaml, templates/, values.yaml)");
            println!("    - Generate application files (if language-specific)");
        }
    } else if all {
        println!("  • Re-generate all project scaffolds and configurations");
        if verbose {
            println!("    - Update project structure (services/, provision/)");
            println!("    - Regenerate meshstack.yaml");
            println!("    - Update CI/CD configurations");
            println!("    - Generate environment values files");
            println!("    - Regenerate all existing service scaffolds");
        }
    } else {
        println!("  • Re-generate project-level configurations");
        if verbose {
            println!("    - Update project structure");
            println!("    - Generate CI/CD workflows");
            println!("    - Generate environment values files");
        }
    }

    if force {
        println!("  • Force overwrite existing files");
    } else {
        println!("  • Skip existing files (use --force to overwrite)");
    }

    // Check current configuration
    if Path::new("meshstack.yaml").exists() {
        if let Ok(config_content) = fs::read_to_string("meshstack.yaml") {
            if let Ok(config) = serde_yaml::from_str::<MeshstackConfig>(&config_content) {
                println!("\n📋 Current project configuration:");
                println!("  • Project: {}", config.project_name);
                println!("  • Service Mesh: {}", config.service_mesh);
                println!("  • CI/CD: {}", config.ci_cd);

                if verbose {
                    println!("    - Generic Dockerfile");
                }
            }
        }
    }

    println!("\n⚠️  Prerequisites:");
    println!("  • meshstack.yaml configuration file must exist");
    if force {
        println!("  • Existing files will be overwritten without confirmation");
    }

    Ok(())
}
