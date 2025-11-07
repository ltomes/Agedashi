use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{self, Read};
use std::process::{Command, Stdio};

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

fn map_terraform_to_diagrams_node(resource_type: &str) -> (&str, &str) {
    // Map terraform resource types to diagrams module and class
    match resource_type {
        // Compute
        t if t.contains("aws_instance") => ("diagrams.aws.compute", "EC2"),
        t if t.contains("aws_lambda") => ("diagrams.aws.compute", "Lambda"),
        t if t.contains("aws_ecs_service") => ("diagrams.aws.compute", "ECS"),
        t if t.contains("aws_ecs") => ("diagrams.aws.compute", "ECS"),
        t if t.contains("aws_eks") => ("diagrams.aws.compute", "EKS"),
        t if t.contains("aws_autoscaling") => ("diagrams.aws.compute", "AutoScaling"),

        // Database
        t if t.contains("aws_db_instance") => ("diagrams.aws.database", "RDS"),
        t if t.contains("aws_db_subnet_group") => ("diagrams.aws.database", "RDS"),
        t if t.contains("aws_dynamodb") => ("diagrams.aws.database", "DynamoDB"),
        t if t.contains("aws_elasticache") => ("diagrams.aws.database", "ElastiCache"),
        t if t.contains("aws_redshift") => ("diagrams.aws.database", "Redshift"),

        // Network
        t if t.contains("aws_elb") || t.contains("aws_lb") => ("diagrams.aws.network", "ELB"),
        t if t.contains("aws_vpc") => ("diagrams.aws.network", "VPC"),
        t if t.contains("aws_subnet") => ("diagrams.aws.network", "PublicSubnet"),
        t if t.contains("aws_security_group") => ("diagrams.aws.network", "VPC"),
        t if t.contains("aws_route53") => ("diagrams.aws.network", "Route53"),
        t if t.contains("aws_cloudfront") => ("diagrams.aws.network", "CloudFront"),
        t if t.contains("aws_api_gateway") => ("diagrams.aws.network", "APIGateway"),

        // Storage
        t if t.contains("aws_s3") => ("diagrams.aws.storage", "S3"),
        t if t.contains("aws_ebs") => ("diagrams.aws.storage", "EBS"),
        t if t.contains("aws_efs") => ("diagrams.aws.storage", "EFS"),

        // Security
        t if t.contains("aws_iam") => ("diagrams.aws.security", "IAM"),
        t if t.contains("aws_kms") => ("diagrams.aws.security", "KMS"),

        // Integration
        t if t.contains("aws_sns") => ("diagrams.aws.integration", "SNS"),
        t if t.contains("aws_sqs") => ("diagrams.aws.integration", "SQS"),

        // Analytics
        t if t.contains("aws_kinesis") => ("diagrams.aws.analytics", "Kinesis"),

        // Default to a generic compute node
        _ => ("diagrams.aws.compute", "EC2"),
    }
}

fn generate_python_code(graph: &TerraformGraph, name: &str, direction: &str, output_format: &str) -> String {
    let mut imports: HashSet<String> = HashSet::new();
    let mut node_definitions: Vec<String> = Vec::new();
    let mut node_var_map: HashMap<String, String> = HashMap::new();

    // Filter AWS resources only
    let aws_resources: Vec<_> = graph.resources.iter()
        .filter(|r| r.resource_type.contains("aws_"))
        .collect();

    // Generate imports and node definitions
    for (idx, resource) in aws_resources.iter().enumerate() {
        let (module, class) = map_terraform_to_diagrams_node(&resource.resource_type);
        imports.insert(format!("from {} import {}", module, class));

        let var_name = format!("node_{}", idx);
        let label = resource.label.replace("\"", "\\\"");
        node_definitions.push(format!("    {} = {}(\"{}\")", var_name, class, label));
        node_var_map.insert(resource.name.clone(), var_name);
    }

    // Generate edges
    let mut edge_definitions: Vec<String> = Vec::new();
    for (from, to) in &graph.edges {
        if let (Some(from_var), Some(to_var)) = (node_var_map.get(from), node_var_map.get(to)) {
            edge_definitions.push(format!("    {} >> {}", from_var, to_var));
        }
    }

    let mut code = String::new();
    code.push_str("from diagrams import Diagram\n");
    for import in imports {
        code.push_str(&format!("{}\n", import));
    }
    code.push_str("\n");
    code.push_str(&format!("with Diagram(\"{}\", show=False, direction=\"{}\", outformat=\"{}\"):\n", name, direction, output_format));

    for def in node_definitions {
        code.push_str(&format!("{}\n", def));
    }

    code.push_str("\n");
    for edge_def in edge_definitions {
        code.push_str(&format!("{}\n", edge_def));
    }

    code
}

fn execute_python_code(python_code: &str) -> Result<()> {
    let mut child = Command::new("python3")
        .arg("-c")
        .arg(python_code)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to execute Python code. Make sure Python 3 and the 'diagrams' package are installed.")?;

    let status = child.wait().context("Failed to wait for Python process")?;

    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut stderr_pipe) = child.stderr {
            stderr_pipe.read_to_string(&mut stderr)?;
        }
        anyhow::bail!("Python execution failed: {}", stderr);
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

    // Parse the DOT graph
    let graph = parse_dot_graph(&input).context("Failed to parse Terraform graph")?;

    if graph.resources.is_empty() {
        eprintln!("Warning: No AWS resources found in the Terraform graph.");
    }

    eprintln!("Found {} resources and {} edges", graph.resources.len(), graph.edges.len());

    // Generate Python code
    let python_code = generate_python_code(&graph, &cli.name, &cli.direction, &cli.output);

    // For debugging: print the generated Python code
    eprintln!("Generated Python code:");
    eprintln!("{}", python_code);
    eprintln!("\nExecuting...");

    // Execute Python code
    execute_python_code(&python_code)?;

    println!("Diagram generated successfully: {}.{}", cli.name, cli.output);

    Ok(())
}
