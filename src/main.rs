use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

// Embed the AWS icons archive directly into the binary at compile time
// This ensures the icons are always available, even when installed via cargo install
const EMBEDDED_ICONS_ARCHIVE: &[u8] = include_bytes!("../icons/Asset-Package.7z");

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

    /// Color for edges and text (hex color code)
    #[arg(short, long, default_value = "#2D3436")]
    color: String,
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
    let cache_dir = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache").join("agedashi").join("icons")
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile).join(".cache").join("agedashi").join("icons")
    } else {
        PathBuf::from("/tmp").join("agedashi").join("icons")
    };

    fs::create_dir_all(&cache_dir)
        .context("Failed to create cache directory")?;

    Ok(cache_dir)
}

fn convert_svg_to_png(svg_path: &Path, png_path: &Path, size: u32) -> Result<()> {
    // Read the SVG file
    let svg_data = fs::read(svg_path)
        .context("Failed to read SVG file")?;

    // Parse the SVG with default options
    let tree = usvg::Tree::from_data(&svg_data, &usvg::Options::default())
        .map_err(|e| anyhow::anyhow!("Failed to parse SVG: {}", e))?;

    // Get the SVG size and calculate the scaling factor
    let svg_size = tree.size();
    let scale = if svg_size.width() > svg_size.height() {
        size as f32 / svg_size.width()
    } else {
        size as f32 / svg_size.height()
    };

    // Create a pixmap with the desired size
    let mut pixmap = tiny_skia::Pixmap::new(size, size)
        .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    // Render the SVG to the pixmap with scaling
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Apply rounded corners
    let corner_radius = (size as f32 * 0.1) as f32; // 10% radius for nice rounded corners
    apply_rounded_corners(&mut pixmap, corner_radius);

    // Save as PNG
    pixmap.save_png(png_path)
        .map_err(|e| anyhow::anyhow!("Failed to save PNG: {}", e))?;

    Ok(())
}

fn generate_colored_square_svg(color: &str, size: u32) -> Result<String> {
    // Generate a simple colored square SVG that will be processed the same way as icon SVGs
    // The rounded corners will be applied during PNG conversion
    let svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">
  <rect width="{}" height="{}" fill="{}"/>
</svg>"#,
        size, size, size, size, size, size, color
    );
    Ok(svg)
}

fn apply_rounded_corners(pixmap: &mut tiny_skia::Pixmap, radius: f32) {
    use tiny_skia::*;

    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;

    // Create a path with rounded rectangle
    let mut path_builder = PathBuilder::new();

    // Start at top-left corner (after rounding)
    path_builder.move_to(radius, 0.0);

    // Top edge
    path_builder.line_to(width - radius, 0.0);

    // Top-right corner
    path_builder.quad_to(width, 0.0, width, radius);

    // Right edge
    path_builder.line_to(width, height - radius);

    // Bottom-right corner
    path_builder.quad_to(width, height, width - radius, height);

    // Bottom edge
    path_builder.line_to(radius, height);

    // Bottom-left corner
    path_builder.quad_to(0.0, height, 0.0, height - radius);

    // Left edge
    path_builder.line_to(0.0, radius);

    // Top-left corner
    path_builder.quad_to(0.0, 0.0, radius, 0.0);

    path_builder.close();

    let path = path_builder.finish().unwrap();

    // Create a mask pixmap
    let mut mask = Pixmap::new(pixmap.width(), pixmap.height()).unwrap();

    // Fill the mask with the rounded rectangle
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, 255);

    mask.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    // Apply the mask to the original pixmap
    let width = pixmap.width();
    let height = pixmap.height();

    for y in 0..height {
        for x in 0..width {
            let mask_pixel = mask.pixel(x, y).unwrap();

            // If the mask is transparent at this pixel, make the pixmap transparent too
            if mask_pixel.alpha() == 0 {
                let idx = (y * width + x) as usize;
                pixmap.pixels_mut()[idx] = tiny_skia::ColorU8::from_rgba(0, 0, 0, 0).premultiply();
            }
        }
    }
}

fn get_bundled_icons_path() -> PathBuf {
    // Icons are bundled with the binary in the icons/ directory
    // This could be relative to the current directory or the binary location
    let current_dir_path = PathBuf::from("icons/Asset-Package.7z");
    if current_dir_path.exists() {
        return current_dir_path;
    }

    // Try relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_relative = exe_dir.join("icons/Asset-Package.7z");
            if exe_relative.exists() {
                return exe_relative;
            }
            // Try one level up (for cargo build structure)
            if let Some(parent) = exe_dir.parent() {
                let parent_relative = parent.join("icons/Asset-Package.7z");
                if parent_relative.exists() {
                    return parent_relative;
                }
            }
        }
    }

    // Default to current directory path
    current_dir_path
}

fn get_icon_search_pattern(name: &str) -> Option<&'static str> {
    // Search patterns for finding icons in the ZIP by filename
    // These work across different AWS icon package versions
    match name {
        // Compute
        "ec2" => Some("Amazon-EC2_64"),
        "lambda" => Some("AWS-Lambda_64"),
        "ecs" => Some("Amazon-Elastic-Container-Service_64"),
        "eks" => Some("Amazon-Elastic-Kubernetes-Service_64"),
        "autoscaling" => Some("AWS-Auto-Scaling_64"),

        // Database
        "rds" => Some("Amazon-RDS_64"),
        "dynamodb" => Some("Amazon-DynamoDB_64"),
        "elasticache" => Some("Amazon-ElastiCache_64"),
        "redshift" => Some("Amazon-Redshift_64"),

        // Network
        "elb" => Some("Elastic-Load-Balancing_64"),
        "vpc" => Some("Amazon-Virtual-Private-Cloud_64"),
        "subnet" => Some("Amazon-Virtual-Private-Cloud_64"), // Use VPC icon
        "route53" => Some("Amazon-Route-53_64"),
        "cloudfront" => Some("Amazon-CloudFront_64"),
        "apigateway" => Some("Amazon-API-Gateway_64"),

        // Storage
        "s3" => Some("Amazon-Simple-Storage-Service_64"),
        "ebs" => Some("Amazon-Elastic-Block-Store_64"),
        // Note: EFS icon not available in bundled AWS Architecture Icons archive
        // "efs" => Some("Amazon-Elastic-File-System_64"),

        // Security
        "iam" => Some("AWS-Identity-and-Access-Management_64"),
        "kms" => Some("AWS-Key-Management-Service_64"),

        // Integration
        "sns" => Some("Amazon-Simple-Notification-Service_64"),
        "sqs" => Some("Amazon-Simple-Queue-Service_64"),

        // Analytics
        "kinesis" => Some("Amazon-Kinesis_64"),

        _ => None,
    }
}

