use infinity_ui::prelude::*;

use super::prose::{DocCard, Grid2, Term};

pub fn doc() -> Doc {
    Doc::new("getting-started", "Getting Started", "Guide", "Guide")
        .scope("infinity-rs")
        .tagline(
            "Scaffold a project with `infinity-msfs create`, build a working WASM gauge, and ship a native SimConnect client, all from one toolchain that runs on Windows or Linux.",
        )
        .tags([
            "infinity-msfs create",
            "WASM",
            "SimConnect",
            "Cargo",
            "MSFS 2024 SDK",
        ])
        .section(why_rust())
        .section(wasm_limits())
        .section(prerequisites())
        .section(install_toolchain())
        .section(create_project())
        .section(templates())
        .section(what_create_writes())
        .section(linker_flags())
        .section(build_it())
        .section(a_real_gauge())
        .section(native_simconnect())
        .section(build_anatomy())
        .section(next_steps())
}

fn why_rust() -> DocSection {
    DocSection::new("why-rust", "Why Rust?", || {
        view! {
            <P>
                "MSFS gauges and systems run as WebAssembly modules. Both Rust and C++ can target WASM, after releasing a pure C++ aircraft, we switched to Rust purely for the benefits it brings to the development experience and the stability of the resulting gauges."
            </P>

            <List>
                <ListItem>
                    <Term>"Memory safety."</Term>
                    " Aircraft systems are complex, there is no light way to put it. Every sim frame, there is not only system caclulations, but also a lot of inharently unsafe FFI crossings (SimConnect, comm bus, gauge draw calls). Rust cannot automatically fix every issue when interfacing with a C abi like MSFS. However with careful binding, we can leverage Rust for the mass of the systems code. Then unsafe surface in "
                    <InlineCode>"infinity-rs"</InlineCode>
                    " is small, and concentrated in one place. On the other hand, C++ has no guarentees at all, and the unsafe surface is the entire codebase."
                </ListItem>
                <ListItem>
                    <Term>"Performance"</Term>
                    " Rust compiles to the same LLVM IR C++. Generated WASM is competitive with hand-tuned C++ on the workloads gauges care about (math-heavy update loops, buffer marshaling, NanoVG draw calls), all while giving us zero cost abstractions that guarantee safety."
                </ListItem>
                <ListItem>
                    <Term>"First-class WASM target."</Term>
                    " "
                    <InlineCode>"rustup target add wasm32-wasip1"</InlineCode>
                    " and "
                    <InlineCode>"cargo build --target wasm32-wasip1"</InlineCode>
                    " is the entire toolchain story. Rust ships "
                    <em>"three"</em>
                    " wasm targets natively ("
                    <InlineCode>"wasm32-unknown-unknown"</InlineCode>
                    ", "
                    <InlineCode>"wasm32-wasip1"</InlineCode>
                    ", "
                    <InlineCode>"wasm32-wasip2"</InlineCode>
                    ") plus"
                    <InlineCode>"std"</InlineCode>
                    " precompiled for them. C++ requires a separately maintained Emscripten or wasi-sdk install, custom CMake/MSBuild plumbing, and a shadow standard library that lags upstream (C++ 17, ouch)."
                </ListItem>
                <ListItem>
                    <Term>"Library ecosystem."</Term>
                    " Cargo packages are pure-Rust by default, so the overwhelming majority of crates on "
                    <InlineCode>"crates.io"</InlineCode>
                    " work on "
                    <InlineCode>"wasm32-wasip1"</InlineCode>
                    " with no porting effort. "
                    <InlineCode>"serde"</InlineCode>
                    ", "
                    <InlineCode>"nom"</InlineCode>
                    ", "
                    <InlineCode>"bitflags"</InlineCode>
                    ", "
                    <InlineCode>"num-traits"</InlineCode>
                    ", "
                    <InlineCode>"uom"</InlineCode>
                    ", and the rest of the long tail just work. The C++ equivalent, vcpkg libraries that compile to WASM, exist but are hard to come by and often require patches to work in MSFS due to how common C++ exceptions are."
                </ListItem>
                <ListItem>
                    <Term>"Build tooling."</Term>
                    " Cargo is one tool. It handles dependencies, features, profiles, cross-compilation, and tests with no per-project plumbing. Our "
                    <InlineCode>"infinity-msfs"</InlineCode>
                    " CLI sits on top of Cargo and adds scaffolding, wasm-opt, and asset packaging; everything else is stock."
                </ListItem>
                <ListItem>
                    <Term>"Strong typing across the FFI seam."</Term>
                    " bindgen turns the SDK headers into typed Rust automatically. Variable units, SimConnect data definitions, and event payloads stay typed all the way through. This minimises some the common risks of FFI work."
                </ListItem>
            </List>

            <Callout tone=Tone::Accent icon=icons::INFO title="Bottom line">
                "We don't use Rust because it's trendy. Ok, maybe a little bit. But the real reason is that it produces more stable, maintainable, and an overall better experience for our team and our users."
            </Callout>
        }
    })
    .numbered("0.0")
    .summary("Why we standardised on Rust for aircraft systems instead of the C++ template the SDK ships with.")
}

