use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Create icons directory if it doesn't exist
    let out_dir = env::var("OUT_DIR").unwrap();
    let icons_dir = Path::new(&out_dir).join("icons");
    fs::create_dir_all(&icons_dir).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons/");

    // Icons will be embedded from the icons/ directory
    // Users should download AWS icons and place them there
    // For now, we'll generate a manifest of available icons

    let icon_names = vec![
        "ec2", "lambda", "ecs", "eks", "autoscaling",
        "rds", "dynamodb", "elasticache", "redshift",
        "elb", "vpc", "subnet", "route53", "cloudfront", "apigateway",
        "s3", "ebs", "efs",
        "iam", "kms",
        "sns", "sqs",
        "kinesis",
    ];

    // Generate Rust code for icon embedding
    let mut code = String::from("// Auto-generated icon data\n\n");
    code.push_str("pub fn get_embedded_icons() -> HashMap<&'static str, &'static [u8]> {\n");
    code.push_str("    let icons = HashMap::new();\n");

    for name in icon_names {
        let icon_path = format!("icons/{}.png", name);
        if Path::new(&icon_path).exists() {
            code.push_str(&format!("    icons.insert(\"{}\", include_bytes!(\"../{}\"));\n", name, icon_path));
        }
    }

    code.push_str("    icons\n");
    code.push_str("}\n");

    fs::write(Path::new(&out_dir).join("icons.rs"), code).unwrap();
}
