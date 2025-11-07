use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Parser, Debug)]
#[command(name = "terrok")]
#[command(about = "Generate infrastructure diagrams from Terraform graph output", long_about = None)]
struct Cli {
    /// Output format (svg, png, pdf, jpg)
    #[arg(short, long, default_value = "png")]
    output: String,

    /// Output file name (without extension)
    #[arg(short, long, default_value = "infrastructure")]
    name: String,

    /// Show graph direction (TB=top-to-bottom, LR=left-to-right)
    #[arg(short, long, default_value = "TB")]
    direction: String,
}

#[derive(Debug, Clone)]
struct TerraformResource {
    name: String,
    resource_type: String,
    label: String,
}

#[derive(Debug)]
struct TerraformGraph {
    resources: Vec<TerraformResource>,
    edges: Vec<(String, String)>,
}

impl TerraformGraph {
    fn new() -> Self {
        Self {
            resources: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn add_resource(&mut self, name: String, resource_type: String, label: String) {
        self.resources.push(TerraformResource {
            name,
            resource_type,
            label,
        });
    }

    fn add_edge(&mut self, from: String, to: String) {
        self.edges.push((from, to));
    }
}

fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Some(cache_home) = dirs::cache_dir() {
        cache_home.join("terrok").join("icons")
    } else {
        // Fallback to home directory
        dirs::home_dir()
            .context("Could not determine home directory")?
            .join(".cache")
            .join("terrok")
            .join("icons")
    };

    fs::create_dir_all(&cache_dir)
        .context("Failed to create cache directory")?;

    Ok(cache_dir)
}

fn url_to_filename(url: &str) -> String {
    // Create a hash of the URL for the filename
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    // Extract extension from URL
    let ext = url.split('.').last().unwrap_or("png");

    format!("{}.{}", &hash[..16], ext)
}

fn download_icon(url: &str, cache_dir: &Path) -> Result<PathBuf> {
    let filename = url_to_filename(url);
    let cache_path = cache_dir.join(&filename);

    // Return cached file if it exists
    if cache_path.exists() {
        return Ok(cache_path);
    }

    // Download the icon
    eprintln!("Downloading icon: {}", url);
    let response = reqwest::blocking::get(url)
        .context(format!("Failed to download icon from {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download icon: HTTP {}", response.status());
    }

    let bytes = response.bytes()
        .context("Failed to read icon data")?;

    // Save to cache
    fs::write(&cache_path, &bytes)
        .context("Failed to save icon to cache")?;

    Ok(cache_path)
}

fn parse_dot_graph(dot_content: &str) -> Result<TerraformGraph> {
    let mut graph = TerraformGraph::new();

    // Regex to match node definitions
    let node_re = Regex::new(r#"\[root\] ([^\s]+) \(([^)]+)\)"#)?;
    let edge_re = Regex::new(r#"\[root\] ([^\s]+) .*-> \[root\] ([^\s]+)"#)?;
    let label_re = Regex::new(r#"label = "([^"]+)""#)?;

    let mut node_map: HashMap<String, (String, String)> = HashMap::new();

    for line in dot_content.lines() {
        let line = line.trim();

        // Parse node definitions
        if line.contains("[label =") {
            if let Some(caps) = node_re.captures(line) {
                let full_name = caps.get(1).map_or("", |m| m.as_str()).to_string();

                // Extract resource type from the node name (e.g., "aws_instance" from "aws_instance.web")
                let resource_type = if let Some(dot_pos) = full_name.find('.') {
                    full_name[..dot_pos].to_string()
                } else {
                    full_name.clone()
                };

                let label = label_re
                    .captures(line)
                    .and_then(|c| c.get(1))
                    .map_or(full_name.clone(), |m| m.as_str().to_string());

                node_map.insert(full_name.clone(), (resource_type, label));
            }
        }

        // Parse edges
        if line.contains("->") {
            if let Some(caps) = edge_re.captures(line) {
                let from = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let to = caps.get(2).map_or("", |m| m.as_str()).to_string();
                graph.add_edge(from, to);
            }
        }
    }

    // Add all nodes to the graph
    for (name, (resource_type, label)) in node_map {
        graph.add_resource(name, resource_type, label);
    }

    Ok(graph)
}

fn get_aws_icon_url(resource_type: &str) -> &str {
    // Using AWS Architecture Icons from GitHub
    match resource_type {
        // Compute
        t if t.contains("aws_instance") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Compute/EC2.png",
        t if t.contains("aws_lambda") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Compute/Lambda.png",
        t if t.contains("aws_ecs") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Compute/ECS.png",
        t if t.contains("aws_eks") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Compute/EKS.png",
        t if t.contains("aws_autoscaling") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Compute/EC2AutoScaling.png",

        // Database
        t if t.contains("aws_db_instance") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Database/RDS.png",
        t if t.contains("aws_db_subnet_group") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Database/RDS.png",
        t if t.contains("aws_dynamodb") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Database/DynamoDB.png",
        t if t.contains("aws_elasticache") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Database/ElastiCache.png",
        t if t.contains("aws_redshift") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Database/Redshift.png",

        // Network
        t if t.contains("aws_elb") || t.contains("aws_lb") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/NetworkingContentDelivery/ElasticLoadBalancing.png",
        t if t.contains("aws_vpc") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/NetworkingContentDelivery/VPC.png",
        t if t.contains("aws_subnet") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/NetworkingContentDelivery/VPC.png",
        t if t.contains("aws_security_group") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/SecurityIdentityCompliance/IAM.png",
        t if t.contains("aws_route53") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/NetworkingContentDelivery/Route53.png",
        t if t.contains("aws_cloudfront") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/NetworkingContentDelivery/CloudFront.png",
        t if t.contains("aws_api_gateway") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/ApplicationIntegration/APIGateway.png",

        // Storage
        t if t.contains("aws_s3") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Storage/SimpleStorageService.png",
        t if t.contains("aws_ebs") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Storage/EBS.png",
        t if t.contains("aws_efs") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Storage/EFS.png",

        // Security
        t if t.contains("aws_iam") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/SecurityIdentityCompliance/IAM.png",
        t if t.contains("aws_kms") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/SecurityIdentityCompliance/KMS.png",

        // Integration
        t if t.contains("aws_sns") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/ApplicationIntegration/SimpleNotificationService.png",
        t if t.contains("aws_sqs") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/ApplicationIntegration/SQS.png",

        // Analytics
        t if t.contains("aws_kinesis") => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Analytics/Kinesis.png",

        // Default
        _ => "https://raw.githubusercontent.com/awslabs/aws-icons-for-plantuml/v18.0/dist/Compute/EC2.png",
    }
}

fn get_service_color(resource_type: &str) -> &str {
    match resource_type {
        t if t.contains("aws_instance") || t.contains("aws_lambda") || t.contains("aws_ecs") || t.contains("aws_eks") => "#FF9900",
        t if t.contains("aws_db_") || t.contains("aws_dynamodb") || t.contains("aws_elasticache") || t.contains("aws_redshift") => "#3B48CC",
        t if t.contains("aws_vpc") || t.contains("aws_subnet") || t.contains("aws_lb") || t.contains("aws_elb") || t.contains("aws_route53") || t.contains("aws_cloudfront") || t.contains("aws_api_gateway") => "#8C4FFF",
        t if t.contains("aws_s3") || t.contains("aws_ebs") || t.contains("aws_efs") => "#7AA116",
        t if t.contains("aws_iam") || t.contains("aws_kms") || t.contains("aws_security_group") => "#DD344C",
        t if t.contains("aws_sns") || t.contains("aws_sqs") => "#FF4F8B",
        t if t.contains("aws_kinesis") => "#8C4FFF",
        _ => "#232F3E",
    }
}

fn generate_dot_graph(graph: &TerraformGraph, name: &str, direction: &str, cache_dir: &Path) -> Result<String> {
    let mut dot = String::new();

    // Graph header
    dot.push_str(&format!("digraph \"{}\" {{\n", name));
    dot.push_str("    node [shape=box, style=filled, fontname=\"Arial\", fontsize=12];\n");
    dot.push_str("    edge [color=\"#666666\", penwidth=2];\n");
    dot.push_str(&format!("    rankdir={};\n", direction));
    dot.push_str("    bgcolor=\"#FFFFFF\";\n");
    dot.push_str("    splines=ortho;\n");
    dot.push_str("    nodesep=1.0;\n");
    dot.push_str("    ranksep=1.0;\n\n");

    // Filter AWS resources only
    let aws_resources: Vec<_> = graph.resources.iter()
        .filter(|r| r.resource_type.contains("aws_"))
        .collect();

    let mut node_map: HashMap<String, String> = HashMap::new();

    // Generate nodes
    for (idx, resource) in aws_resources.iter().enumerate() {
        let node_id = format!("node_{}", idx);
        let icon_url = get_aws_icon_url(&resource.resource_type);
        let color = get_service_color(&resource.resource_type);

        // Download and cache the icon
        let icon_path = download_icon(icon_url, cache_dir)?;
        let icon_path_str = icon_path.to_string_lossy();

        // Create node with local image path
        dot.push_str(&format!(
            "    {} [label=\"{}\", image=\"{}\", fillcolor=\"{}\", imagescale=true, fixedsize=true, width=2, height=2];\n",
            node_id, resource.label, icon_path_str, color
        ));

        node_map.insert(resource.name.clone(), node_id);
    }

    dot.push_str("\n");

    // Generate edges
    for (from, to) in &graph.edges {
        if let (Some(from_node), Some(to_node)) = (node_map.get(from), node_map.get(to)) {
            dot.push_str(&format!("    {} -> {};\n", from_node, to_node));
        }
    }

    dot.push_str("}\n");
    Ok(dot)
}

fn execute_dot_command(dot_content: &str, output_format: &str, output_file: &str) -> Result<()> {
    // Check if dot command exists
    let dot_check = Command::new("dot")
        .arg("-V")
        .output();

    if dot_check.is_err() {
        anyhow::bail!(
            "GraphViz 'dot' command not found. Please install GraphViz:\n\
             - macOS: brew install graphviz\n\
             - Ubuntu/Debian: sudo apt-get install graphviz\n\
             - Fedora: sudo dnf install graphviz\n\
             - Windows: Download from https://graphviz.org/download/"
        );
    }

    // Create temporary file for DOT content
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(dot_content.as_bytes())?;
    let temp_path = temp_file.path();

    // Execute dot command
    let output = Command::new("dot")
        .arg(format!("-T{}", output_format))
        .arg(temp_path)
        .arg("-o")
        .arg(output_file)
        .output()
        .context("Failed to execute dot command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("GraphViz dot command failed: {}", stderr);
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("Failed to read from stdin")?;

    if input.trim().is_empty() {
        anyhow::bail!("No input provided. Please pipe terraform graph output to terrok.");
    }

    // Parse the Terraform graph
    let graph = parse_dot_graph(&input).context("Failed to parse Terraform graph")?;

    if graph.resources.is_empty() {
        eprintln!("Warning: No AWS resources found in the Terraform graph.");
    }

    eprintln!("Found {} resources and {} edges", graph.resources.len(), graph.edges.len());

    // Filter for AWS resources
    let aws_count = graph.resources.iter().filter(|r| r.resource_type.contains("aws_")).count();

    if aws_count == 0 {
        eprintln!("Warning: No AWS resources found to visualize.");
        return Ok(());
    }

    eprintln!("Visualizing {} AWS resources...", aws_count);

    // Get cache directory
    let cache_dir = get_cache_dir()?;
    eprintln!("Icon cache: {}", cache_dir.display());

    // Generate DOT graph (will download icons as needed)
    let dot_content = generate_dot_graph(&graph, &cli.name, &cli.direction, &cache_dir)?;

    // For debugging: save DOT file
    if std::env::var("TERROK_DEBUG").is_ok() {
        fs::write(format!("{}.dot", cli.name), &dot_content)?;
        eprintln!("DOT file saved to: {}.dot", cli.name);
    }

    // Output file path
    let output_file = format!("{}.{}", cli.name, cli.output);

    // Execute dot command
    execute_dot_command(&dot_content, &cli.output, &output_file)?;

    println!("Diagram generated successfully: {}", output_file);

    Ok(())
}
