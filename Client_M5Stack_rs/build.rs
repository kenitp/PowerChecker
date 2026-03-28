use std::path::PathBuf;

fn main() {
    // ── Load .env (if present) into this process env ─────────────────────────
    // `dotenv` does not overwrite variables already set in the shell.
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let env_path = manifest_dir.join(".env");
    if env_path.is_file() {
        let _ = dotenv::from_path(&env_path);
        println!("cargo:rerun-if-changed=.env");
    }

    // Pass through to rustc for `env!()` in main.rs (compile-time embedding).
    const REQUIRED: &[&str] = &["WIFI_SSID", "WIFI_PASS", "POWER_CHECKER_URL"];
    for &var in REQUIRED {
        match std::env::var(var) {
            Ok(val) if !val.is_empty() => {
                println!("cargo:rustc-env={}={}", var, val);
            }
            _ => {
                eprintln!(
                    "WARNING: {} is not set. Use .env (see .env.example) or export before build.",
                    var
                );
            }
        }
    }

    // ── Slint UI compilation ──────────────────────────────────────────────────
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
    slint_build::print_rustc_flags().unwrap();
}