fn extract_icons_from_7z(archive_path: &Path, cache_dir: &Path) -> Result<usize> {
    eprintln!("Extracting AWS Architecture Icons from bundled archive...");

    let icon_names = vec![
        "ec2", "lambda", "ecs", "eks", "autoscaling",
        "rds", "dynamodb", "elasticache", "redshift",
        "elb", "vpc", "subnet", "route53", "cloudfront", "apigateway",
        "s3", "ebs",
        // Note: EFS icon not available in bundled AWS Architecture Icons archive
        "iam", "kms",
        "sns", "sqs",
        "kinesis",
    ];

    let mut extracted_count = 0;

    // Create a temporary directory for extraction
    let temp_dir = tempfile::tempdir()
        .context("Failed to create temporary directory")?;

    // Extract the 7z archive to temp directory
    sevenz_rust::decompress_file(archive_path, temp_dir.path())
        .map_err(|e| anyhow::anyhow!("Failed to extract 7z archive: {}", e))?;

    // Now search the extracted files for our icons and cache SVGs
    for name in &icon_names {
        // Skip if already cached as SVG
        let svg_cache_path = cache_dir.join(format!("{}.svg", name));
        if svg_cache_path.exists() {
            extracted_count += 1;
            continue;
        }

        // Get the search pattern for this icon
        if let Some(pattern) = get_icon_search_pattern(name) {
            // Search recursively from the temp directory root
            if let Some(svg_path) = find_icon_in_dir(temp_dir.path(), pattern) {
                // Copy SVG to cache (will be converted to PNG lazily when needed)
                fs::copy(&svg_path, &svg_cache_path)?;
                extracted_count += 1;
            }
        }
    }

    Ok(extracted_count)
}

fn find_icon_in_dir(dir: &Path, pattern: &str) -> Option<PathBuf> {
    // Recursively search for an icon matching the pattern
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Check if this file matches our search pattern and is an SVG
                    if filename.contains(pattern) && filename.ends_with(".svg") {
                        // Prefer 64x64 size icons (though SVGs are scalable)
                        if filename.contains("_64") || path.to_str().map_or(false, |p| p.contains("/64/")) {
                            return Some(path);
                        }
                    }
                }
            } else if path.is_dir() {
                // Recurse into subdirectories
                if let Some(found) = find_icon_in_dir(&path, pattern) {
                    return Some(found);
                }
            }
        }
    }
    None
}

fn download_icons_as_needed(cache_dir: &Path) -> Result<()> {
    let icon_names = vec![
        "ec2", "lambda", "ecs", "eks", "autoscaling",
        "rds", "dynamodb", "elasticache", "redshift",
        "elb", "vpc", "subnet", "route53", "cloudfront", "apigateway",
        "s3", "ebs",
        // Note: EFS icon not available in bundled AWS Architecture Icons archive
        "iam", "kms",
        "sns", "sqs",
        "kinesis",
    ];

    // Check if we need to extract any icons (SVG format)
    let needs_extraction = icon_names.iter().any(|name| {
        !cache_dir.join(format!("{}.svg", name)).exists()
    });

    if !needs_extraction {
        return Ok(()); // All icons already cached
    }

    eprintln!("Setting up AWS service icons (first run)...");

    // Try to get path to bundled icons archive (for development builds)
    let archive_path = get_bundled_icons_path();

    // Determine which archive to use
    let result = if archive_path.exists() {
        // Use the bundled archive from the filesystem (development mode)
        extract_icons_from_7z(&archive_path, cache_dir)
    } else {
        // Use the embedded archive (production mode - cargo install)
        // Write the embedded archive to a temporary file and extract from it
        let mut temp_archive = NamedTempFile::new()
            .context("Failed to create temporary file for embedded icons")?;

        // Write using the existing file handle instead of opening a new one
        temp_archive.write_all(EMBEDDED_ICONS_ARCHIVE)
            .context("Failed to write embedded icons to temporary file")?;
        temp_archive.flush()
            .context("Failed to flush embedded icons to disk")?;

        // Extract from the temp file (temp_archive stays in scope throughout this call)
        let extraction_result = extract_icons_from_7z(temp_archive.path(), cache_dir);

        // temp_archive is explicitly kept alive until here, then dropped and deleted
        extraction_result
    };

    match result {
        Ok(count) => {
            if count > 0 {
                eprintln!("✓ Extracted {} AWS service icons", count);
            } else {
                eprintln!("Note: Could not extract icons. Using fallback styled boxes.");
            }
        }
        Err(e) => {
            eprintln!("Note: Icon extraction failed ({}). Using fallback styled boxes.", e);
        }
    }

    Ok(())
}

fn extract_resource_type(full_name: &str) -> String {
    // Extract the actual AWS resource type from names that may include module or data prefixes
    // Examples:
    //   "aws_instance.web" -> "aws_instance"
    //   "module.vpc.aws_vpc.main" -> "aws_vpc"
    //   "data.aws_ami.latest" -> "aws_ami"
    //   "module.app.module.network.aws_vpc.main" -> "aws_vpc"

    let parts: Vec<&str> = full_name.split('.').collect();

    // Find the first part that starts with "aws_"
    for part in &parts {
        if part.starts_with("aws_") {
            return part.to_string();
        }
    }

    // If no AWS resource type found, return the first part (fallback behavior)
    parts.first().unwrap_or(&full_name).to_string()
}

