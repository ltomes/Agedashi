use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write as IoWrite};
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Parser, Debug)]
#[command(name = "agedashi")]
#[command(about = "Generate infrastructure diagrams from Terraform/OpenTofu graph output - making tofu pretty!", long_about = None)]
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

fn get_service_info(resource_type: &str) -> (&str, &str, &str) {
    // Returns (category, color, icon_char)
    match resource_type {
        // Compute
        t if t.contains("aws_instance") => ("EC2", "#FF9900", "💻"),
        t if t.contains("aws_lambda") => ("Lambda", "#FF9900", "λ"),
        t if t.contains("aws_ecs") => ("ECS", "#FF9900", "🐳"),
        t if t.contains("aws_eks") => ("EKS", "#FF9900", "☸"),
        t if t.contains("aws_autoscaling") => ("AutoScaling", "#FF9900", "📈"),

        // Database
        t if t.contains("aws_db_instance") => ("RDS", "#3B48CC", "🗄"),
        t if t.contains("aws_db_subnet_group") => ("RDS", "#3B48CC", "🗄"),
        t if t.contains("aws_dynamodb") => ("DynamoDB", "#3B48CC", "⚡"),
        t if t.contains("aws_elasticache") => ("ElastiCache", "#3B48CC", "💾"),
        t if t.contains("aws_redshift") => ("Redshift", "#3B48CC", "🔷"),

        // Network
        t if t.contains("aws_elb") || t.contains("aws_lb") => ("ELB", "#8C4FFF", "⚖"),
        t if t.contains("aws_vpc") => ("VPC", "#8C4FFF", "🌐"),
        t if t.contains("aws_subnet") => ("Subnet", "#8C4FFF", "📡"),
        t if t.contains("aws_security_group") => ("SecurityGroup", "#DD344C", "🛡"),
        t if t.contains("aws_route53") => ("Route53", "#8C4FFF", "🌍"),
        t if t.contains("aws_cloudfront") => ("CloudFront", "#8C4FFF", "☁"),
        t if t.contains("aws_api_gateway") => ("APIGateway", "#8C4FFF", "🚪"),

        // Storage
        t if t.contains("aws_s3") => ("S3", "#7AA116", "🪣"),
        t if t.contains("aws_ebs") => ("EBS", "#7AA116", "💽"),
        t if t.contains("aws_efs") => ("EFS", "#7AA116", "📁"),

        // Security
        t if t.contains("aws_iam") => ("IAM", "#DD344C", "🔑"),
        t if t.contains("aws_kms") => ("KMS", "#DD344C", "🔐"),

        // Integration
        t if t.contains("aws_sns") => ("SNS", "#FF4F8B", "📢"),
        t if t.contains("aws_sqs") => ("SQS", "#FF4F8B", "📬"),

        // Analytics
        t if t.contains("aws_kinesis") => ("Kinesis", "#8C4FFF", "📊"),

        // Default
        _ => ("Service", "#232F3E", "🔧"),
    }
}

fn generate_dot_graph(graph: &TerraformGraph, name: &str, direction: &str) -> String {
    let mut dot = String::new();

    // Graph header with modern styling
    dot.push_str(&format!("digraph \"{}\" {{\n", name));
    dot.push_str("    graph [fontname=\"Arial\", fontsize=14, bgcolor=\"#F5F5F5\", pad=\"0.5\"];\n");
    dot.push_str("    node [fontname=\"Arial\", fontsize=12, style=\"filled,rounded\", shape=box, margin=\"0.3,0.2\"];\n");
    dot.push_str("    edge [fontname=\"Arial\", fontsize=10, color=\"#555555\", penwidth=2, arrowsize=0.8];\n");
    dot.push_str(&format!("    rankdir={};\n", direction));
    dot.push_str("    splines=ortho;\n");
    dot.push_str("    nodesep=0.8;\n");
    dot.push_str("    ranksep=1.2;\n\n");

    // Filter AWS resources only
    let aws_resources: Vec<_> = graph.resources.iter()
        .filter(|r| r.resource_type.contains("aws_"))
        .collect();

    let mut node_map: HashMap<String, String> = HashMap::new();

    // Generate nodes with styled boxes
    for (idx, resource) in aws_resources.iter().enumerate() {
        let node_id = format!("node_{}", idx);
        let (service_name, color, icon) = get_service_info(&resource.resource_type);

        // Create a nice label with service type and resource name
        let label = format!("{} {}\\n{}", icon, service_name, resource.label);

        // Create node with styling
        dot.push_str(&format!(
            "    {} [label=\"{}\", fillcolor=\"{}\", fontcolor=\"white\", style=\"filled,rounded\"];\n",
            node_id, label, color
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
    dot
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
        anyhow::bail!("No input provided. Please pipe terraform graph output to agedashi.");
    }

    // Parse the Terraform graph
    let graph = parse_dot_graph(&input).context("Failed to parse Terraform graph")?;

    if graph.resources.is_empty() {
        eprintln!("Warning: No resources found in the Terraform graph.");
    }

    eprintln!("Found {} resources and {} edges", graph.resources.len(), graph.edges.len());

    // Filter for AWS resources
    let aws_count = graph.resources.iter().filter(|r| r.resource_type.contains("aws_")).count();

    if aws_count == 0 {
        eprintln!("Warning: No AWS resources found to visualize.");
        return Ok(());
    }

    eprintln!("Visualizing {} AWS resources...", aws_count);

    // Generate DOT graph
    let dot_content = generate_dot_graph(&graph, &cli.name, &cli.direction);

    // For debugging: save DOT file
    if std::env::var("AGEDASHI_DEBUG").is_ok() {
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