fn wasm_limits() -> DocSection {
    DocSection::new("wasm-limits", "The few limits Rust can't avoid", || {
        view! {
            <P>
                "As for normal operations and release oriented code, we can safely trust Rust here; "
                <InlineCode>"panic!"</InlineCode>
                " is in full effect in the sim meaning you can program a crash and the wasm module will cleanly unload itself without crashing the sim."
            </P>

            <Callout
                tone=Tone::Danger
                icon=icons::SHIELD_ALERT
                title="IEEE-754 arithmatic that results in NaN or Infinity will bypass the panic handler."
            >
                "If a user has "
                <InlineCode>"Wasm Floating Point Exceptions"</InlineCode>
                " enabled in their settings any floating point arithmatic that results in either"
                <InlineCode>"NaN"</InlineCode>
                " or "
                <InlineCode>"Infinity"</InlineCode>
                ", the module will throw an exception and crash, bypassing the "
                <InlineCode>"panic!"</InlineCode>
                " handler."
            </Callout>

            <List>
                <ListItem>
                    <Term>"Everything else is just Rust."</Term>
                    " "
                    <InlineCode>"panic!()"</InlineCode>
                    ", "
                    <InlineCode>"unwrap()"</InlineCode>
                    ", "
                    <InlineCode>"expect()"</InlineCode>
                    ", slice bounds violations, and other runtime checks all fail cleanly without taking down the simulator. Your program will only crash if you tell it to."
                </ListItem>
            </List>

            <Callout tone=Tone::Accent icon=icons::INFO title="Bottom line">
                "In practice, Rust gauges are extremely robust. Aside from the optional floating point traps, mostly every failure is handled predictably, making Rust significantly more reliable than the equivalent C++ implementation."
            </Callout>
        }
    })
    .numbered("0.1")
    .summary("Rust eliminates almost every common crash in MSFS, but a small number of WASM runtime traps still exist. Fortunately, they're easy to avoid and are only relevent to users with FP exceptions on")
}

fn prerequisites() -> DocSection {
    DocSection::new("prerequisites", "Prerequisites", || {
        view! {
            <P>
                <InlineCode>"infinity-rs"</InlineCode>
                " targets "
                <InlineCode>"wasm32-wasip1"</InlineCode>
                " for WASM gauges and systems, and the host triple for native SimConnect clients. The required Rust toolchain plus a couple of host tools is all you need on the box. The MSFS SDK headers and the SimConnect import library are downloaded and cached by "
                <InlineCode>"infinity-msfs sdk install"</InlineCode>
                "."
            </P>

            <List>
                <ListItem>
                    <Term>"Rust 1.85+"</Term>
                    " via "
                    <InlineCode>"rustup"</InlineCode>
                    " (edition 2024 is required)."
                </ListItem>
                <ListItem>
                    <Term>"wasm32-wasip1 target"</Term>
                    " via "
                    <InlineCode>"rustup target add wasm32-wasip1"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>"Binaryen / wasm-opt"</Term>
                    " on PATH (optional but recommended; disable per-project with "
                    <InlineCode>"[rust.wasm_opt] enabled = false"</InlineCode>
                    ")."
                </ListItem>
                <ListItem>
                    <Term>"bun"</Term>
                    " only if you are scaffolding one of the JS/TS instrument templates."
                </ListItem>
                <ListItem>
                    <Term>"MSFS 2024 SDK"</Term>
                    " only for native SimConnect linking, or for "
                    <InlineCode>"infinity-msfs package"</InlineCode>
                    ". WASM-only builds work off the cached SDK subset."
                </ListItem>
            </List>

            <Callout tone=Tone::Accent icon=icons::INFO title="Cross-platform">
                "The full pipeline (including SimConnect bindings) builds on Linux. Native SimConnect "
                <em>"linking"</em>
                " is Windows-only because the SimConnect import library is shipped only for Windows, but bindgen of the SimConnect headers and WASM gauge builds work identically on either OS."
            </Callout>
        }
    })
    .numbered("1.0")
    .summary("Install once. The `infinity-msfs` CLI provisions the SDK for you, so you only need a working Rust toolchain and a couple of host tools.")
}