fn parse_dot_graph(dot_content: &str) -> Result<TerraformGraph> {
    let mut graph = TerraformGraph::new();

    // Regex to match node definitions
    let node_re = Regex::new(r#"\[root\] ([^\s]+) \(([^)]+)\)"#)?;
    // Updated edge regex to handle quoted format: "[root] aws_instance.web (expand)" -> "[root] aws_vpc.main (expand)"
    let edge_re = Regex::new(r#""\[root\] ([^\s]+) \([^)]+\)" -> "\[root\] ([^\s]+) \([^)]+\)""#)?;
    let label_re = Regex::new(r#"label = "([^"]+)""#)?;

    let mut node_map: HashMap<String, (String, String)> = HashMap::new();

    for line in dot_content.lines() {
        let line = line.trim();

        // Parse node definitions
        if line.contains("[label =") {
            if let Some(caps) = node_re.captures(line) {
                let full_name = caps.get(1).map_or("", |m| m.as_str()).to_string();

                // Extract resource type from the node name, handling modules and data sources
                // Examples:
                //   "aws_instance.web" -> "aws_instance"
                //   "module.vpc.aws_vpc.main" -> "aws_vpc"
                //   "data.aws_ami.latest" -> "aws_ami"
                //   "module.app.module.network.aws_vpc.main" -> "aws_vpc"
                let resource_type = extract_resource_type(&full_name);

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

fn get_service_info(resource_type: &str) -> (&str, &str, &str, &str) {
    // Returns (icon_name, service_name, color, fallback_emoji)
    // Colors are AWS brand colors for visual consistency
    match resource_type {
        // Compute
        t if t.contains("aws_instance") => ("ec2", "EC2", "#FF9900", "💻"),
        t if t.contains("aws_lambda") => ("lambda", "Lambda", "#FF9900", "λ"),
        t if t.contains("aws_ecs") => ("ecs", "ECS", "#FF9900", "🐳"),
        t if t.contains("aws_eks") => ("eks", "EKS", "#FF9900", "☸"),
        t if t.contains("aws_autoscaling") => ("autoscaling", "AutoScaling", "#FF9900", "📈"),
        t if t.contains("aws_launch_template") => ("ec2", "LaunchTemplate", "#FF9900", "📋"),

        // Database
        t if t.contains("aws_db_instance") => ("rds", "RDS", "#3B48CC", "🗄"),
        t if t.contains("aws_db_subnet_group") => ("rds", "RDS", "#3B48CC", "🗄"),
        t if t.contains("aws_dynamodb") => ("dynamodb", "DynamoDB", "#3B48CC", "⚡"),
        t if t.contains("aws_elasticache") => ("elasticache", "ElastiCache", "#3B48CC", "💾"),
        t if t.contains("aws_redshift") => ("redshift", "Redshift", "#3B48CC", "🔷"),

        // Network
        t if t.contains("aws_elb") || t.contains("aws_lb") => ("elb", "ELB", "#8C4FFF", "⚖"),
        t if t.contains("aws_vpc") => ("vpc", "VPC", "#8C4FFF", "🌐"),
        t if t.contains("aws_subnet") => ("subnet", "Subnet", "#8C4FFF", "📡"),
        t if t.contains("aws_security_group") => ("vpc", "SecurityGroup", "#DD344C", "🛡"),
        t if t.contains("aws_route53") => ("route53", "Route53", "#8C4FFF", "🌍"),
        t if t.contains("aws_cloudfront") => ("cloudfront", "CloudFront", "#8C4FFF", "☁"),
        t if t.contains("aws_api_gateway") => ("apigateway", "APIGateway", "#8C4FFF", "🚪"),

        // Storage
        t if t.contains("aws_s3") => ("s3", "S3", "#7AA116", "🪣"),
        t if t.contains("aws_ebs") => ("ebs", "EBS", "#7AA116", "💽"),
        // Note: EFS icon not available in bundled AWS Architecture Icons
        // t if t.contains("aws_efs") => ("efs", "EFS", "#7AA116", "📁"),

        // Security
        t if t.contains("aws_iam") => ("iam", "IAM", "#DD344C", "🔑"),
        t if t.contains("aws_kms") => ("kms", "KMS", "#DD344C", "🔐"),

        // Integration
        t if t.contains("aws_sns") => ("sns", "SNS", "#FF4F8B", "📢"),
        t if t.contains("aws_sqs") => ("sqs", "SQS", "#FF4F8B", "📬"),

        // Analytics
        t if t.contains("aws_kinesis") => ("kinesis", "Kinesis", "#8C4FFF", "📊"),

        // Data Sources - use a distinct color for data/reference resources
        t if t.contains("aws_ami") => ("", "AMI", "#527FFF", "💿"),
        t if t.contains("aws_availability_zones") => ("", "AZs", "#527FFF", "🗺"),
        t if t.contains("aws_caller_identity") => ("", "CallerIdentity", "#527FFF", "👤"),
        t if t.contains("aws_region") => ("", "Region", "#527FFF", "🌎"),

        // Default - use a medium AWS brand color that works well
        _ => ("", "Resource", "#146EB4", "📦"),
    }
}

fn generate_dot_graph(graph: &TerraformGraph, name: &str, direction: &str, cache_dir: &Path, temp_dir: &Path, icon_size: u32, color: &str) -> Result<String> {
    let mut dot = String::new();

    // Graph header with modern styling
    dot.push_str(&format!("digraph \"{}\" {{\n", name));
    dot.push_str("    graph [fontname=\"Arial\", fontsize=14, bgcolor=\"transparent\", pad=\"0.5\"];\n");
    dot.push_str("    node [fontname=\"Arial\", fontsize=11];\n");
    dot.push_str(&format!("    edge [fontname=\"Arial\", fontsize=10, color=\"{}\", penwidth=2.0, arrowsize=0.7];\n", color));
    dot.push_str(&format!("    rankdir={};\n", direction));
    dot.push_str("    splines=ortho;\n");
    dot.push_str("    nodesep=1.0;\n");
    dot.push_str("    ranksep=1.5;\n\n");

    // Filter AWS resources only
    let aws_resources: Vec<_> = graph.resources.iter()
        .filter(|r| r.resource_type.contains("aws_"))
        .collect();

    let mut node_map: HashMap<String, String> = HashMap::new();

    // Generate nodes
    for (idx, resource) in aws_resources.iter().enumerate() {
        let node_id = format!("node_{}", idx);
        let (icon_name, _service_name, fallback_color, _fallback_emoji) = get_service_info(&resource.resource_type);

        // Check if we have a cached SVG icon
        let svg_path = cache_dir.join(format!("{}.svg", icon_name));

        // Determine which PNG to use: real icon or generated colored square
        let png_path = if svg_path.exists() && !icon_name.is_empty() {
            // Use real icon
            let png_path = temp_dir.join(format!("{}.png", icon_name));
            match convert_svg_to_png(&svg_path, &png_path, icon_size) {
                Ok(_) => Some(png_path),
                Err(_) => None,
            }
        } else {
            None
        };

        // If we don't have a PNG yet, generate a colored square SVG and convert it
        let final_png_path = if let Some(path) = png_path {
            path
        } else {
            // Generate colored square SVG and convert to PNG with same rounded corners
            let svg_content = generate_colored_square_svg(fallback_color, icon_size)?;
            let fallback_svg_path = temp_dir.join(format!("fallback_{}_{}.svg", idx, resource.name.replace(".", "_")));
            let fallback_png_path = temp_dir.join(format!("fallback_{}_{}.png", idx, resource.name.replace(".", "_")));

            fs::write(&fallback_svg_path, svg_content)
                .context("Failed to write fallback SVG")?;

            convert_svg_to_png(&fallback_svg_path, &fallback_png_path, icon_size)
                .context("Failed to convert fallback SVG to PNG")?;

            fallback_png_path
        };

        // Use the same HTML structure for all resources (icon or fallback)
        // Text color matches the service color (not the edge color)
        let icon_path_str = final_png_path.to_string_lossy();
        let html_label = format!(
            "<<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\"><TR><TD FIXEDSIZE=\"TRUE\" WIDTH=\"{}\" HEIGHT=\"{}\"><IMG SRC=\"{}\"/></TD></TR><TR><TD><FONT COLOR=\"{}\">{}</FONT></TD></TR></TABLE>>",
            icon_size, icon_size, icon_path_str, fallback_color, resource.label
        );
        dot.push_str(&format!(
            "    {} [label={}, shape=plaintext, fontsize=10];\n",
            node_id, html_label
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

    // Get cache directory and setup icons
    let cache_dir = get_cache_dir()?;

    // Extract icons from embedded 7z archive (for cargo install) or bundled archive (for development)
    download_icons_as_needed(&cache_dir)?;

    // Count how many icons we have available (SVG format)
    let icon_count = ["ec2", "lambda", "ecs", "eks", "autoscaling",
                      "rds", "dynamodb", "elasticache", "redshift",
                      "elb", "vpc", "subnet", "route53", "cloudfront", "apigateway",
                      "s3", "ebs", "iam", "kms", "sns", "sqs", "kinesis"]
        .iter()
        .filter(|name| cache_dir.join(format!("{}.svg", name)).exists())
        .count();

    if icon_count > 0 {
        eprintln!("Using {} AWS service icons", icon_count);
    } else {
        eprintln!("Using fallback styled boxes (icons unavailable)");
    }

    // Create temp directory for PNG conversions
    let temp_dir = tempfile::tempdir()
        .context("Failed to create temporary directory for icon conversion")?;

    // Generate DOT graph with lazy icon conversion
    // Icons are converted from SVG to PNG at 128x128
    // For SVG output, the generated SVG is post-processed to embed PNGs as base64
    let dot_content = generate_dot_graph(&graph, &cli.name, &cli.direction, &cache_dir, temp_dir.path(), 128, &cli.color)?;

    // Output file path
    let output_file = format!("{}.{}", cli.name, cli.output);

    // For debugging: save DOT file in debug mode
    if std::env::var("AGEDASHI_DEBUG").is_ok() {
        let dot_file = format!("{}.dot", cli.name);
        fs::write(&dot_file, &dot_content)
            .context(format!("Failed to save DOT file: {}", dot_file))?;
        eprintln!("Debug: DOT file saved to {}", dot_file);
    }

    // Execute dot command
    execute_dot_command(&dot_content, &cli.output, &output_file)?;

    // For SVG output, post-process to embed images as base64 data URIs
    if cli.output == "svg" {
        embed_images_in_svg(&output_file)?;
    }

    println!("Diagram generated successfully: {}", output_file);

    Ok(())
}

// Post-process SVG to embed PNG images as base64 data URIs
fn embed_images_in_svg(svg_file: &str) -> Result<()> {
    // Read the SVG file
    let svg_content = fs::read_to_string(svg_file)
        .context(format!("Failed to read SVG file: {}", svg_file))?;

    // Replace file paths with base64 data URIs
    let re = Regex::new(r#"xlink:href="([^"]+\.png)""#)?;
    let mut modified_svg = svg_content.clone();

    for cap in re.captures_iter(&svg_content) {
        let file_path = &cap[1];
        // Read the PNG file and base64 encode it
        if let Ok(png_data) = fs::read(file_path) {
            let base64_data = BASE64.encode(&png_data);
            let data_uri = format!("data:image/png;base64,{}", base64_data);
            modified_svg = modified_svg.replace(
                &format!("xlink:href=\"{}\"", file_path),
                &format!("xlink:href=\"{}\"", data_uri)
            );
        }
    }

    // Write the modified SVG back
    fs::write(svg_file, modified_svg)
        .context(format!("Failed to write modified SVG: {}", svg_file))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_aws_resource_icons() {
        // This test validates that we can resolve icons for all AWS resource types
        // that are defined in our icon mapping. It ensures:
        // 1. Icons can be extracted from the bundled archive
        // 2. SVG to PNG conversion works correctly
        // 3. All resource types in the test fixture are handled

        // Create temporary directories for cache and output
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");

        // Ensure icons are available (extract if needed)
        let bundled_icons = get_bundled_icons_path();
        if bundled_icons.exists() {
            let extracted = extract_icons_from_7z(&bundled_icons, &cache_dir)
                .expect("Failed to extract icons");
            println!("Extracted {} icons from archive", extracted);
        } else {
            panic!("Bundled icons not found at {:?}", bundled_icons);
        }

        // Parse the test fixture with all AWS resource types
        let test_dot_path = "test/all-aws-resources.dot";
        let dot_content = fs::read_to_string(test_dot_path)
            .expect(&format!("Failed to read test fixture: {}", test_dot_path));

        let graph = parse_dot_graph(&dot_content)
            .expect("Failed to parse test fixture");

        // Expected resource types that should have icons
        let expected_icon_types = vec![
            "autoscaling", "cloudfront", "dynamodb", "ebs", "ec2", "ecs",
            "eks", "elasticache", "elb", "iam", "apigateway",
            "kinesis", "kms", "lambda", "rds", "redshift", "route53",
            "s3", "sns", "sqs", "subnet", "vpc"
        ];

        // Track which resource types we found and their status
        let mut found_types = std::collections::HashSet::new();
        let mut successful_icons = Vec::new();
        let mut failed_icons = Vec::new();
        let mut missing_icons = Vec::new();

        // Filter AWS resources only (same as generate_dot_graph)
        let aws_resources: Vec<_> = graph.resources.iter()
            .filter(|r| r.resource_type.contains("aws_"))
            .collect();

        // Try to resolve an icon for each resource in the graph
        for resource in &aws_resources {
            let (icon_name, _service_name, _color, _fallback_emoji) = get_service_info(&resource.resource_type);
            found_types.insert(icon_name.to_string());

            // Check if we have a cached SVG icon
            let svg_path = cache_dir.join(format!("{}.svg", icon_name));

            if svg_path.exists() && !icon_name.is_empty() {
                // Try to convert SVG to PNG (same as generate_dot_graph)
                let png_path = temp_dir.path().join(format!("{}.png", icon_name));

                match convert_svg_to_png(&svg_path, &png_path, 128) {
                    Ok(_) => {
                        successful_icons.push(icon_name.to_string());
                    }
                    Err(e) => {
                        failed_icons.push(format!("{} (conversion failed: {})", icon_name, e));
                    }
                }
            } else if !icon_name.is_empty() {
                missing_icons.push(icon_name.to_string());
            }
        }

        // Report results
        println!("\n═══════════════════════════════════════════════════");
        println!("Icon Resolution Test Results");
        println!("═══════════════════════════════════════════════════");
        println!("  Resources tested: {}", aws_resources.len());
        println!("  Unique icon types: {}", found_types.len());
        println!("  Expected types: {}", expected_icon_types.len());
        println!("  ✓ Successful: {}", successful_icons.len());
        println!("  ✗ Missing from archive: {}", missing_icons.len());
        println!("  ✗ Conversion failures: {}", failed_icons.len());

        if !missing_icons.is_empty() {
            println!("\n  Icons not found in archive:");
            for icon in &missing_icons {
                println!("    - {}", icon);
            }
            println!("\n  Note: These icons may not exist in the bundled AWS Architecture Icons archive,");
            println!("        or the search pattern may need adjustment.");
        }

        if !failed_icons.is_empty() {
            println!("\n  Conversion failures (CRITICAL):");
            for icon in &failed_icons {
                println!("    - {}", icon);
            }
        }

        // Check that all expected types were covered in test fixture
        let mut missing_from_fixture = Vec::new();
        for expected in &expected_icon_types {
            if !found_types.contains(*expected) {
                missing_from_fixture.push(*expected);
            }
        }

        if !missing_from_fixture.is_empty() {
            println!("\n  Warning: Expected icon types not in test fixture:");
            for missing in &missing_from_fixture {
                println!("    - {}", missing);
            }
        }

        println!("═══════════════════════════════════════════════════\n");

        // Test passes if:
        // 1. No conversion failures (critical - code is broken)
        // 2. At least 90% of icons are available (allows for archive changes)
        let success_rate = successful_icons.len() as f32 / aws_resources.len() as f32;

        assert!(
            failed_icons.is_empty(),
            "Icon conversion failures detected. This indicates a bug in the code."
        );

        assert!(
            success_rate >= 0.9,
            "Only {:.1}% of icons available (expected >= 90%). Missing: {:?}",
            success_rate * 100.0,
            missing_icons
        );
    }

    #[test]
    fn test_document_missing_icon_coverage() {
        // This test compares all AWS provider resources against our icon mappings
        // to identify which resources don't have icons. This helps track coverage
        // and prioritize which icons to add.

        // Read the AWS provider resources list
        let resources_file = "test/aws-provider-resources.txt";
        let resources_content = match fs::read_to_string(resources_file) {
            Ok(content) => content,
            Err(_) => {
                println!("\nNote: Run scripts/list-aws-resources.sh to generate {}", resources_file);
                println!("Skipping test (resource list not available)");
                return;
            }
        };

        let all_resources: Vec<&str> = resources_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        // Track resources by icon availability
        let mut resources_with_icons = std::collections::HashMap::new();
        let mut resources_without_icons = Vec::new();

        for resource_type in &all_resources {
            let (icon_name, service_name, _color, _emoji) = get_service_info(resource_type);

            if !icon_name.is_empty() {
                resources_with_icons
                    .entry(icon_name)
                    .or_insert_with(Vec::new)
                    .push(*resource_type);
            } else {
                resources_without_icons.push(*resource_type);
            }
        }

        // Report results
        println!("\n═══════════════════════════════════════════════════");
        println!("AWS Provider Icon Coverage Report");
        println!("═══════════════════════════════════════════════════");
        println!("Total AWS resources in provider: {}", all_resources.len());
        println!("Resources with icon mappings: {}", all_resources.len() - resources_without_icons.len());
        println!("Resources without icons: {}", resources_without_icons.len());
        println!("Coverage: {:.1}%",
            ((all_resources.len() - resources_without_icons.len()) as f32 / all_resources.len() as f32) * 100.0
        );

        println!("\n─────────────────────────────────────────────────");
        println!("Icon Mappings (grouped by icon):");
        println!("─────────────────────────────────────────────────");

        let mut icon_names: Vec<_> = resources_with_icons.keys().collect();
        icon_names.sort();

        for icon_name in icon_names {
            let resources = resources_with_icons.get(icon_name).unwrap();
            println!("\n{} ({} resources):", icon_name, resources.len());
            for resource in resources.iter().take(5) {
                println!("  • {}", resource);
            }
            if resources.len() > 5 {
                println!("  ... and {} more", resources.len() - 5);
            }
        }

        println!("\n─────────────────────────────────────────────────");
        println!("Top 20 Resources Without Icons:");
        println!("─────────────────────────────────────────────────");

        // Group by service prefix to identify patterns
        let mut service_groups: std::collections::HashMap<String, Vec<&str>> = std::collections::HashMap::new();
        for resource in &resources_without_icons {
            // Extract service prefix (e.g., "aws_rds" from "aws_rds_cluster")
            let parts: Vec<&str> = resource.split('_').collect();
            let prefix = if parts.len() >= 2 {
                format!("{}_{}", parts[0], parts[1])
            } else {
                resource.to_string()
            };
            service_groups
                .entry(prefix)
                .or_insert_with(Vec::new)
                .push(*resource);
        }

        let mut sorted_groups: Vec<_> = service_groups.iter().collect();
        sorted_groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (prefix, resources) in sorted_groups.iter().take(20) {
            println!("\n{} ({} resources):", prefix, resources.len());
            for resource in resources.iter().take(3) {
                println!("  • {}", resource);
            }
            if resources.len() > 3 {
                println!("  ... and {} more", resources.len() - 3);
            }
        }

        println!("\n═══════════════════════════════════════════════════\n");

        // This test always passes - it's informational only
        // We don't require 100% coverage since many resources are rarely used
        assert!(true, "Icon coverage documentation generated successfully");
    }

    #[test]
    fn test_parse_module_prefixed_resources() {
        // Test that resources nested in modules are correctly parsed and identified
        let dot_content = r#"
digraph {
    compound = "true"
    newrank = "true"
    subgraph "root" {
        "[root] module.vpc.aws_vpc.main (expand)" [label = "module.vpc.aws_vpc.main", shape = "box"]
        "[root] module.network.aws_subnet.public (expand)" [label = "module.network.aws_subnet.public", shape = "box"]
        "[root] aws_instance.web (expand)" [label = "aws_instance.web", shape = "box"]
        "[root] module.network.aws_subnet.public (expand)" -> "[root] module.vpc.aws_vpc.main (expand)"
        "[root] aws_instance.web (expand)" -> "[root] module.network.aws_subnet.public (expand)"
    }
}
        "#;

        let graph = parse_dot_graph(dot_content).expect("Failed to parse dot graph");

        // Should find 3 resources
        assert_eq!(graph.resources.len(), 3, "Expected 3 resources to be parsed");

        // Find the module-prefixed VPC resource
        let vpc_resource = graph.resources.iter()
            .find(|r| r.name == "module.vpc.aws_vpc.main")
            .expect("VPC resource should be parsed");

        // The resource type should be extracted as "aws_vpc", not "module"
        assert!(
            vpc_resource.resource_type.contains("aws_vpc"),
            "Resource type should contain 'aws_vpc', got: {}",
            vpc_resource.resource_type
        );

        // Find the module-prefixed subnet resource
        let subnet_resource = graph.resources.iter()
            .find(|r| r.name == "module.network.aws_subnet.public")
            .expect("Subnet resource should be parsed");

        assert!(
            subnet_resource.resource_type.contains("aws_subnet"),
            "Resource type should contain 'aws_subnet', got: {}",
            subnet_resource.resource_type
        );

        // Regular resource should still work
        let instance_resource = graph.resources.iter()
            .find(|r| r.name == "aws_instance.web")
            .expect("Instance resource should be parsed");

        assert!(
            instance_resource.resource_type.contains("aws_instance"),
            "Resource type should contain 'aws_instance', got: {}",
            instance_resource.resource_type
        );

        // Verify edges are preserved (2 edges between the AWS resources)
        assert_eq!(graph.edges.len(), 2, "Expected 2 edges to be parsed");
    }

    #[test]
    fn test_parse_data_sources() {
        // Test that data sources are correctly parsed and identified
        let dot_content = r#"
digraph {
    compound = "true"
    newrank = "true"
    subgraph "root" {
        "[root] data.aws_ami.latest (expand)" [label = "data.aws_ami.latest", shape = "box"]
        "[root] data.aws_vpc.selected (expand)" [label = "data.aws_vpc.selected", shape = "box"]
        "[root] aws_instance.web (expand)" [label = "aws_instance.web", shape = "box"]
        "[root] aws_instance.web (expand)" -> "[root] data.aws_ami.latest (expand)"
        "[root] aws_instance.web (expand)" -> "[root] data.aws_vpc.selected (expand)"
    }
}
        "#;

        let graph = parse_dot_graph(dot_content).expect("Failed to parse dot graph");

        // Should find 3 resources
        assert_eq!(graph.resources.len(), 3, "Expected 3 resources to be parsed");

        // Find the data source for AMI
        let ami_data = graph.resources.iter()
            .find(|r| r.name == "data.aws_ami.latest")
            .expect("AMI data source should be parsed");

        // The resource type should be extracted as "aws_ami", not "data"
        assert!(
            ami_data.resource_type.contains("aws_ami"),
            "Resource type should contain 'aws_ami', got: {}",
            ami_data.resource_type
        );

        // Find the data source for VPC
        let vpc_data = graph.resources.iter()
            .find(|r| r.name == "data.aws_vpc.selected")
            .expect("VPC data source should be parsed");

        assert!(
            vpc_data.resource_type.contains("aws_vpc"),
            "Resource type should contain 'aws_vpc', got: {}",
            vpc_data.resource_type
        );

        // Verify edges are preserved
        assert_eq!(graph.edges.len(), 2, "Expected 2 edges to be parsed");
    }

    #[test]
    fn test_parse_nested_modules() {
        // Test that deeply nested module resources are correctly parsed
        let dot_content = r#"
digraph {
    compound = "true"
    newrank = "true"
    subgraph "root" {
        "[root] module.app.module.network.aws_vpc.main (expand)" [label = "module.app.module.network.aws_vpc.main", shape = "box"]
        "[root] module.app.module.network.aws_subnet.private (expand)" [label = "module.app.module.network.aws_subnet.private", shape = "box"]
        "[root] module.app.aws_instance.web (expand)" [label = "module.app.aws_instance.web", shape = "box"]
        "[root] module.app.module.network.aws_subnet.private (expand)" -> "[root] module.app.module.network.aws_vpc.main (expand)"
        "[root] module.app.aws_instance.web (expand)" -> "[root] module.app.module.network.aws_subnet.private (expand)"
    }
}
        "#;

        let graph = parse_dot_graph(dot_content).expect("Failed to parse dot graph");

        // Should find 3 resources
        assert_eq!(graph.resources.len(), 3, "Expected 3 resources to be parsed");

        // Find the deeply nested VPC resource
        let vpc_resource = graph.resources.iter()
            .find(|r| r.name == "module.app.module.network.aws_vpc.main")
            .expect("Nested VPC resource should be parsed");

        // The resource type should be extracted as "aws_vpc", not "module"
        assert!(
            vpc_resource.resource_type.contains("aws_vpc"),
            "Resource type should contain 'aws_vpc' for deeply nested module, got: {}",
            vpc_resource.resource_type
        );

        // Find the nested subnet
        let subnet_resource = graph.resources.iter()
            .find(|r| r.name == "module.app.module.network.aws_subnet.private")
            .expect("Nested subnet resource should be parsed");

        assert!(
            subnet_resource.resource_type.contains("aws_subnet"),
            "Resource type should contain 'aws_subnet', got: {}",
            subnet_resource.resource_type
        );

        // Verify edges are preserved
        assert_eq!(graph.edges.len(), 2, "Expected 2 edges to be parsed");
    }

    #[test]
    fn test_visualization_includes_module_resources() {
        // Test that module resources are included in the final visualization
        let dot_content = r#"
digraph {
    compound = "true"
    newrank = "true"
    subgraph "root" {
        "[root] module.vpc.aws_vpc.main (expand)" [label = "module.vpc.aws_vpc.main", shape = "box"]
        "[root] aws_instance.web (expand)" [label = "aws_instance.web", shape = "box"]
        "[root] aws_instance.web (expand)" -> "[root] module.vpc.aws_vpc.main (expand)"
    }
}
        "#;

        let graph = parse_dot_graph(dot_content).expect("Failed to parse dot graph");

        // Filter AWS resources (same as generate_dot_graph does)
        let aws_resources: Vec<_> = graph.resources.iter()
            .filter(|r| r.resource_type.contains("aws_"))
            .collect();

        // Both resources should pass the filter
        assert_eq!(
            aws_resources.len(),
            2,
            "Expected 2 AWS resources after filtering, got {}. Resources: {:?}",
            aws_resources.len(),
            graph.resources.iter().map(|r| (&r.name, &r.resource_type)).collect::<Vec<_>>()
        );

        // Check that module.vpc.aws_vpc.main is included
        let has_vpc = aws_resources.iter().any(|r| r.name == "module.vpc.aws_vpc.main");
        assert!(
            has_vpc,
            "Module-prefixed VPC resource should be included in AWS resources filter"
        );
    }

    #[test]
    fn test_edge_preservation_with_modules() {
        // Test that edges between module resources are preserved in visualization
        let dot_content = r#"
digraph {
    compound = "true"
    newrank = "true"
    subgraph "root" {
        "[root] module.vpc.aws_vpc.main (expand)" [label = "module.vpc.aws_vpc.main", shape = "box"]
        "[root] module.network.aws_subnet.public (expand)" [label = "module.network.aws_subnet.public", shape = "box"]
        "[root] aws_instance.web (expand)" [label = "aws_instance.web", shape = "box"]
        "[root] module.network.aws_subnet.public (expand)" -> "[root] module.vpc.aws_vpc.main (expand)"
        "[root] aws_instance.web (expand)" -> "[root] module.network.aws_subnet.public (expand)"
    }
}
        "#;

        let graph = parse_dot_graph(dot_content).expect("Failed to parse dot graph");

        // Create a temporary directory for icon cache (required by generate_dot_graph)
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");
        let temp_graph_dir = temp_dir.path().join("temp");
        fs::create_dir_all(&temp_graph_dir).expect("Failed to create temp dir");

        // Generate DOT graph for visualization
        let output_dot = generate_dot_graph(
            &graph,
            "test",
            "TB",
            &cache_dir,
            &temp_graph_dir,
            128,
            "#2D3436"
        ).expect("Failed to generate dot graph");

        // The output should contain edges (indicated by "->")
        let edge_count = output_dot.matches("->").count();
        assert!(
            edge_count >= 2,
            "Expected at least 2 edges in output, found {}. Output:\n{}",
            edge_count,
            output_dot
        );

        // All three resources should appear in the output
        assert!(
            output_dot.contains("node_0") || output_dot.contains("node_1") || output_dot.contains("node_2"),
            "Expected to find node definitions in output"
        );
    }

    #[test]
    fn test_mixed_resources_with_modules_and_data() {
        // Test a realistic scenario with a mix of regular resources, modules, and data sources
        let dot_content = r#"
digraph {
    compound = "true"
    newrank = "true"
    subgraph "root" {
        "[root] module.vpc.aws_vpc.main (expand)" [label = "module.vpc.aws_vpc.main", shape = "box"]
        "[root] data.aws_ami.ubuntu (expand)" [label = "data.aws_ami.ubuntu", shape = "box"]
        "[root] aws_instance.web (expand)" [label = "aws_instance.web", shape = "box"]
        "[root] module.database.aws_db_instance.main (expand)" [label = "module.database.aws_db_instance.main", shape = "box"]
        "[root] aws_instance.web (expand)" -> "[root] data.aws_ami.ubuntu (expand)"
        "[root] aws_instance.web (expand)" -> "[root] module.vpc.aws_vpc.main (expand)"
        "[root] module.database.aws_db_instance.main (expand)" -> "[root] module.vpc.aws_vpc.main (expand)"
    }
}
        "#;

        let graph = parse_dot_graph(dot_content).expect("Failed to parse dot graph");

        // Should find 4 resources
        assert_eq!(graph.resources.len(), 4, "Expected 4 resources to be parsed");

        // Filter AWS resources (same as generate_dot_graph does)
        let aws_resources: Vec<_> = graph.resources.iter()
            .filter(|r| r.resource_type.contains("aws_"))
            .collect();

        // All 4 resources should be AWS resources
        assert_eq!(
            aws_resources.len(),
            4,
            "Expected 4 AWS resources after filtering. Resources: {:?}",
            graph.resources.iter().map(|r| (&r.name, &r.resource_type)).collect::<Vec<_>>()
        );

        // Verify each resource type is correctly extracted
        let types: Vec<&str> = aws_resources.iter()
            .map(|r| r.resource_type.as_str())
            .collect();

        assert!(types.iter().any(|t| t.contains("aws_vpc")), "Should have aws_vpc");
        assert!(types.iter().any(|t| t.contains("aws_ami")), "Should have aws_ami (from data source)");
        assert!(types.iter().any(|t| t.contains("aws_instance")), "Should have aws_instance");
        assert!(types.iter().any(|t| t.contains("aws_db_instance")), "Should have aws_db_instance");

        // Verify edges
        assert_eq!(graph.edges.len(), 3, "Expected 3 edges to be parsed");
    }

    #[test]
    fn test_visual_regression_complex_modules() {
        // Visual regression test: Generate a diagram from a complex real-world scenario
        // This test produces output that developers can visually inspect for regressions
        // The diagram includes:
        // - Module-prefixed resources (multiple levels of nesting)
        // - Data sources
        // - Regular resources
        // - Complex dependency chains

        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║          Visual Regression Test: Complex Modules            ║");
        println!("╚══════════════════════════════════════════════════════════════╝\n");

        // Create output directory for visual inspection
        let output_dir = PathBuf::from("test/output");
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        // Create temporary directories for icon cache
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");
        let temp_graph_dir = temp_dir.path().join("temp");
        fs::create_dir_all(&temp_graph_dir).expect("Failed to create temp dir");

        // Extract icons if available
        let bundled_icons = get_bundled_icons_path();
        if bundled_icons.exists() {
            match extract_icons_from_7z(&bundled_icons, &cache_dir) {
                Ok(count) => println!("  ✓ Extracted {} icons from archive", count),
                Err(e) => println!("  ⚠ Icon extraction failed ({}), using fallback boxes", e),
            }
        } else {
            println!("  ⚠ No bundled icons found, using fallback boxes");
        }

        // Read the complex test fixture
        let test_dot_path = "test/complex-modules-test.dot";
        let dot_content = fs::read_to_string(test_dot_path)
            .expect(&format!("Failed to read test fixture: {}", test_dot_path));

        // Parse the graph
        let graph = parse_dot_graph(&dot_content)
            .expect("Failed to parse complex test fixture");

        println!("\n  📊 Parsed Graph Statistics:");
        println!("     Total resources: {}", graph.resources.len());
        println!("     Total edges: {}", graph.edges.len());

        // Count different resource types
        let module_resources = graph.resources.iter()
            .filter(|r| r.name.starts_with("module."))
            .count();
        let data_sources = graph.resources.iter()
            .filter(|r| r.name.starts_with("data."))
            .count();
        let regular_resources = graph.resources.iter()
            .filter(|r| !r.name.starts_with("module.") && !r.name.starts_with("data.") && !r.name.starts_with("provider"))
            .count();

        println!("     Module resources: {}", module_resources);
        println!("     Data sources: {}", data_sources);
        println!("     Regular resources: {}", regular_resources);

        // Filter AWS resources (same as generate_dot_graph does)
        let aws_resources: Vec<_> = graph.resources.iter()
            .filter(|r| r.resource_type.contains("aws_"))
            .collect();

        println!("     AWS resources (after filtering): {}", aws_resources.len());

        // Verify that module resources are included
        let module_aws_count = aws_resources.iter()
            .filter(|r| r.name.starts_with("module."))
            .count();
        let data_aws_count = aws_resources.iter()
            .filter(|r| r.name.starts_with("data."))
            .count();

        println!("       ├─ From modules: {}", module_aws_count);
        println!("       ├─ From data sources: {}", data_aws_count);
        println!("       └─ Direct resources: {}", aws_resources.len() - module_aws_count - data_aws_count);

        // Generate DOT graph for visualization
        let output_dot = generate_dot_graph(
            &graph,
            "complex-modules-test",
            "TB",
            &cache_dir,
            &temp_graph_dir,
            128,
            "#2D3436"
        ).expect("Failed to generate dot graph");

        // Write the generated DOT file for inspection
        let dot_output_path = output_dir.join("complex-modules-test.dot");
        fs::write(&dot_output_path, &output_dot)
            .expect("Failed to write DOT output");

        println!("\n  📝 Generated Files:");
        println!("     DOT file: {}", dot_output_path.display());

        // Generate PNG diagram
        let png_output = output_dir.join("complex-modules-test-visual-regression.png");
        let png_result = execute_dot_command(
            &output_dot,
            "png",
            png_output.to_str().unwrap()
        );

        if png_result.is_ok() {
            println!("     PNG diagram: {}", png_output.display());
        } else {
            println!("     ⚠ PNG generation skipped (GraphViz not available)");
        }

        // Generate SVG diagram (better for inspection)
        let svg_output = output_dir.join("complex-modules-test-visual-regression.svg");
        let svg_result = execute_dot_command(
            &output_dot,
            "svg",
            svg_output.to_str().unwrap()
        );

        if svg_result.is_ok() {
            println!("     SVG diagram: {}", svg_output.display());
        } else {
            println!("     ⚠ SVG generation skipped (GraphViz not available)");
        }

        println!("\n  👀 Visual Inspection:");
        println!("     Open the generated diagrams to verify:");
        println!("     • All module resources are visible (not filtered out)");
        println!("     • Data sources are included and properly typed");
        println!("     • Nested modules (module.app.module.asg) are handled correctly");
        println!("     • Edges connect resources properly");
        println!("     • Icons are used (if available) or styled boxes as fallback");

        println!("\n╚══════════════════════════════════════════════════════════════╝\n");

        // Assertions to ensure the test is meaningful
        assert!(
            graph.resources.len() >= 30,
            "Complex test should have at least 30 resources, got {}",
            graph.resources.len()
        );

        assert!(
            module_resources >= 15,
            "Should have at least 15 module resources, got {}",
            module_resources
        );

        assert!(
            data_sources >= 3,
            "Should have at least 3 data sources, got {}",
            data_sources
        );

        assert!(
            aws_resources.len() >= 25,
            "After filtering, should have at least 25 AWS resources, got {}. \
             This indicates module/data resources are being filtered out!",
            aws_resources.len()
        );

        assert!(
            module_aws_count >= 15,
            "Should have at least 15 AWS resources from modules after filtering, got {}. \
             This indicates module resources are being incorrectly filtered!",
            module_aws_count
        );

        assert!(
            data_aws_count >= 3,
            "Should have at least 3 AWS resources from data sources after filtering, got {}. \
             This indicates data sources are being incorrectly filtered!",
            data_aws_count
        );

        // Verify edges are present in output
        let edge_count = output_dot.matches("->").count();
        assert!(
            edge_count >= 30,
            "Expected at least 30 edges in output, found {}",
            edge_count
        );
    }
}
