const KEYS: [(&str, bool); 5] = [
    ("AUTH0_DOMAIN", true),
    ("AUTH0_CLIENT_ID", true),
    ("AUTH0_AUDIENCE", true),
    ("SUPPORT_API_URL", true),
    ("CDN_WORKER_URL", false),
];

// only pull env when built for wasm, else just disregard
fn main() {
    init();
}

#[cfg(not(target_arch = "wasm32"))]
fn init() {}

#[cfg(target_arch = "wasm32")]
fn init() {
    println!("cargo:rerun-if-changed=.kivro.toml");

    let mut missing = Vec::new();

    for (key, required) in KEYS {
        println!("cargo:rerun-if-env-changed={key}");

        let value = std::env::var(key).unwrap_or_default();
        if value.is_empty() && required {
            missing.push(key);
        }
        println!("cargo:rustc-env={key}={value}");
    }

    if !missing.is_empty() {
        println!(
            "cargo:warning=missing {} — the build will run signed out and the \
               features backed by those endpoints will fail. Provide them with \
               `kivro run -- trunk build` (see .kivro.toml), or export them yourself.",
            missing.join(", "),
        );
    }
}
