# Agedashi

*Making tofu pretty!* 🍲

A Rust CLI utility that transforms Terraform/OpenTofu graph output into beautiful infrastructure diagrams.

> **⚠️ BETA STATUS**
> This project is currently in active development (beta). While core functionality is stable, the API and features may change.
>
> **For developers:**
> - **Active development** happens on the `develop` branch
> - **Latest stable code** is on the `main` branch
> - To contribute or test cutting-edge features, please check out the `develop` branch
> - For production use, we recommend waiting for the 1.0 release or testing thoroughly in your environment

## Overview

Agedashi is a Rust command-line tool that reads Terraform/OpenTofu graph output and automatically generates visual architecture diagrams. It focuses on AWS resources and creates professional-looking infrastructure diagrams with minimal effort.

**Why "Agedashi"?** Like the Japanese dish where tofu is deep-fried to perfection, Agedashi takes your infrastructure-as-code (tofu/OpenTofu) and makes it visually appealing!

## Sample Output

Here's an example of what Agedashi generates from a Terraform graph:

![Sample Infrastructure Diagram](samples/sample-infrastructure.png)

The diagram shows AWS resources with official AWS Architecture Icons, clean layouts, and automatic relationship mapping. Icons have rounded corners for a modern appearance, and resource names appear below each icon.

## Features

- **Built with Rust**: Fast, reliable, and memory-safe
- **Simple CLI Interface**: Just pipe your OpenTofu/Terraform graph output directly to agedashi
- **Multiple Output Formats**: Generate diagrams in SVG, PNG, PDF, or JPG
- **AWS Resource Mapping**: Automatically maps 25+ AWS resource types to official AWS icons
- **Official AWS Icons**: Uses AWS Architecture Icons from official AWS repository
- **Customizable Layout**: Control diagram direction (top-to-bottom or left-to-right)
- **No Configuration Required**: Works out of the box with sensible defaults
- **Transparent Backgrounds**: Generated diagrams work on light and dark themes

## Prerequisites

### Runtime Requirements

