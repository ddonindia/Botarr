use std::process::Command;

fn main() {
    // Only build frontend in release mode or if requested explicitly
    // to avoid slowing down dev builds unnecessarily,
    // BUT user asked for "cargo build ... should build we automatically", so we do it.
    // We can check profile if we want, but let's just do it.

    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");
    println!("cargo:rerun-if-changed=web/index.html");

    // Skip npm build if:
    // 1. CROSS_COMPILE is set (cross-compilation environment)
    // 2. SKIP_WEB_BUILD is set
    // 3. web/dist already exists (pre-built by CI)
    if std::env::var("CROSS_COMPILE").is_ok()
        || std::env::var("SKIP_WEB_BUILD").is_ok()
        || std::path::Path::new("web/dist/index.html").exists()
    {
        println!("cargo:warning=Skipping web build (pre-built or cross-compile)");
        return;
    }

    let _is_release = std::env::var("PROFILE").unwrap() == "release";

    #[cfg(windows)]
    let npm_cmd = "npm.cmd";
    #[cfg(not(windows))]
    let npm_cmd = "npm";

    // 1. Install dependencies
    let status = Command::new(npm_cmd)
        .arg("install")
        .current_dir("web")
        .status()
        .expect("Failed to run npm install");

    if !status.success() {
        panic!("npm install failed");
    }

    // 2. Build frontend
    let status = Command::new(npm_cmd)
        .arg("run")
        .arg("build")
        .current_dir("web")
        .status()
        .expect("Failed to run npm run build");

    if !status.success() {
        panic!("npm run build failed");
    }
}
