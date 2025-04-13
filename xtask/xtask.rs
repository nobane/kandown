// xtask/xtask.rs
use std::{
    env,
    fs, // Added for directory creation/removal
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

// --- Configuration ---
const OUTPUT_DIR: &str = "web/bindgen";
const TYPES_FILENAME: &str = "kandown.ts";
const WASM_CRATE_NAME: &str = "kandown-wasm";
const TYPES_GENERATOR_BIN: &str = "generate-ts-types"; // Added for clarity

// --- Error Handling ---
type Result<T> = std::result::Result<T, String>;

// --- Main Task Dispatch ---
fn main() -> ExitCode {
    // Determine project root first
    let root = match find_project_root() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error finding project root: {}", e);
            return ExitCode::FAILURE;
        }
    };
    println!("Operating from project root: {}", root.display());

    // Basic argument parsing
    let args: Vec<String> = env::args().collect();
    let task = args.get(1).map(|s| s.as_str());

    // Dispatch task
    let result = match task {
        Some("build") => build_all(&root),
        Some("build-wasm") => build_wasm(&root),
        Some("build-types") => build_types(&root),
        Some("clean") => clean(&root),
        Some("watch") => watch(&root), // Added watch task
        _ => print_help(),
    };

    // Handle task result
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

// --- Helper Functions ---

/// Finds the project root containing `Cargo.toml` by searching upwards
/// from the directory containing the currently running executable.
fn find_project_root() -> Result<PathBuf> {
    let current_exe =
        env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))?;
    // Try CARGO_MANIFEST_DIR first, it's usually more reliable if run via `cargo xtask`
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let potential_root = PathBuf::from(manifest_dir).join("../.."); // Assumes xtask is in ./xtask
        if potential_root.join("Cargo.toml").is_file() {
            println!("Using CARGO_MANIFEST_DIR to find project root.");
            return fs::canonicalize(potential_root)
                .map_err(|e| format!("Failed to canonicalize path: {}", e));
        }
    }

    // Fallback to searching upwards from executable
    println!("CARGO_MANIFEST_DIR not found or unusable, searching from executable path.");
    let mut current_dir = current_exe
        .parent()
        .ok_or_else(|| format!("Executable path has no parent directory: {current_exe:?}",))?
        .to_path_buf();

    loop {
        let marker_path = current_dir.join("Cargo.toml");
        if marker_path.is_file() {
            println!("Found project root marker at: {}", marker_path.display());
            // Canonicalize to get a clean, absolute path
            return fs::canonicalize(current_dir)
                .map_err(|e| format!("Failed to canonicalize path: {}", e));
        }

        if let Some(parent) = current_dir.parent() {
            if parent == current_dir {
                return Err(format!(
                    "Reached filesystem root without finding Cargo.toml starting from {}",
                    current_exe.display()
                ));
            }
            current_dir = parent.to_path_buf();
        } else {
            return Err(format!(
                "Reached filesystem root without finding Cargo.toml starting from {}",
                current_exe.display()
            ));
        }
    }
}