**GraphViz** is required to render diagrams:
- **macOS**: `brew install graphviz`
- **Ubuntu/Debian**: `sudo apt-get install graphviz`
- **Fedora**: `sudo dnf install graphviz`
- **Windows**: Download from [graphviz.org](https://graphviz.org/download/)

### Build Requirements (if building from source)

**Rust** toolchain:
- Install from [rustup.rs](https://rustup.rs/)

## Installation

> **Note:** Agedashi is currently in beta. There are no published package releases yet. Installation is currently only available by building from source.

### From Source (Recommended for Beta)

For the latest stable code:
```bash
git clone https://github.com/ltomes/Agedashi.git
cd agedashi
git checkout main
cargo build --release
sudo cp target/release/agedashi /usr/local/bin/
```

For the latest development features (bleeding edge):
```bash
git clone https://github.com/ltomes/Agedashi.git
cd agedashi
git checkout develop
cargo build --release
sudo cp target/release/agedashi /usr/local/bin/
```

### Future Installation Options

Once Agedashi reaches v1.0, we plan to provide:
- Pre-compiled binaries for Linux, macOS, and Windows
- Installation via cargo: `cargo install agedashi`
- Homebrew formula for macOS
- Package manager support for major Linux distributions

## Usage

### Basic Usage

The simplest way to use agedashi is to pipe the output of `tofu graph` directly to it:

```bash
tofu graph | agedashi
```

This will generate a PNG file named `infrastructure.png` in your current directory.

### Command-Line Options

```bash
agedashi [OPTIONS]
```

#### Options:

- `-o, --output <FORMAT>` - Output format (svg, png, pdf, jpg) [default: png]
- `-n, --name <NAME>` - Output file name without extension [default: infrastructure]
- `-d, --direction <DIR>` - Graph direction: TB (top-to-bottom) or LR (left-to-right) [default: TB]
- `-h, --help` - Print help information
- `-V, --version` - Print version information

### Examples

Generate an SVG diagram:
```bash
tofu graph | agedashi --output svg
```

Create a diagram with a custom name:
```bash
tofu graph | agedashi --name my-infrastructure --output png
```

Create a left-to-right diagram in PDF format:
```bash
tofu graph | agedashi --direction LR --output pdf --name aws-architecture
```

Save the graph output first, then process it:
```bash
tofu graph > graph.dot
cat graph.dot | agedashi --output svg --name production
```

## Supported AWS Resources

Agedashi currently maps the following AWS resource types to diagram icons:

### Compute
- `aws_instance` → EC2
- `aws_lambda_function` → Lambda
- `aws_ecs_*` → ECS
- `aws_eks_*` → EKS
- `aws_autoscaling_*` → AutoScaling

### Database
- `aws_db_instance` → RDS
- `aws_dynamodb_table` → DynamoDB
- `aws_elasticache_*` → ElastiCache
- `aws_redshift_*` → Redshift

### Network
- `aws_elb`, `aws_lb` → ELB (Load Balancer)
- `aws_vpc` → VPC
- `aws_subnet` → PublicSubnet
- `aws_route53_*` → Route53
- `aws_cloudfront_*` → CloudFront
- `aws_api_gateway_*` → API Gateway

### Storage
- `aws_s3_bucket` → S3
- `aws_ebs_*` → EBS
- `aws_efs_*` → EFS

### Security
- `aws_iam_*` → IAM
- `aws_security_group` → SecurityGroup
- `aws_kms_*` → KMS

### Integration
- `aws_sns_*` → SNS
- `aws_sqs_*` → SQS

### Analytics
- `aws_kinesis_*` → Kinesis

Resources not explicitly mapped will default to EC2 icons.

## Example Workflow

1. Navigate to your OpenTofu project:
   ```bash
   cd /path/to/opentofu/project
   ```

2. Initialize OpenTofu (if not already done):
   ```bash
   tofu init
   ```

3. Generate and visualize your infrastructure:
   ```bash
   tofu graph | agedashi --output svg --name my-infra
   ```

4. Open the generated diagram:
   ```bash
   # macOS
   open my-infra.svg

   # Linux
   xdg-open my-infra.svg

   # Windows
   start my-infra.svg
   ```

## How It Works

1. **Input**: Agedashi reads Terraform graph output (in DOT format) from stdin
2. **Parse**: Extracts resource nodes and their relationships using Rust regex
3. **Map**: Maps Terraform resource types to official AWS Architecture Icons
4. **Convert**: Converts cached SVG icons to PNG with rounded corners
5. **Generate**: Creates enhanced GraphViz DOT format with embedded icons and styling
6. **Render**: Invokes GraphViz `dot` command to render the final diagram
7. **Post-process**: For SVG output, embeds images as base64 for portability
8. **Output**: Saves the diagram in your specified format (PNG/SVG/PDF/JPG)

The core logic is written in Rust for speed and reliability, with GraphViz handling the final layout and rendering.

## Examples Directory

Check out the `examples/` directory for sample Terraform configurations:

- `simple-aws.tf` - Basic AWS setup with VPC, EC2, RDS, and S3
- `complex-aws.tf` - Advanced multi-tier architecture with 20+ services

To test with examples:

```bash
cd examples
tofu init
tofu graph | ../target/release/agedashi --output png --name example
```

## Troubleshooting

### "GraphViz 'dot' command not found"

Agedashi requires GraphViz to render diagrams. Install it for your platform:

```bash
# macOS
brew install graphviz

# Ubuntu/Debian
sudo apt-get install graphviz

# Fedora
sudo dnf install graphviz

# Windows
# Download from https://graphviz.org/download/
```

### "No AWS resources found"

This means your Terraform graph doesn't contain any AWS resources, or they're not being parsed correctly. Ensure you're using AWS provider resources in your Terraform configuration.

### Debug Mode

Set the `AGEDASHI_DEBUG` environment variable to save the generated DOT file:

```bash
AGEDASHI_DEBUG=1 tofu graph | agedashi --output png
```

This will save `infrastructure.dot` so you can inspect the generated GraphViz code.

### Build errors

Make sure you have the latest Rust toolchain:
```bash
rustup update stable
```

## Contributing

Contributions are welcome! Here are some ways you can help:

- Add support for more AWS resources
- Add support for other cloud providers (Azure, GCP)
- Improve the DOT parser
- Add clustering/grouping features
- Improve error messages

### Development Workflow

1. Fork the repository
2. Clone your fork locally
3. **Create a feature branch from `develop`** (not `main`)
4. Make your changes and add tests
5. Ensure tests pass and code builds
6. Submit a pull request to the `develop` branch

The `develop` branch is where active development happens. The `main` branch contains the latest stable code.

## Roadmap

- [ ] Support for Azure and GCP resources
- [ ] Automatic resource grouping by VPC/subnet
- [ ] Interactive HTML output
- [ ] Cost estimation annotations
- [ ] Security group rule visualization
- [ ] Custom resource icon mapping via config file

## License

MIT License - see LICENSE file for details

## Credits

- Built with [Rust](https://www.rust-lang.org/)
- Visualization powered by [GraphViz](https://graphviz.org/)
- Official [AWS Architecture Icons](https://aws.amazon.com/architecture/icons/)
- CLI parsing with [clap](https://github.com/clap-rs/clap)
- SVG rendering with [resvg](https://github.com/RazrFalcon/resvg)

## Author

Created with passion for infrastructure as code and beautiful diagrams.
