use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Create icons directory if it doesn't exist
    let out_dir = env::var("OUT_DIR").unwrap();
    let icons_dir = Path::new(&out_dir).join("icons");
    fs::create_dir_all(&icons_dir).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons/");

    // Capture build metadata for version command
    capture_build_metadata();

    // Icons will be embedded from the icons/ directory
    // Users should download AWS icons and place them there
    // For now, we'll generate a manifest of available icons

    let icon_names = vec![
        "ec2",
        "lambda",
        "ecs",
        "eks",
        "autoscaling",
        "rds",
        "dynamodb",
        "elasticache",
        "redshift",
        "elb",
        "vpc",
        "subnet",
        "route53",
        "cloudfront",
        "apigateway",
        "s3",
        "ebs",
        "efs",
        "iam",
        "kms",
        "sns",
        "sqs",
        "kinesis",
    ];

    // Generate Rust code for icon embedding
    let mut code = String::from("// Auto-generated icon data\n\n");
    code.push_str("pub fn get_embedded_icons() -> HashMap<&'static str, &'static [u8]> {\n");
    code.push_str("    let icons = HashMap::new();\n");

    for name in icon_names {
        let icon_path = format!("icons/{}.png", name);
        if Path::new(&icon_path).exists() {
            code.push_str(&format!(
                "    icons.insert(\"{}\", include_bytes!(\"../{}\"));\n",
                name, icon_path
            ));
        }
    }

    code.push_str("    icons\n");
    code.push_str("}\n");

    fs::write(Path::new(&out_dir).join("icons.rs"), code).unwrap();
}

fn capture_build_metadata() {
    // Capture git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Capture git branch
    let git_branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check if working directory is clean
    let git_dirty = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map(|status| !status.success())
        .unwrap_or(false);

    // Capture build timestamp
    let build_time = chrono::Utc::now().to_rfc3339();

    // Capture target triple
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    // Capture profile (debug/release)
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());

    // Set environment variables for use in main.rs
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    println!(
        "cargo:rustc-env=GIT_DIRTY={}",
        if git_dirty { "true" } else { "false" }
    );
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    println!("cargo:rustc-env=BUILD_TARGET={}", target);
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

    // Rerun build script if git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