/// Runs a command, inheriting stdio and returning an error string on failure.
fn run_command(program: &str, args: &[&str], current_dir: &Path) -> Result<()> {
    println!(
        "Running command: {} {} (in {})",
        program,
        args.join(" "),
        current_dir.display()
    );
    let status = Command::new(program)
        .args(args)
        .current_dir(current_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to execute command '{}': {}", program, e))?;

    if status.success() {
        Ok(())
    } else {
        // Check for specific 'command not found' scenario for cargo-watch
        if program == "cargo" && args.first() == Some(&"watch") && status.code().is_none() {
            return Err(format!(
                "Command '{} {}' failed. '{}' might not be installed. Try running: `cargo install cargo-watch`",
                program,
                args.join(" "),
                "cargo-watch"
            ));
        }
        Err(format!(
            "Command '{} {}' failed with exit code: {:?}",
            program,
            args.join(" "),
            status.code()
        ))
    }
}

// --- Task Implementations ---

fn print_help() -> Result<()> {
    println!("Usage: cargo xtask <task>");
    println!("Available tasks:");
    println!(
        "  build         Build Wasm ({}) and generate TypeScript types",
        WASM_CRATE_NAME
    );
    println!(
        "  build-wasm    Run wasm-pack build for {}",
        WASM_CRATE_NAME
    );
    println!("  build-types   Generate TypeScript types (requires output dir)",);
    println!(
        "  watch         Watch project files and run 'build' on changes (requires cargo-watch)"
    );
    println!(
        "  clean         Remove the output directory ({})",
        OUTPUT_DIR
    );
    Ok(())
}

fn build_all(root: &Path) -> Result<()> {
    println!("--- Running Full Build Task ---");
    // It's often better to build types *first* if the wasm build depends on them,
    // but in this case wasm-pack generates JS bindings that the types might augment.
    // Let's stick to wasm -> types order unless there's a specific dependency.
    build_wasm(root)?;
    build_types(root)?;
    println!("--- Full Build Finished Successfully ---");
    Ok(())
}

fn build_wasm(root: &Path) -> Result<()> {
    println!("--- Building Wasm ({}) ---", WASM_CRATE_NAME);
    let wasm_crate_path = root.join("crates").join(WASM_CRATE_NAME);
    let output_path = root.join(OUTPUT_DIR);

    // Ensure paths exist for clarity in wasm-pack command
    if !wasm_crate_path.exists() {
        return Err(format!(
            "Wasm crate path not found: {}",
            wasm_crate_path.display()
        ));
    }
    // Output dir will be created by wasm-pack if needed, but we create it for build_types later anyway.
    fs::create_dir_all(&output_path).map_err(|e| {
        format!(
            "Failed to create output directory {}: {}",
            output_path.display(),
            e
        )
    })?;

    let wasm_pack_args = &[
        "build",
        &wasm_crate_path.display().to_string(), // Use path relative to root
        "--target",
        "web",
        "--out-dir",
        &output_path.display().to_string(), // Use path relative to root
                                            // Optional: Add --dev for faster debug builds during watch
                                            // "--dev",
    ];

    // Run wasm-pack from the project root
    run_command("wasm-pack", wasm_pack_args, root)?;
    println!("--- Wasm Build Succeeded ---");
    Ok(())
}

fn build_types(root: &Path) -> Result<()> {
    println!("--- Generating TypeScript Types ---");
    let output_dir = root.join(OUTPUT_DIR);
    let output_file = output_dir.join(TYPES_FILENAME);

    // Ensure the output directory exists (wasm-pack should create it, but double-check)
    if !output_dir.exists() {
        // Attempt to create it if missing
        println!(
            "Output directory '{}' does not exist, attempting to create...",
            output_dir.display()
        );
        fs::create_dir_all(&output_dir).map_err(|e| {
            format!(
                "Failed to create output directory {}: {}",
                output_dir.display(),
                e
            )
        })?;
        // return Err(format!(
        //     "Output directory '{}' does not exist. Run 'build-wasm' first or check permissions.",
        //     output_dir.display()
        // ));
    }

    let cargo_path = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output_arg = output_file.display().to_string(); // Path relative to root for the argument

    // Arguments for `cargo run --bin ... -- <output_file>`
    let cargo_args = &[
        "run",
        "--bin",
        TYPES_GENERATOR_BIN,
        "--",
        &output_arg, // Pass the relative path to the binary
    ];

    // Run `cargo run` from the project root
    run_command(&cargo_path, cargo_args, root)?;
    println!(
        "--- TypeScript Types Generation Succeeded ({}) ---",
        output_file.display()
    );
    Ok(())
}

fn clean(root: &Path) -> Result<()> {
    let output_path = root.join(OUTPUT_DIR);
    println!(
        "--- Cleaning Output Directory ({}) ---",
        output_path.display()
    );

    if output_path.exists() {
        fs::remove_dir_all(&output_path).map_err(|e| {
            format!(
                "Failed to remove directory {}: {}",
                output_path.display(),
                e
            )
        })?;
        println!("Successfully removed '{}'.", output_path.display());
    } else {
        println!(
            "Directory '{}' does not exist, nothing to clean.",
            output_path.display()
        );
    }
    Ok(())
}

fn path_str(path: PathBuf) -> String {
    path.display().to_string()
}

// --- New Watch Task ---
fn watch(root: &Path) -> Result<()> {
    println!("--- Starting Watch Task (requires cargo-watch) ---");
    println!("Watching for changes in workspace crates...");
    println!("Will run 'cargo xtask build' on change.");
    println!("Press Ctrl+C to stop.");

    // Determine the path to the current xtask executable or use `cargo xtask`

    // Command to run: cargo watch -c -x "cargo xtask build"
    // -c: Clear screen before each run
    // -x: Execute the command
    // We run `cargo watch` from the project root, so it watches the entire workspace by default.
    let watch_args = &[
        "watch",
        "--why", // Print succinct reason for rebuild
        "-c",    // Clear screen on change
        "-x",    // Execute command
        "xtask build",
        "-w",
        &path_str(root.join("./crates/kandown-wasm")),
        "-w",
        &path_str(root.join("./crates/kandown")),
    ];

    // Run `cargo watch` from the project root
    run_command("cargo", watch_args, root) // Use "cargo" as the program
        .map_err(|e| {
            // Add specific hint if cargo-watch seems missing
            if e.contains("might not be installed") {
                e // Return the specific error from run_command
            } else {
                format!(
                    "{} \nMaybe 'cargo-watch' is not installed? Try: `cargo install cargo-watch`",
                    e
                )
            }
        })
}
