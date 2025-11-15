use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");

    // Try debug first (most common for tests), then release
    let debug_path = path.join("debug").join("agedashi");
    let release_path = path.join("release").join("agedashi");

    #[cfg(windows)]
    let (debug_path, release_path) = {
        (
            debug_path.with_extension("exe"),
            release_path.with_extension("exe"),
        )
    };

    // Return the path that exists, preferring debug
    if debug_path.exists() {
        debug_path
    } else if release_path.exists() {
        release_path
    } else {
        // Default to debug if neither exists (will fail with better error)
        debug_path
    }
}

/// Helper to run agedashi with given input and arguments
fn run_agedashi(input: &str, args: &[&str]) -> std::process::Output {
    let binary = get_binary_path();
    let mut cmd = Command::new(&binary);
    cmd.args(args);

    // Write input to stdin
    use std::io::Write;
    use std::process::Stdio;

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn agedashi process");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    child
        .wait_with_output()
        .expect("Failed to wait for agedashi")
}

#[test]
fn test_help_command() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute agedashi --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("agedashi"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--direction"));
}

#[test]
fn test_version_command() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute agedashi --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("agedashi"));
}

#[test]
fn test_empty_input() {
    let output = run_agedashi("", &[]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No input provided"));
}

#[test]
fn test_sample_graph_png_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_name = "test-integration-png";

    // Read sample graph
    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    // Run in temp directory
    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let output = run_agedashi(&sample_graph, &["--output", "png", "--name", output_name]);

    // Check success
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("agedashi failed to generate PNG");
    }

    // Verify output file exists
    let output_file = temp_dir.path().join(format!("{}.png", output_name));
    assert!(output_file.exists(), "PNG file was not created");

    // Verify file is not empty
    let metadata = fs::metadata(&output_file).expect("Failed to get file metadata");
    assert!(metadata.len() > 0, "PNG file is empty");
}

#[test]
fn test_sample_graph_svg_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_name = "test-integration-svg";

    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let output = run_agedashi(&sample_graph, &["--output", "svg", "--name", output_name]);

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("agedashi failed to generate SVG");
    }

    let output_file = temp_dir.path().join(format!("{}.svg", output_name));
    assert!(output_file.exists(), "SVG file was not created");

    // Verify SVG content
    let svg_content = fs::read_to_string(&output_file).expect("Failed to read SVG");
    assert!(
        svg_content.contains("<svg"),
        "SVG file doesn't contain valid SVG markup"
    );
}

#[test]
fn test_direction_parameter() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    // Test TB direction
    let output_tb = run_agedashi(
        &sample_graph,
        &["--output", "png", "--name", "test-tb", "--direction", "TB"],
    );
    assert!(output_tb.status.success(), "TB direction failed");
    assert!(temp_dir.path().join("test-tb.png").exists());

    // Test LR direction
    let output_lr = run_agedashi(
        &sample_graph,
        &["--output", "png", "--name", "test-lr", "--direction", "LR"],
    );
    assert!(output_lr.status.success(), "LR direction failed");
    assert!(temp_dir.path().join("test-lr.png").exists());
}

#[test]
fn test_all_output_formats() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    for format in &["png", "svg", "pdf", "jpg"] {
        let output = run_agedashi(
            &sample_graph,
            &["--output", format, "--name", &format!("test-{}", format)],
        );

        if !output.status.success() {
            eprintln!("Failed format: {}", format);
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Format {} failed", format);
        }

        let output_file = temp_dir.path().join(format!("test-{}.{}", format, format));
        assert!(output_file.exists(), "{} file was not created", format);

        let metadata = fs::metadata(&output_file).expect("Failed to get metadata");
        assert!(metadata.len() > 0, "{} file is empty", format);
    }
}

#[test]
fn test_complex_modules() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Skip if complex modules test file doesn't exist
    let complex_modules_path = "test/complex-modules-test.dot";
    if !PathBuf::from(complex_modules_path).exists() {
        eprintln!("Skipping test: {} not found", complex_modules_path);
        return;
    }

    let complex_graph =
        fs::read_to_string(complex_modules_path).expect("Failed to read complex-modules-test.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let output = run_agedashi(
        &complex_graph,
        &["--output", "png", "--name", "test-complex-modules"],
    );

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Complex modules test failed");
    }

    let output_file = temp_dir.path().join("test-complex-modules.png");
    assert!(
        output_file.exists(),
        "Complex modules diagram was not created"
    );
}

#[test]
fn test_custom_color() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let output = run_agedashi(
        &sample_graph,
        &[
            "--output",
            "png",
            "--name",
            "test-color",
            "--color",
            "#FF0000",
        ],
    );

    assert!(output.status.success(), "Custom color test failed");
    assert!(temp_dir.path().join("test-color.png").exists());
}

#[test]
fn test_large_graph_performance() {
    use std::time::Instant;

    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    // Repeat the graph content to simulate a larger infrastructure
    let large_graph = sample_graph.repeat(5);

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let start = Instant::now();
    let output = run_agedashi(&large_graph, &["--output", "png", "--name", "test-large"]);
    let duration = start.elapsed();

    assert!(output.status.success(), "Large graph processing failed");

    // Should complete in reasonable time (adjust as needed)
    assert!(
        duration.as_secs() < 30,
        "Large graph took too long: {:?}",
        duration
    );
}

#[test]
fn test_svg_base64_embedding() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let output = run_agedashi(&sample_graph, &["--output", "svg", "--name", "test-base64"]);

    assert!(output.status.success(), "SVG generation failed");

    let svg_file = temp_dir.path().join("test-base64.svg");
    let svg_content = fs::read_to_string(&svg_file).expect("Failed to read SVG file");

    // Check that images are embedded as base64 data URIs
    assert!(
        svg_content.contains("data:image/png;base64,") || !svg_content.contains("xlink:href"),
        "SVG should contain base64-encoded images or no image references"
    );
}

#[test]
fn test_debug_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    // Set debug environment variable
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .args(&["--output", "png", "--name", "test-debug"])
        .env("AGEDASHI_DEBUG", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(sample_graph.as_bytes())?;
            }
            child.wait_with_output()
        })
        .expect("Failed to run agedashi in debug mode");

    assert!(output.status.success(), "Debug mode failed");

    // In debug mode, a .dot file should be created
    let dot_file = temp_dir.path().join("test-debug.dot");
    assert!(dot_file.exists(), "Debug .dot file was not created");
}