fn install_toolchain() -> DocSection {
    DocSection::new("install-the-build-tool", "Install the toolchain", || {
        view! {
            <P>
                "The CLI wraps Cargo and the JS bundler, post-processes WASM with "
                <InlineCode>"wasm-opt"</InlineCode>
                ", copies artifacts into your package's expected layout, and (for native packages) bundles "
                <InlineCode>"SimConnect.dll"</InlineCode>
                " next to the produced exe. Project templates are embedded in the binary, so this one install is all you need."
            </P>

            <CodeBlock
                language="bash"
                title="install"
                code=r#"cargo install infinity-msfs
rustup target add wasm32-wasip1"#
            />

            <P>
                "Provision the SDK subset (headers plus the SimConnect import library) into the per-user cache:"
            </P>

            <CodeBlock
                language="bash"
                title="sdk"
                code=r#"infinity-msfs sdk install          # download + cache
infinity-msfs sdk install --force  # re-download
infinity-msfs sdk path             # print the resolved SDK path
infinity-msfs sdk remove           # clear the cache"#
            />

            <P>
                "Verify the whole environment in one shot with "
                <InlineCode>"infinity-msfs doctor"</InlineCode>
                ":"
            </P>

            <CodeBlock
                language="text"
                title="infinity-msfs doctor"
                code=r#"$ infinity-msfs doctor
Checking infinity-msfs build environment

  ✓ cargo           cargo 1.96.0 (30a34c682 2026-05-25)
  ✓ wasm32-wasip1   installed
  ✓ wasm-opt        wasm-opt version 116 (version_116)
  ✓ MSFS SDK        MSFS2024_SDK = C:\MSFS 2024 SDK\
  ✓ SimConnect lib  found at C:\MSFS 2024 SDK\SimConnect SDK\lib

all checks passed"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="MSFS2024_SDK wins">
                "If you have the full SDK installed locally and "
                <InlineCode>"MSFS2024_SDK"</InlineCode>
                " pointed at its root, the CLI uses that instead of the download cache."
            </Callout>
        }
    })
    .numbered("1.1")
    .summary("`infinity-msfs` is a standalone CLI that scaffolds, builds, watches, tests, and packages every kind of MSFS project we ship.")
}

