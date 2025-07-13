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
        Commands::Install { component, profile, dry_run, context } => {
            install_component(component, profile, *dry_run, context)?;
        }
    }
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