fn create_project() -> DocSection {
    DocSection::new("create-a-project", "Create a project", || {
        view! {
            <P>
                "Don't hand-roll a crate. Run "
                <InlineCode>"infinity-msfs create"</InlineCode>
                " and pick a template. Templates are compiled into the binary, so there is nothing to clone and nothing to keep in sync."
            </P>

            <CodeBlock
                language="bash"
                title="interactive"
                code=r#"infinity-msfs create               # prompts for template, path, and name
infinity-msfs create my-gauge      # path supplied, still prompts for the rest"#
            />

            <P>"The picker comes first, then the path, then the project name:"</P>

            <CodeBlock
                language="text"
                title="the TUI"
                code=r#"? Pick a template ›
  ReScript + React instrument                    — A React-based MSFS instrument written in ReScript…
❯ Rust WASM gauge                                — A cdylib MSFS gauge built to wasm32-wasip1, using infinity-rs.
  Rust WASM system                               — A cdylib MSFS system module built to wasm32-wasip1…
  Rust native (SimConnect)                       — A native host-target Rust binary linked against SimConnect…
  TypeScript + React instrument                  — A plain TypeScript + React MSFS instrument bundled via rolldown.
  TypeScript Avionics Framework (BaseInstrument) — For high-frequency PFD/ND/EFIS gauges.

? Target directory › my-gauge
? Project name (kebab-case) › my-gauge
? Exported gauge symbol (snake_case) › my_gauge

Scaffolding Rust WASM gauge → /path/to/my-gauge
✓ wrote 4 file(s) to /path/to/my-gauge

Next steps:
  cd my-gauge
  infinity-msfs doctor
  infinity-msfs build"#
            />

            <P>
                "Every prompt has a flag, so the same command scripts cleanly in CI. "
                <InlineCode>"--no-input"</InlineCode>
                " makes any missing required field a hard error instead of a prompt:"
            </P>

            <CodeBlock
                language="bash"
                title="scripted"
                code=r#"infinity-msfs create my-gauge \
  --template rust-wasm-gauge \
  --name my-gauge \
  --no-input

# scaffold into a directory that already has files in it
infinity-msfs create . --template ts-react --name pfd --force"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Token substitution">
                "The renderer replaces "
                <InlineCode>"replace-me"</InlineCode>
                " (kebab), "
                <InlineCode>"replace_me"</InlineCode>
                " (snake), and "
                <InlineCode>"ReplaceMe"</InlineCode>
                " (pascal) with your project name in the matching case, across file contents "
                <em>"and"</em>
                " directory names. The two Rust WASM templates additionally prompt for the exported symbol, so the gauge symbol can differ from the project name when you need it to."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("infinity-msfs comes with a TUI that scaffolds any of six project types. It writes the crate, the config, and the gitignore, then tells you what to run next.")
}

fn templates() -> DocSection {
    DocSection::new("templates", "The six templates", || {
        view! {
            <List>
                <ListItem>
                    <Term>
                        <InlineCode>"rust-wasm-gauge"</InlineCode>
                    </Term>
                    " is a cdylib MSFS gauge built to "
                    <InlineCode>"wasm32-wasip1"</InlineCode>
                    ". Implements "
                    <InlineCode>"Gauge"</InlineCode>
                    " (init / update / draw / kill) and exports it with "
                    <InlineCode>"export_gauge!"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"rust-wasm-system"</InlineCode>
                    </Term>
                    " is the same shape for a system module. Implements "
                    <InlineCode>"System"</InlineCode>
                    " (init / update / kill, no draw) and exports with "
                    <InlineCode>"export_system!"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"rust-native"</InlineCode>
                    </Term>
                    " is a host-target binary linked against SimConnect. Windows-only at link time."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"ts-react"</InlineCode>
                    </Term>
                    " is a TypeScript + React instrument bundled via rolldown, wired to "
                    <InlineCode>"react-msfs"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"ts-framework"</InlineCode>
                    </Term>
                    " is a TypeScript Avionics Framework instrument using "
                    <InlineCode>"BaseInstrument"</InlineCode>
                    " and "
                    <InlineCode>"FSComponent"</InlineCode>
                    ". Reach for this on high-frequency PFD/ND/EFIS displays."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"rescript-react"</InlineCode>
                    </Term>
                    " is a React instrument written in ReScript on top of "
                    <InlineCode>"@infinity-msfs/rescript-msfs"</InlineCode>
                    ". The build runs the ReScript compile before bundling."
                </ListItem>
            </List>

            <Callout tone=Tone::Accent icon=icons::INFO title="JS templates use bun">
                "The three instrument templates print "
                <InlineCode>"bun install"</InlineCode>
                " then "
                <InlineCode>"bun run infinity:build"</InlineCode>
                " as their next steps. That npm script chains the ReScript compile (where applicable) into "
                <InlineCode>"infinity-msfs build"</InlineCode>
                ", so one command covers it from then on. The Build Tools guide covers the "
                <InlineCode>"[js]"</InlineCode>
                " config surface in depth."
            </Callout>
        }
    })
    .numbered("2.1")
    .summary("Three Rust project shapes and three JS/TS instrument shapes, each with a working entry point and a matching infinity-msfs.toml.")
}

fn what_create_writes() -> DocSection {
    DocSection::new(
        "what-create-writes",
        "What the gauge template writes",
        || {
            view! {
                <P>
                    <InlineCode>
                        "infinity-msfs create my-gauge -t rust-wasm-gauge -n my-gauge"
                    </InlineCode>
                    " produces a crate that builds as-is. "
                    <InlineCode>"infinity-rs"</InlineCode>
                    " comes from crates.io, so there is no git dependency to pin."
                </P>

                <CodeBlock
                    language="toml"
                    title="Cargo.toml"
                    code=r#"[package]
name = "my-gauge"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
infinity-rs = "0.2.8""#
                />

                <CodeBlock
                    language="toml"
                    title="infinity-msfs.toml"
                    code=r#"[rust]
default_target = "wasm32-wasip1"
output_dir = "build/msfs"

[rust.wasm_opt]
enabled = true
passes = [
    "-O1",
    "--signext-lowering",
    "--enable-bulk-memory",
    "--enable-nontrapping-float-to-int",
]

[[rust.packages]]
cargo_package = "my-gauge"
cargo_bin = "my_gauge"
artifact_name = "my-gauge.wasm""#
                />

                <CodeBlock
                    language="rust"
                    title="src/lib.rs"
                    code=r#"use infinity_rs::prelude::*;

pub struct MyGauge;

impl Gauge for MyGauge {
    fn init(&mut self, _ctx: &Context, _install: &mut GaugeInstall) -> bool {
        true
    }
    fn update(&mut self, _ctx: &Context, _dt: f32) -> bool {
        true
    }
    fn draw(&mut self, _ctx: &Context, _draw: &mut GaugeDraw) -> bool {
        true
    }
    fn kill(&mut self, _ctx: &Context) -> bool {
        true
    }
}

infinity_rs::export_gauge!(name = my_gauge, state = MyGauge, ctor = MyGauge);"#
                />

                <Callout tone=Tone::Accent icon=icons::INFO title="Config keys">
                    <InlineCode>"cargo_package"</InlineCode>
                    " is the Cargo package name, "
                    <InlineCode>"cargo_bin"</InlineCode>
                    " is the lib/bin target (underscored by Cargo), and "
                    <InlineCode>"artifact_name"</InlineCode>
                    " is what lands in "
                    <InlineCode>"output_dir"</InlineCode>
                    ". Add one "
                    <InlineCode>"[[rust.packages]]"</InlineCode>
                    " block per artifact for multi-system workspaces."
                </Callout>
            }
        },
    )
    .numbered("2.2")
    .summary("Four files: the crate manifest, the build config, a compiling Gauge impl, and a gitignore.")
}

fn linker_flags() -> DocSection {
    DocSection::new("linker-flags", "Linker flags for host symbols", || {
        view! {
            <P>
                "MSFS resolves functions like "
                <InlineCode>"fsVarsRegisterLVar"</InlineCode>
                " and "
                <InlineCode>"fsCommBusCall"</InlineCode>
                " at module load time, so they are undefined at link time. The empty scaffold links fine because it never calls them. As soon as you use "
                <InlineCode>"LVar"</InlineCode>
                ", "
                <InlineCode>"Subscription"</InlineCode>
                ", or anything else that crosses into the host, the link fails:"
            </P>

            <CodeBlock
                language="text"
                title="without the flags"
                code=r#"rust-lld: error: undefined symbol: fsVarsRegisterLVar
rust-lld: error: undefined symbol: fsVarsLVarSet
rust-lld: error: undefined symbol: fsCommBusRegister
error: could not compile `my-gauge` (lib) due to 1 previous error"#
            />

            <P>
                "Add "
                <InlineCode>".cargo/config.toml"</InlineCode>
                " next to your "
                <InlineCode>"Cargo.toml"</InlineCode>
                ". This is the standard MSFS WASM link configuration and every shipping aircraft in the org uses it:"
            </P>

            <CodeBlock
                language="toml"
                title=".cargo/config.toml"
                code=r#"[target.wasm32-wasip1]
rustflags = [
  "-Clink-arg=--export-table",
  "-Clink-arg=--export=malloc",
  "-Clink-arg=--export=free",
  "-Clink-arg=--import-undefined",
]"#
            />

            <List>
                <ListItem>
                    <InlineCode>"--import-undefined"</InlineCode>
                    " is the one that matters. It turns unresolved symbols into WASM imports that the sim satisfies at load."
                </ListItem>
                <ListItem>
                    <InlineCode>"--export=malloc"</InlineCode>
                    " and "
                    <InlineCode>"--export=free"</InlineCode>
                    " let the host allocate inside your module's linear memory, which the comm bus and network layers rely on."
                </ListItem>
                <ListItem>
                    <InlineCode>"--export-table"</InlineCode>
                    " exposes the function table so the host can call back into registered callbacks."
                </ListItem>
            </List>

            <Callout tone=Tone::Warning icon=icons::TRIANGLE_ALERT title="Add it before you write real code">
                "Because the generated stub compiles without these flags, the failure shows up later, when you add your first LVar. Drop the file in right after scaffolding and you will never hit it."
            </Callout>
        }
    })
    .numbered("2.3")
    .summary("The moment your gauge touches an LVar or the comm bus, you need a .cargo/config.toml. The scaffold does not write one.")
}

fn build_it() -> DocSection {
    DocSection::new("build-it", "Build it", || {
        view! {
            <CodeBlock
                language="bash"
                title="build"
                code=r#"cd my-gauge
infinity-msfs build --release"#
            />

            <CodeBlock
                language="text"
                title="output"
                code=r#"Building 1 package (release; wasm-opt on) /path/to/my-gauge
✓ my-gauge -> my-gauge.wasm  (38.8 KB +38.2 KB · 3.957s)
Done built 1 package in 3.958s"#
            />

            <P>
                "The "
                <InlineCode>"+38.2 KB"</InlineCode>
                " is the delta versus the previous build, tracked in "
                <InlineCode>"target/.infinity-msfs-stats.json"</InlineCode>
                ". It is a cheap way to catch accidental bloat in review."
            </P>

            <P>
                <InlineCode>"infinity-msfs projects"</InlineCode>
                " shows what the CLI resolved from your config without building anything:"
            </P>

            <CodeBlock
                language="text"
                title="infinity-msfs projects"
                code=r#"Projects /path/to/my-gauge
  rust
    • my-gauge [bin: my_gauge] [target: wasm32-wasip1] -> my-gauge.wasm"#
            />

            <P>
                "For an edit-compile loop, leave "
                <InlineCode>"watch"</InlineCode>
                " running. It discovers every workspace-local crate's source dir plus each JS instrument entry, and filters out its own output dirs so a build never retriggers itself:"
            </P>

            <CodeBlock
                language="bash"
                title="watch"
                code=r#"infinity-msfs watch                    # rebuild everything on change
infinity-msfs watch --release
infinity-msfs watch --only my-gauge    # restrict to one entry
infinity-msfs watch --rust-only        # skip the JS pipeline
infinity-msfs watch --debounce 500     # collapse events within 500ms (default 300)"#
            />
        }
    })
    .numbered("3.0")
    .summary("One command builds every configured package. The finish line reports artifact size, wall time, and the size delta against your last build.")
}

fn a_real_gauge() -> DocSection {
    DocSection::new("a-real-gauge", "A gauge that does something", || {
        view! {
            <P>
                "The stub compiles but does nothing. Here is the same gauge wired to an LVar and the comm bus: it applies inbound events to "
                <InlineCode>"L:DEMO_TOGGLE"</InlineCode>
                ", and broadcasts the value back out whenever it changes."
            </P>

            <CodeBlock
                language="rust"
                title="src/lib.rs"
                code=r#"use infinity_rs::prelude::*;

const EVT_TOGGLE: &str = "demo/toggle";

pub struct DemoGauge {
    toggle: LVar,
    last_sent: i32,
    _sub: Subscription,
}

impl DemoGauge {
    pub fn new() -> Self {
        let toggle = LVar::new("L:DEMO_TOGGLE", "Bool").expect("LVar create failed");

        // LVar is a handle, not a Copy value. Make a second one for the
        // callback instead of moving the field out.
        let toggle_for_cb = LVar::new("L:DEMO_TOGGLE", "Bool").expect("LVar create failed");

        let sub = Subscription::subscribe(EVT_TOGGLE, move |bytes| {
            let next = bytes.first().copied().unwrap_or(0) as f64;
            let _ = toggle_for_cb.set(next);
        })
        .expect("commbus subscribe failed");

        Self { toggle, last_sent: -1, _sub: sub }
    }
}

impl Gauge for DemoGauge {
    fn init(&mut self, _ctx: &Context, _install: &mut GaugeInstall) -> bool { true }

    fn update(&mut self, _ctx: &Context, _dt: f32) -> bool {
        let v = self.toggle.get().unwrap_or(0.0) as i32;
        if v != self.last_sent {
            let _ = commbus_call(EVT_TOGGLE, &[v as u8], BroadcastFlags::ALL);
            self.last_sent = v;
        }
        true
    }

    fn draw(&mut self, _ctx: &Context, _draw: &mut GaugeDraw) -> bool { true }
    fn kill(&mut self, _ctx: &Context) -> bool { true }
}

infinity_rs::export_gauge!(
    name = demo_gauge,
    state = DemoGauge,
    ctor = DemoGauge::new()
);"#
            />

            <List>
                <ListItem>
                    <Term>"Hold the subscription."</Term>
                    " "
                    <InlineCode>"Subscription"</InlineCode>
                    " unregisters on drop, so bind it to a field ("
                    <InlineCode>"_sub"</InlineCode>
                    ") rather than letting it fall out of scope at the end of "
                    <InlineCode>"new()"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>"Create a second handle for the closure."</Term>
                    " "
                    <InlineCode>"LVar"</InlineCode>
                    " is not "
                    <InlineCode>"Copy"</InlineCode>
                    ". Calling "
                    <InlineCode>"LVar::new"</InlineCode>
                    " twice on the same name is the idiomatic move here, and it is what the "
                    <InlineCode>"comm_bus_system"</InlineCode>
                    " example in the repo does."
                </ListItem>
                <ListItem>
                    <Term>"Gate your broadcasts."</Term>
                    " "
                    <InlineCode>"update"</InlineCode>
                    " runs every frame. The "
                    <InlineCode>"last_sent"</InlineCode>
                    " comparison keeps the bus quiet when nothing changed."
                </ListItem>
            </List>

            <Callout tone=Tone::Warning icon=icons::TRIANGLE_ALERT title="This is where you need the linker flags">
                "This version calls into the host, so it will not link without the "
                <InlineCode>".cargo/config.toml"</InlineCode>
                " from section 2.3."
            </Callout>
        }
    })
    .numbered("3.1")
    .summary("Drive a custom LVar from the comm bus and broadcast changes back out. This is the shape most systems code takes.")
}

fn native_simconnect() -> DocSection {
    DocSection::new("native-simconnect", "Native SimConnect", || {
        view! {
            <P>
                "Native packages set "
                <InlineCode>"artifact_kind = \"native\""</InlineCode>
                ". The CLI drops "
                <InlineCode>"--target"</InlineCode>
                " so cargo uses the host triple, forwards the feature list, skips "
                <InlineCode>"wasm-opt"</InlineCode>
                ", and copies "
                <InlineCode>"SimConnect.dll"</InlineCode>
                " from the cached SDK next to the resulting "
                <InlineCode>".exe"</InlineCode>
                ". On Linux the native plan is silently skipped."
            </P>

            <CodeBlock
                language="bash"
                title="scaffold"
                code="infinity-msfs create airspeed-probe -t rust-native -n airspeed-probe"
            />

            <Grid2>
                <DocCard eyebrow="Cargo.toml" title="Crate setup">
                    <P>
                        "The crate disables default features (so the gauge-host modules are excluded from the native binary) and opts into "
                        <InlineCode>"simconnect"</InlineCode>
                        "."
                    </P>
                </DocCard>
                <DocCard eyebrow="infinity-msfs.toml" title="Native package">
                    <P>
                        "Set "
                        <InlineCode>"artifact_kind = \"native\""</InlineCode>
                        ". Note there is no "
                        <InlineCode>"[rust]"</InlineCode>
                        " table needed at all for a native-only project."
                    </P>
                </DocCard>
            </Grid2>

            <CodeBlock
                language="toml"
                title="Cargo.toml"
                code=r#"[package]
name = "airspeed-probe"
version = "0.1.0"
edition = "2024"

[dependencies]
infinity-rs = { version = "0.2.8", default-features = false, features = ["simconnect"] }"#
            />

            <CodeBlock
                language="toml"
                title="infinity-msfs.toml"
                code=r#"[[rust.packages]]
cargo_package = "airspeed-probe"
cargo_bin = "airspeed-probe"
artifact_kind = "native"
output_dir = "build/native""#
            />

            <P>
                "The template ships a one-line connect check. Here is a fuller probe that pulls indicated airspeed once and exits:"
            </P>

            <CodeBlock
                language="rust"
                title="src/main.rs"
                code=r#"use infinity_rs::simconnect::{
    DataDefinitionId, DataRequestFlag, DataRequestId, DataType, ObjectId,
    Period, Recv, SimConnect, SimObjectDataView, UNUSED,
};
use std::time::{Duration, Instant};

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
struct Aircraft { airspeed_kt: f64 }

const DEF: DataDefinitionId = DataDefinitionId::new(1);
const REQ: DataRequestId = DataRequestId::new(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sc = SimConnect::open("airspeed-probe")?;
    sc.add_to_data_definition(
        DEF, "AIRSPEED INDICATED", "knots",
        DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED,
    )?;
    sc.request_data_on_sim_object(
        REQ, DEF, ObjectId::USER, Period::SIMCONNECT_PERIOD_ONCE,
        DataRequestFlag::empty(), 0, 0, 0,
    )?;

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let mut got = false;
        sc.dispatch(|msg| if let Recv::SimObjectData(d) = msg {
            let v: Aircraft = unsafe { SimObjectDataView(d).data_as() };
            println!("{:.1} kt", v.airspeed_kt);
            got = true;
        })?;
        if got { return Ok(()); }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err("timed out".into())
}"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Build & run">
                <InlineCode>"infinity-msfs build --release"</InlineCode>
                " produces "
                <InlineCode>"build/native/airspeed-probe.exe"</InlineCode>
                " alongside "
                <InlineCode>"SimConnect.dll"</InlineCode>
                ". Launch MSFS, load any flight, then run the exe."
            </Callout>
        }
    })
    .numbered("4.0")
    .summary("The same crate compiles to a native exe that talks to a running MSFS via SimConnect. Scaffold it with the rust-native template.")
}

fn build_anatomy() -> DocSection {
    DocSection::new("build-anatomy", "Build anatomy", || {
        view! {
            <List>
                <ListItem>
                    <Term>"SDK provisioning."</Term>
                    " Uses the cached SDK subset from "
                    <InlineCode>"infinity-msfs sdk install"</InlineCode>
                    ", or the full SDK when "
                    <InlineCode>"MSFS2024_SDK"</InlineCode>
                    " is set."
                </ListItem>
                <ListItem>
                    <Term>"Cargo invocation."</Term>
                    " Calls "
                    <InlineCode>"cargo build -p <pkg> --target wasm32-wasip1"</InlineCode>
                    " for WASM packages and drops "
                    <InlineCode>"--target"</InlineCode>
                    " for native ones, forwarding "
                    <InlineCode>"cargo_features"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>"Optimization."</Term>
                    " Pipes WASM artifacts through "
                    <InlineCode>"wasm-opt"</InlineCode>
                    " with the "
                    <InlineCode>"[rust.wasm_opt] passes"</InlineCode>
                    " list. Skipped for native packages and by "
                    <InlineCode>"--no-wasm-opt"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>"Asset placement."</Term>
                    " Copies the artifact plus any "
                    <InlineCode>"copy_files = [...]"</InlineCode>
                    " rules into "
                    <InlineCode>"output_dir"</InlineCode>
                    ". Native builds also pull "
                    <InlineCode>"SimConnect.dll"</InlineCode>
                    " from the SDK."
                </ListItem>
                <ListItem>
                    <Term>"Ordering."</Term>
                    " "
                    <InlineCode>"[[rust.packages]]"</InlineCode>
                    " build first, then "
                    <InlineCode>"[[js.instruments]]"</InlineCode>
                    ". "
                    <InlineCode>"[hooks] pre"</InlineCode>
                    " and "
                    <InlineCode>"post"</InlineCode>
                    " fire once around the whole pipeline, not per side."
                </ListItem>
            </List>

            <CodeBlock
                language="bash"
                title="day-to-day"
                code=r#"infinity-msfs build              # debug, everything configured
infinity-msfs build --release    # optimized
infinity-msfs build --only foo   # one entry (matches rust and js names)
infinity-msfs build --rust-only  # skip the JS pipeline
infinity-msfs build --js-only    # skip the cargo pipeline
infinity-msfs build --no-wasm-opt
infinity-msfs build -v           # stream cargo/wasm-opt output

infinity-msfs test               # cargo test on the host for configured packages
infinity-msfs test -- --nocapture
infinity-msfs watch              # rebuild on change
infinity-msfs projects           # list resolved plans without building
infinity-msfs doctor             # environment pre-flight check"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Compiling the sim package">
                <InlineCode>"infinity-msfs package"</InlineCode>
                " wraps "
                <InlineCode>"fspackagetool.exe"</InlineCode>
                " to turn project XMLs into installable MSFS packages via "
                <InlineCode>"[[sim_packages]]"</InlineCode>
                ". It is Windows-only and needs the full SDK plus MSFS 2024 installed. See the Build Tools guide."
            </Callout>
        }
    })
    .numbered("4.1")
    .summary("What `infinity-msfs build` does for you, and the knobs you can turn when you need to.")
}

fn next_steps() -> DocSection {
    DocSection::new("next-steps", "Next steps", || {
        view! {
            <List>
                <ListItem>
                    "Read the "
                    <Term>"Build Tools"</Term>
                    " guide for the full "
                    <InlineCode>"infinity-msfs.toml"</InlineCode>
                    " surface: multi-package workspaces, the three JS instrument modes, "
                    <InlineCode>"[[sim_packages]]"</InlineCode>
                    ", and hooks."
                </ListItem>
                <ListItem>
                    "Browse the "
                    <Term>"API Overview"</Term>
                    " for a tour of every module: vars, comm bus, flow events, IO, network, charts, planned route, map view, VFX, NanoVG, and SimConnect."
                </ListItem>
                <ListItem>
                    "Explore the "
                    <InlineCode>"msfs/examples/"</InlineCode>
                    " directory in the repo for working gauges and systems covering each subsystem."
                </ListItem>
                <ListItem>
                    "Pin an exact "
                    <InlineCode>"infinity-rs"</InlineCode>
                    " version while the API is pre-1.0. The surface is stabilising but expect occasional renames between minor versions."
                </ListItem>
            </List>
        }
    })
    .numbered("5.0")
    .summary("Where to go now that the toolchain works.")
}
