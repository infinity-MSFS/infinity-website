use infinity_ui::prelude::*;

use super::prose::{DocCard, Grid2, Term};

const REPO_BUILD_TOOLS: &str = "https://github.com/infinity-MSFS/infinity-msfs-build-tools";
const REPO_INFINITY_RS: &str = "https://github.com/infinity-MSFS/infinity-rs";
const REPO_RESCRIPT_MSFS: &str = "https://github.com/infinity-MSFS/rescript-msfs";
const LINK: &str = "text-accent-300/80 underline-offset-4 hover:underline";

pub fn doc() -> Doc {
    Doc::new("build-tools", "Build Tools", "Guide", "Guide")
        .scope("infinity-rs")
        .tagline(
            "infinity-msfs is one CLI for every kind of MSFS package we ship — Rust WASM gauges, Rust native SimConnect clients, ReScript-React instruments, plain TypeScript-React instruments, TypeScript Avionics-Framework BaseInstrument gauges, and full sim packages compiled by fspackagetool. One infinity-msfs.toml per project, one command to build.",
        )
        .tags([
            "infinity-msfs",
            "Cargo",
            "WASM",
            "SimConnect",
            "ReScript",
            "TypeScript",
            "Avionics Framework",
            "fspackagetool",
        ])
        .section(overview())
        .section(install())
        .section(config_shape())
        .section(cargo_wasm())
        .section(cargo_native())
        .section(rescript_react())
        .section(ts_react())
        .section(ts_avionics())
        .section(sim_packages())
        .section(watch())
        .section(create())
        .section(cli_reference())
}

fn overview() -> DocSection {
    DocSection::new("overview", "What the build tool does", || {
        view! {
            <Lead>
                <InlineCode>"infinity-msfs"</InlineCode>
                " is a single Rust CLI that orchestrates every flavour of MSFS artefact this project ships: WASM gauges/systems, native SimConnect clients, three different JS/TS instrument bundlers, and full sim-package compilation via "
                <InlineCode>"fspackagetool.exe"</InlineCode>
                ". There is no per-project shell glue — you describe everything declaratively in "
                <InlineCode>"infinity-msfs.toml"</InlineCode>
                " and run one command."
            </Lead>

            <List>
                <ListItem>
                    <Term>"Rust WASM gauges/systems"</Term>
                    " — wraps "
                    <InlineCode>"cargo build --target wasm32-wasip1"</InlineCode>
                    ", runs "
                    <InlineCode>"wasm-opt"</InlineCode>
                    ", copies the artefact and any asset rules into the output package layout."
                </ListItem>
                <ListItem>
                    <Term>"Rust native SimConnect clients"</Term>
                    " — builds for the host triple, skips wasm-opt, and bundles "
                    <InlineCode>"SimConnect.dll"</InlineCode>
                    " from the cached SDK alongside the produced exe."
                </ListItem>
                <ListItem>
                    <Term>"JS/TS instruments"</Term>
                    " — three modes: "
                    <InlineCode>"react"</InlineCode>
                    " (TypeScript + React), "
                    <InlineCode>"rescriptReact"</InlineCode>
                    " (ReScript + React), and "
                    <InlineCode>"baseInstrument"</InlineCode>
                    " (TypeScript Avionics Framework / FSComponent). Bundling is done via rolldown with Coherent-GT-friendly defaults; the harness HTML + "
                    <InlineCode>"BaseInstrument"</InlineCode>
                    " shell is generated for you."
                </ListItem>
                <ListItem>
                    <Term>"MSFS sim packages"</Term>
                    " — "
                    <InlineCode>"infinity-msfs package"</InlineCode>
                    " wraps "
                    <InlineCode>"fspackagetool.exe"</InlineCode>
                    " from the MSFS 2024 SDK to compile project XMLs (aircraft, scenery, marketplace bundles) into ready-to-install packages. Windows-only; requires MSFS 2024 installed locally."
                </ListItem>
                <ListItem>
                    <Term>"SDK provisioning"</Term>
                    " — downloads the pinned MSFS 2024 SDK headers + SimConnect import library on first use, caches them per-user, and exposes them to both bindgen and the linker."
                </ListItem>
                <ListItem>
                    <Term>"Watch mode"</Term>
                    " — "
                    <InlineCode>"infinity-msfs watch"</InlineCode>
                    " debounces filesystem events across every workspace crate and JS instrument source dir, rebuilding the unified pipeline on change."
                </ListItem>
                <ListItem>
                    <Term>"Project scaffolding"</Term>
                    " — "
                    <InlineCode>"infinity-msfs create"</InlineCode>
                    " spins up a new project from a built-in template (Rust WASM gauge / system / native, ReScript-React, TypeScript-React, TypeScript Avionics-Framework). Templates are embedded in the binary; no clone needed."
                </ListItem>
            </List>

            <Callout tone=Tone::Accent icon=icons::INFO title="Cross-platform">
                "The Rust + JS pipelines run on Linux, macOS, and Windows. Native "
                <em>"linking"</em>
                " against SimConnect is Windows-only (the import lib only ships for Windows). "
                <InlineCode>"infinity-msfs package"</InlineCode>
                " is also Windows-only — fspackagetool drives a partial sim instance to compile assets, and the sim binary is Windows-only."
            </Callout>
        }
    })
    .numbered("0.0")
    .summary("One infinity-msfs.toml describes any mix of Rust crates, JS/TS instruments, and MSFS sim packages. The CLI fans out to cargo, rescript, rolldown, and fspackagetool as needed.")
}

fn install() -> DocSection {
    DocSection::new("install", "Install & doctor", || {
        view! {
            <CodeBlock
                language="bash"
                title="install"
                code=r#"# from crates.io
cargo install infinity-msfs

# or from a clone of the build-tools repo
git clone https://github.com/infinity-MSFS/infinity-msfs-build-tools
cd infinity-msfs-build-tools
cargo install --path ."#
            />

            <P>
                <InlineCode>"infinity-msfs doctor"</InlineCode>
                " validates the toolchain up front so first-time builds don't fail half-way through:"
            </P>

            <CodeBlock
                language="bash"
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

            <List>
                <ListItem>
                    <Term>"Rust 1.85+"</Term>
                    " with the "
                    <InlineCode>"wasm32-wasip1"</InlineCode>
                    " target installed for any WASM project."
                </ListItem>
                <ListItem>
                    <Term>"wasm-opt"</Term>
                    " on PATH (can be disabled per-project via "
                    <InlineCode>"[rust.wasm_opt] enabled = false"</InlineCode>
                    ")."
                </ListItem>
                <ListItem>
                    <Term>"bun (or node + a JS runner)"</Term>
                    " for any JS/TS instrument — the bundler shells out to your "
                    <InlineCode>"build_command"</InlineCode>
                    " for ReScript projects, and resolves npm dependencies via the JS runtime."
                </ListItem>
                <ListItem>
                    <Term>"MSFS 2024 + the full SDK"</Term>
                    " only required for "
                    <InlineCode>"infinity-msfs package"</InlineCode>
                    ". Set "
                    <InlineCode>"MSFS2024_SDK"</InlineCode>
                    " to the SDK root so "
                    <InlineCode>"fspackagetool.exe"</InlineCode>
                    " can be located."
                </ListItem>
            </List>

            <P>"The SDK headers + SimConnect lib are fetched on demand:"</P>

            <CodeBlock
                language="bash"
                title="sdk"
                code=r#"infinity-msfs sdk install          # download + cache
infinity-msfs sdk install --force  # re-download
infinity-msfs sdk path             # print the resolved cache path
infinity-msfs sdk remove           # nuke the cache"#
            />
        }
    })
    .numbered("1.0")
    .summary("Install once from the workspace, then run infinity-msfs doctor to verify your environment.")
}

fn config_shape() -> DocSection {
    DocSection::new("config-shape", "infinity-msfs.toml at a glance", || {
        view! {
            <P>
                "The workspace is split into encapsulated crates: "
                <InlineCode>"infinity-build-rust"</InlineCode>
                ", "
                <InlineCode>"infinity-build-js"</InlineCode>
                ", "
                <InlineCode>"infinity-build-package"</InlineCode>
                ", "
                <InlineCode>"infinity-build-sdk"</InlineCode>
                ", "
                <InlineCode>"infinity-build-watch"</InlineCode>
                ". The TOML schema below mirrors that split."
            </P>

            <CodeBlock
                language="toml"
                title="infinity-msfs.toml — full surface"
                code=r#"# ── Cargo build pipeline ────────────────────────────────────
[rust]
default_target = "wasm32-wasip1"
output_dir     = "build/msfs"
copy_files = [
  { from = "manifest.json", to = "build/msfs/manifest.json" },
  { from = "layout.json",   to = "build/msfs/layout.json" },
]

  [rust.wasm_opt]
  enabled = true
  passes  = [
    "-O1",
    "--signext-lowering",
    "--enable-bulk-memory",
    "--enable-nontrapping-float-to-int",
  ]

  [[rust.packages]]
  cargo_package  = "demo-gauge"     # cargo package name
  cargo_bin      = "demo_gauge"     # cargo bin / lib target
  artifact_name  = "demo-gauge.wasm"
  # artifact_kind  = "wasm" (default) | "native"
  # cargo_features = ["foo"]
  # output_dir     = "build/msfs/systems"
  # target         = "wasm32-wasip1"            # override default_target
  # copy_files     = [{ from = "...", to = "..." }]

# ── JS / TS instruments ─────────────────────────────────────
[js]
package_name = "my-aircraft"
package_dir  = "PackageSources"

  [[js.instruments]]
  name  = "pfd"
  index = "src/pfd/index.tsx"

    [js.instruments.simulator_package]
    type           = "react"          # | "rescriptReact" | "baseInstrument"
    file_name      = "pfd"
    template_id    = "PFD"
    is_interactive = true
    imports        = ["/JS/dataStorage.js"]

# ── MSFS sim packages (fspackagetool.exe) ───────────────────
[[sim_packages]]
name        = "aerodynamics-dc10"
project_xml = "DC10.xml"
output_dir  = "build/Packages"
temp_dir    = "build/PackageSourcesTemp"
# marketplace_dir = "build/Marketplace"
# force_rebuild   = false
# mirror_output   = false
# prefer_steam    = false

# ── Build lifecycle hooks ───────────────────────────────────
[hooks]
pre  = ["cargo fmt --check"]
post = ["python ./tools/fix_layout.py"]"#
            />

            <List>
                <ListItem>
                    <Term>"Per-package overrides."</Term>
                    " "
                    <InlineCode>"target"</InlineCode>
                    ", "
                    <InlineCode>"output_dir"</InlineCode>
                    ", "
                    <InlineCode>"artifact_kind"</InlineCode>
                    ", and "
                    <InlineCode>"copy_files"</InlineCode>
                    " in "
                    <InlineCode>"[[rust.packages]]"</InlineCode>
                    " shadow / extend the top-level "
                    <InlineCode>"[rust]"</InlineCode>
                    " defaults. Useful for mixing WASM gauges and native SimConnect clients in one workspace."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"copy_files"</InlineCode>
                        " rules"
                    </Term>
                    " run after a successful build so a single command produces a runnable MSFS package directory (manifest, layout, JSON config, textures)."
                </ListItem>
                <ListItem>
                    <Term>"JS instrument modes"</Term>
                    " are mutually exclusive per instrument; nothing stops you mixing modes across "
                    <InlineCode>"[[js.instruments]]"</InlineCode>
                    " entries in the same project. The same toml builds an FSComponent PFD, a ReScript EFB, and a vanilla React overlay if that's what you need."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"[[sim_packages]]"</InlineCode>
                    </Term>
                    " is independent of "
                    <InlineCode>"[rust]"</InlineCode>
                    " — a project can have only Rust packages, only sim packages, only JS instruments, or any combination."
                </ListItem>
            </List>
        }
    })
    .numbered("1.1")
    .summary("Four independent sections: [rust] for cargo builds, [js] for JS/TS bundling, [[sim_packages]] for fspackagetool, [hooks] for lifecycle scripts. You only fill in what you use.")
}

fn cargo_wasm() -> DocSection {
    DocSection::new("cargo-wasm", "Project: Cargo WASM gauge / system", || {
        view! {
            <P>
                "The crate emits a "
                <InlineCode>"cdylib"</InlineCode>
                " and uses the "
                <InlineCode>"export_gauge!"</InlineCode>
                " / "
                <InlineCode>"export_system!"</InlineCode>
                " macros to wire the standard MSFS gauge / system entry points behind a Rust trait impl. The build tool handles the target, optimization, and asset placement."
            </P>

            <Grid2>
                <DocCard eyebrow="Cargo.toml" title="Crate setup">
                    <P>
                        <InlineCode>"crate-type = [\"cdylib\"]"</InlineCode>
                        ", edition 2024, depend on "
                        <InlineCode>"infinity-rs"</InlineCode>
                        "."
                    </P>
                </DocCard>
                <DocCard eyebrow="infinity-msfs.toml" title="One package entry">
                    <P>
                        <InlineCode>"artifact_kind = \"wasm\""</InlineCode>
                        " (the default), point at the cargo package, name the artefact."
                    </P>
                </DocCard>
            </Grid2>

            <CodeBlock
                language="toml"
                title="Cargo.toml"
                code=r#"[package]
name = "demo-gauge"
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
output_dir     = "build/msfs"
copy_files = [
  { from = "manifest.json", to = "build/msfs/manifest.json" },
  { from = "layout.json",   to = "build/msfs/layout.json" },
]

  [rust.wasm_opt]
  enabled = true
  passes  = [
    "-O1",
    "--signext-lowering",
    "--enable-bulk-memory",
    "--enable-nontrapping-float-to-int",
  ]

  [[rust.packages]]
  cargo_package = "demo-gauge"
  cargo_bin     = "demo_gauge"
  artifact_name = "demo-gauge.wasm""#
            />

            <CodeBlock
                language="rust"
                title="src/lib.rs"
                code=r#"use infinity_rs::prelude::*;

pub struct DemoGauge;

impl Gauge for DemoGauge {
    fn init(&mut self, _ctx: &Context, _install: &mut GaugeInstall) -> bool { true }
    fn update(&mut self, _ctx: &Context, _dt: f32) -> bool { true }
    fn draw(&mut self, _ctx: &Context, _draw: &mut GaugeDraw) -> bool { true }
    fn kill(&mut self, _ctx: &Context) -> bool { true }
}

infinity_rs::export_gauge!(name = demo_gauge, state = DemoGauge, ctor = DemoGauge);"#
            />

            <P>
                "Multi-WASM workspaces (e.g. one package per ATA system) are just a longer "
                <InlineCode>"[[rust.packages]]"</InlineCode>
                " array. See the 10-systems repo for a real example with twelve WASM artefacts in one config:"
            </P>

            <CodeBlock
                language="toml"
                title="infinity-msfs.toml — multi-package"
                code=r#"[[rust.packages]]
cargo_package = "wasm-md10-10"
cargo_bin     = "wasm_md10_10"
artifact_name = "md10_10_systems.wasm"

[[rust.packages]]
cargo_package = "wasm-md10-30"
cargo_bin     = "wasm_md10_30"
artifact_name = "md10_30_systems.wasm"

# … one entry per system / display"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Build">
                <InlineCode>"infinity-msfs build --release"</InlineCode>
                " builds every configured package. Use "
                <InlineCode>"--only wasm-md10-10"</InlineCode>
                " (repeatable, matched against "
                <InlineCode>"cargo_package"</InlineCode>
                ") to restrict."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("A cdylib crate compiled to wasm32-wasip1. The default project shape — every infinity-rs gauge or system module is one of these.")
}

fn cargo_native() -> DocSection {
    DocSection::new("cargo-native", "Project: Cargo native (SimConnect)", || {
        view! {
            <P>
                "Native packages are useful for out-of-sim tools — frame probes, protocol bridges, recording tools, custom telemetry exporters. The build tool drops "
                <InlineCode>"--target"</InlineCode>
                " so cargo uses the host triple, forwards the configured features, skips "
                <InlineCode>"wasm-opt"</InlineCode>
                ", and copies "
                <InlineCode>"SimConnect.dll"</InlineCode>
                " from the cached SDK next to the produced "
                <InlineCode>".exe"</InlineCode>
                "."
            </P>

            <CodeBlock
                language="toml"
                title="Cargo.toml"
                code=r#"[package]
name = "airspeed-probe"
version = "0.1.0"
edition = "2024"

[dependencies]
# disable WASM defaults; opt into the simconnect feature
infinity-rs = { version = "0.2.8", default-features = false, features = ["simconnect"] }"#
            />

            <CodeBlock
                language="toml"
                title="infinity-msfs.toml"
                code=r#"[[rust.packages]]
cargo_package  = "airspeed-probe"
cargo_bin      = "airspeed-probe"
artifact_kind  = "native"
cargo_features = ["simconnect"]
output_dir     = "build""#
            />

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
const REQ: DataRequestId    = DataRequestId::new(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sc = SimConnect::open("airspeed-probe")?;
    sc.add_to_data_definition(DEF, "AIRSPEED INDICATED", "knots",
        DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED)?;
    sc.request_data_on_sim_object(REQ, DEF, ObjectId::USER,
        Period::SIMCONNECT_PERIOD_ONCE, DataRequestFlag::empty(), 0, 0, 0)?;

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        sc.dispatch(|msg| if let Recv::SimObjectData(d) = msg {
            let v: Aircraft = unsafe { SimObjectDataView(d).data_as() };
            println!("{:.1} kt", v.airspeed_kt);
        })?;
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}"#
            />

            <Callout
                tone=Tone::Warning
                icon=icons::TRIANGLE_ALERT
                title="Windows-only linking"
            >
                "The SimConnect import library is shipped only for Windows. Native packages are silently skipped on Linux. Bindgen of the SimConnect headers and WASM gauge builds work identically on either OS — only the link step is gated."
            </Callout>
        }
    })
    .numbered("2.1")
    .summary("The same crate shape, with artifact_kind = \"native\" and the simconnect feature. Builds an exe + bundled SimConnect.dll on Windows.")
}

fn rescript_react() -> DocSection {
    DocSection::new(
        "rescript-react",
        "Project: ReScript + React instrument",
        || {
            view! {
                <P>
                    "Use this mode for interactive panels (EFBs, FMS pages, pop-up overlays) where you want React's component model and ReScript's type-safe bindings. The companion library "
                    <InlineCode>"@infinity-msfs/rescript-msfs"</InlineCode>
                    " wraps the SimVar / Coherent / CommBus surfaces in a change-gated store so a 200-subscription gauge stays under 5 µs/frame (see the "
                    <a href=REPO_RESCRIPT_MSFS target="_blank" rel="noreferrer" class=LINK>
                        "rescript-msfs"
                    </a>
                    " repo)."
                </P>

                <List>
                    <ListItem>
                        "ReScript ≥ 12 with "
                        <InlineCode>"@rescript/core"</InlineCode>
                        " and "
                        <InlineCode>"@rescript/react"</InlineCode>
                        " ≥ 0.15."
                    </ListItem>
                    <ListItem>
                        "React 19.2+ with the React Compiler enabled (the hook layer relies on stable closures)."
                    </ListItem>
                    <ListItem>
                        "bun (or another JS package manager) for the npm dependency graph."
                    </ListItem>
                </List>

                <CodeBlock
                    language="json"
                    title="rescript.json"
                    code=r#"{
  "name": "your-gauge",
  "sources": [{ "dir": "src", "subdirs": true }],
  "package-specs": [{ "module": "es6", "in-source": false }],
  "suffix": ".res.mjs",
  "bs-dependencies": [
    "@rescript/core",
    "@rescript/react",
    "@infinity-msfs/rescript-msfs"
  ],
  "jsx": { "version": 4, "mode": "automatic" }
}"#
                />

                <CodeBlock
                    language="rescript"
                    title="src/Altimeter.res"
                    code=r#"open InfinityMSFS

module Panel = {
  @react.component
  let make = () => {
    let (alt, _)  = useSimVar("INDICATED ALTITUDE", "feet", ())
    let (qnh, set) = useSimVar("L:DC10_BARO_INHG", "number", ())
    <div className="alt">
      {React.string(Float.toString(alt))}
    </div>
  }
}

@module("react-dom/client") external createRoot: Dom.element => 'r = "createRoot"
@send external rootRender: ('r, React.element) => unit = "render"

let mount = (): unit => {
  let target = RenderTarget.getRenderTarget()
  let root = createRoot(target)
  rootRender(root, <InfinityMSFS.Provider> <Panel /> </InfinityMSFS.Provider>)
}"#
                />

                <Callout
                    tone=Tone::Accent
                    icon=icons::INFO
                    title="Don't call mount() yourself"
                >
                    "The bundler injects the auto-invocation of the bundle's "
                    <InlineCode>"mount"</InlineCode>
                    " export on script load. Calling "
                    <InlineCode>"mount()"</InlineCode>
                    " at module scope yourself will double-mount."
                </Callout>

                <CodeBlock
                    language="toml"
                    title="infinity-msfs.toml"
                    code=r#"[js]
package_name = "dc10-cockpit"
package_dir  = "PackageSources"

  [[js.instruments]]
  name  = "altimeter"
  index = "lib/es6/src/Altimeter.res.mjs"   # rescript output

    [js.instruments.simulator_package]
    type           = "rescriptReact"
    file_name      = "altimeter"
    template_id    = "Altimeter"
    is_interactive = true
    imports        = ["/JS/dataStorage.js"]
    build_command  = "bun run build"   # default; runs before bundling
    # build_dir    = "ui"              # optional, runs build_command from here"#
                />

                <P>"For each instrument the bundler emits:"</P>

                <CodeBlock
                    language="text"
                    title="output"
                    code=r#"PackageSources/html_ui/Pages/VCockpit/Instruments/dc10-cockpit/altimeter/
├── altimeter.html        # template + script tags
├── altimeter.index.js    # generated BaseInstrument harness
├── altimeter.js          # rolldown bundle of your ReScript output
└── altimeter.css         # extracted CSS"#
                />

                <P>
                    "Wire it in "
                    <InlineCode>"panel.cfg"</InlineCode>
                    ":"
                </P>

                <CodeBlock
                    language="ini"
                    title="panel.cfg"
                    code=r#"[VCockpit01]
size_mm     = 1080,1080
pixel_size  = 1080,1080
texture     = $DIS_MD_1
htmlgauge00 = dc10-cockpit/altimeter/altimeter.html,0,0,1080,1080"#
                />
            }
        },
    )
    .numbered("3.0")
    .summary("type = \"rescriptReact\". The bundler runs your ReScript build, then bundles the emitted .res.mjs entry via rolldown.")
}

fn ts_react() -> DocSection {
    DocSection::new("ts-react", "Project: TypeScript + React instrument", || {
        view! {
            <P>
                "Identical pipeline to the ReScript flow without the ReScript compile step. Use this mode if your team already has a TypeScript codebase or you want to consume an existing library like "
                <InlineCode>"react-msfs"</InlineCode>
                ". A working example lives in "
                <InlineCode>"10-systems/src-visualizer"</InlineCode>
                " — a hydraulic visualizer that runs both inside MSFS and in ACE for development."
            </P>

            <CodeBlock
                language="toml"
                title="infinity-msfs.toml"
                code=r#"[[js.instruments]]
name  = "visualizer"
index = "src-visualizer/index.tsx"

  [js.instruments.simulator_package]
  type           = "react"
  file_name      = "visualizer"
  is_interactive = true
  imports        = ["/JS/dataStorage.js"]"#
            />

            <CodeBlock
                language="tsx"
                title="src-visualizer/index.tsx"
                code=r#"import "./index.css";
import React from "react";
import { render } from "./lib/render";
import { SimVarProvider } from "react-msfs";
import { App } from "./components/App";

const Provider = SimVarProvider as any;
render(
  <Provider>
    <App />
  </Provider>,
);"#
            />

            <Callout
                tone=Tone::Accent
                icon=icons::INFO
                title="Same defaults as rescriptReact"
            >
                "Rolldown is configured as IIFE, lowers syntax to ES2019 (Coherent chokes on "
                <InlineCode>"?."</InlineCode>
                " and "
                <InlineCode>"??"</InlineCode>
                "), externalises "
                <InlineCode>"/Images/*"</InlineCode>
                " and "
                <InlineCode>"/Fonts/*"</InlineCode>
                ", and injects the auto-invocation of the bundle entry. Override either template per-instrument with "
                <InlineCode>"html_template"</InlineCode>
                " / "
                <InlineCode>"js_template"</InlineCode>
                "."
            </Callout>
        }
    })
    .numbered("3.1")
    .summary("type = \"react\". Plain TSX entry — no ReScript step, but the bundling, harness generation, and Coherent-GT lowering are identical.")
}

fn ts_avionics() -> DocSection {
    DocSection::new(
        "ts-avionics",
        "Project: TypeScript Avionics Framework (BaseInstrument)",
        || {
            view! {
                <P>
                    "When you need the most performance the JS side can give you — PFD, ND, EICAS, anything that re-renders nearly every tick — drop React and target the MSFS Avionics Framework directly. The instrument defines its own "
                    <InlineCode>"BaseInstrument"</InlineCode>
                    " subclass and renders via "
                    <InlineCode>"FSComponent"</InlineCode>
                    ". The build tool generates the harness HTML and the "
                    <InlineCode>"BaseInstrument"</InlineCode>
                    " shell from your config."
                </P>

                <CodeBlock
                    language="toml"
                    title="infinity-msfs.toml"
                    code=r#"[[js.instruments]]
name  = "InfinityBridge"
index = "src-bridge/Main.res.mjs"   # or any TS/JS entry

  [js.instruments.simulator_package]
  type             = "baseInstrument"
  template_id      = "Infinity-Bridge"        # must match BaseInstrument.templateID()
  mount_element_id = "infinity-bridge-gauge"  # must match the FSComponent.render() target"#
                />

                <P>
                    "The two required fields, "
                    <InlineCode>"template_id"</InlineCode>
                    " and "
                    <InlineCode>"mount_element_id"</InlineCode>
                    ", are wired through to the generated HTML and harness — keep them in sync with what your instrument code declares. Example with the ReScript "
                    <InlineCode>"Instrument.define"</InlineCode>
                    " wrapper from "
                    <InlineCode>"@infinity-msfs/rescript-instrument"</InlineCode>
                    ":"
                </P>

                <CodeBlock
                    language="rescript"
                    title="src-bridge/Main.res"
                    code=r#"%%raw(`import "./index.css"`)

Instrument.define(
  "infinity-bridge-gauge",
  {
    templateID: "Infinity-Bridge",
    onConnected: _self => {
      // open sockets, register listeners, …
    },
    onDisconnected: _self => {
      // tear down
    },
  },
)"#
                />

                <P>
                    "The same shape works in plain TypeScript — the entry point exports a class extending "
                    <InlineCode>"BaseInstrument"</InlineCode>
                    ", the bundler wraps it with the harness, and Coherent finds the template by ID at panel-load time:"
                </P>

                <CodeBlock
                    language="tsx"
                    title="src/Pfd.tsx"
                    code=r#"import { FSComponent, DisplayComponent, VNode } from '@microsoft/msfs-sdk';

class Pfd extends DisplayComponent<{}> {
  public render(): VNode {
    return <div class="pfd">…</div>;
  }
}

class PfdInstrument extends BaseInstrument {
  public get templateID(): string { return 'PFD'; }
  public connectedCallback(): void {
    super.connectedCallback();
    FSComponent.render(<Pfd />, document.getElementById('pfd-mount'));
  }
}

registerInstrument('pfd-instrument', PfdInstrument);"#
                />

                <Callout
                    tone=Tone::Accent
                    icon=icons::INFO
                    title="When to pick this over React"
                >
                    "React (TS or ReScript) is fine for interactive surfaces and even mid-tick gauges if you cache aggressively. Reach for "
                    <InlineCode>"baseInstrument"</InlineCode>
                    " when (a) you need direct access to the gauge lifecycle / SVG renderer, or (b) profiling says React's reconciler is the bottleneck. Many of our aircraft mix all three modes in the same toml."
                </Callout>
            }
        },
    )
    .numbered("3.2")
    .summary("type = \"baseInstrument\". For high-frequency PFD/ND/EFIS gauges where you want raw access to the FSComponent / Avionics Framework lifecycle.")
}

fn sim_packages() -> DocSection {
    DocSection::new(
        "sim-packages",
        "Project: MSFS sim packages (fspackagetool)",
        || {
            view! {
                <P>
                    <InlineCode>"fspackagetool.exe"</InlineCode>
                    " ships in the MSFS 2024 SDK and orchestrates a partial sim instance to compile MaterialLibs, BGL scenery, model.xml → glTF, encrypted flight models, layout.json, manifest.json, and Marketplace export bundles. There is no standalone compiler, so this command is Windows-only and additionally requires MSFS 2024 to be installed locally."
                </P>

                <CodeBlock
                    language="toml"
                    title="infinity-msfs.toml"
                    code=r#"[[sim_packages]]
name        = "aerodynamics-dc10"
project_xml = "PackageSources/DC10/DC10.xml"
output_dir  = "build/Packages"
temp_dir    = "build/PackageSourcesTemp"

# Optional: export Marketplace bundles into a separate dir.
# marketplace_dir = "build/Marketplace"

# Optional booleans (all default false):
# force_rebuild = true      # -rebuild   force all conversions
# mirror_output = true      # -mirroring delete stale outputs
# prefer_steam  = true      # -forcesteam pick Steam install over MS Store"#
                />

                <CodeBlock
                    language="bash"
                    title="build the packages"
                    code=r#"infinity-msfs package                       # all configured entries
infinity-msfs package --only aerodynamics-dc10
infinity-msfs package --rebuild --marketplace build/Marketplace
infinity-msfs package -v                    # stream fspackagetool output"#
                />

                <Callout
                    tone=Tone::Warning
                    icon=icons::TRIANGLE_ALERT
                    title="Windows + full SDK only"
                >
                    "The SDK fetched by "
                    <InlineCode>"infinity-msfs sdk install"</InlineCode>
                    " only contains the WASM + SimConnect subset. For "
                    <InlineCode>"infinity-msfs package"</InlineCode>
                    " you need the "
                    <em>"full"</em>
                    " SDK installed via the official installer; point "
                    <InlineCode>"MSFS2024_SDK"</InlineCode>
                    " at its root and the CLI will locate "
                    <InlineCode>"Tools/bin/fspackagetool.exe"</InlineCode>
                    " automatically."
                </Callout>
            }
        },
    )
    .numbered("4.0")
    .summary("infinity-msfs package wraps fspackagetool.exe to compile project XMLs (aircraft, scenery, marketplace exports) into ready-to-install packages. Windows + MSFS 2024 required.")
}

fn watch() -> DocSection {
    DocSection::new("watch", "Watch mode", || {
        view! {
            <P>
                <InlineCode>"watch"</InlineCode>
                " walks "
                <InlineCode>"cargo metadata"</InlineCode>
                " to discover every workspace-local crate's "
                <InlineCode>"src/"</InlineCode>
                " dir, plus the parent of each JS instrument "
                <InlineCode>"index"</InlineCode>
                " entry. Events inside "
                <InlineCode>"target/"</InlineCode>
                ", configured output dirs, and any "
                <InlineCode>"[[sim_packages]]"</InlineCode>
                " "
                <InlineCode>"output_dir"</InlineCode>
                " / "
                <InlineCode>"temp_dir"</InlineCode>
                " are filtered out so a successful build doesn't trigger itself."
            </P>

            <CodeBlock
                language="bash"
                title="watch"
                code=r#"infinity-msfs watch                    # rebuild everything on change
infinity-msfs watch --release          # release profile
infinity-msfs watch --only demo-gauge  # restrict to one entry (rust or js)
infinity-msfs watch --rust-only        # skip JS pipeline
infinity-msfs watch --js-only          # skip cargo pipeline
infinity-msfs watch --debounce 500     # collapse events within 500ms (default 300)"#
            />

            <List>
                <ListItem>
                    "The first build runs immediately on startup so you see a known-good baseline before edits start flowing in."
                </ListItem>
                <ListItem>
                    "Build errors don't kill the watcher — the failure is printed and the loop keeps waiting for the next change."
                </ListItem>
                <ListItem>
                    <InlineCode>"infinity-msfs.toml"</InlineCode>
                    " changes are picked up via the workspace Cargo.toml watcher; restart explicitly if you're debugging the watcher discovery itself."
                </ListItem>
            </List>
        }
    })
    .numbered("5.0")
    .summary("infinity-msfs watch debounces filesystem events and re-runs the unified build (rust + js) on change.")
}

fn create() -> DocSection {
    DocSection::new("create", "Project scaffolding (create)", || {
        view! {
            <P>
                "Templates are embedded directly in the binary, so "
                <InlineCode>"cargo install infinity-msfs"</InlineCode>
                " is enough — no separate clone needed. Each template ships a "
                <InlineCode>"template.toml"</InlineCode>
                " manifest declaring its display name, description, prompts, and token map."
            </P>

            <CodeBlock
                language="bash"
                title="interactive"
                code=r#"infinity-msfs create               # prompts for path + template + name
infinity-msfs create my-aircraft   # path supplied; still prompts for template"#
            />

            <CodeBlock
                language="bash"
                title="scripted"
                code=r#"infinity-msfs create my-aircraft \
  --template rust-wasm-gauge \
  --name infinity-demo-gauge \
  --no-input

# overwrite a non-empty target dir
infinity-msfs create . --template ts-react --name pfd --force"#
            />

            <List>
                <ListItem>
                    <Term>
                        <InlineCode>"rust-wasm-gauge"</InlineCode>
                    </Term>
                    " — cdylib MSFS gauge built to "
                    <InlineCode>"wasm32-wasip1"</InlineCode>
                    ", using infinity-rs."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"rust-wasm-system"</InlineCode>
                    </Term>
                    " — cdylib MSFS system module, infinity-rs."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"rust-native"</InlineCode>
                    </Term>
                    " — host-target Rust binary linked against SimConnect. Windows-only at link time."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"rescript-react"</InlineCode>
                    </Term>
                    " — React-based MSFS instrument written in ReScript, using "
                    <InlineCode>"@infinity-msfs/rescript-msfs"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"ts-react"</InlineCode>
                    </Term>
                    " — plain TypeScript + React MSFS instrument bundled via rolldown."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"ts-framework"</InlineCode>
                    </Term>
                    " — TypeScript Avionics-Framework "
                    <InlineCode>"BaseInstrument"</InlineCode>
                    " gauge using FSComponent."
                </ListItem>
            </List>

            <Callout tone=Tone::Accent icon=icons::INFO title="Token substitution">
                "All templates use the same placeholder convention. The renderer replaces "
                <InlineCode>"replace-me"</InlineCode>
                " (kebab), "
                <InlineCode>"replace_me"</InlineCode>
                " (snake), and "
                <InlineCode>"ReplaceMe"</InlineCode>
                " (pascal) with the project name in the matching case — across file contents AND directory names. Rust gauge / system templates also prompt for an "
                <InlineCode>"exported symbol"</InlineCode>
                " so "
                <InlineCode>"MyGauge"</InlineCode>
                " / "
                <InlineCode>"my_gauge"</InlineCode>
                " can differ from the project name when needed."
            </Callout>
        }
    })
    .numbered("6.0")
    .summary("infinity-msfs create scaffolds a new project from a built-in template. Picker + prompts, or fully scripted.")
}

fn cli_reference() -> DocSection {
    DocSection::new("cli-reference", "CLI reference", || {
        view! {
            <P>
                <InlineCode>"build"</InlineCode>
                " runs the full pipeline: "
                <InlineCode>"[[rust.packages]]"</InlineCode>
                " first, then "
                <InlineCode>"[[js.instruments]]"</InlineCode>
                ". Pre/post hooks fire once around the whole thing. "
                <InlineCode>"--only NAME"</InlineCode>
                " matches both "
                <InlineCode>"cargo_package"</InlineCode>
                " AND "
                <InlineCode>"js.instruments.name"</InlineCode>
                "; pass it multiple times to allow several names. Use "
                <InlineCode>"--rust-only"</InlineCode>
                " / "
                <InlineCode>"--js-only"</InlineCode>
                " to skip a side entirely."
            </P>

            <CodeBlock
                language="bash"
                title="build"
                code=r#"infinity-msfs build                            # rust then js, all entries
infinity-msfs build --release                  # optimized rust + minified-capable js
infinity-msfs build --only demo-gauge          # exact-match filter (rust or js)
infinity-msfs build --only demo-gauge --only pfd
infinity-msfs build --rust-only                # skip [[js.instruments]]
infinity-msfs build --js-only                  # skip [[rust.packages]]
infinity-msfs build --no-wasm-opt              # skip the wasm-opt pass
infinity-msfs build --minify                   # minify bundled JS
infinity-msfs build --sourcemap inline         # inline | external | file
infinity-msfs build --skip-simulator-package   # raw rolldown bundle, no harness
infinity-msfs build -v                         # stream subprocess output"#
            />

            <CodeBlock
                language="bash"
                title="package (Windows)"
                code=r#"infinity-msfs package                       # invoke fspackagetool for every entry
infinity-msfs package --only aerodynamics-dc10
infinity-msfs package --rebuild
infinity-msfs package --marketplace build/Marketplace
infinity-msfs package --force-steam
infinity-msfs package -v                    # stream fspackagetool output"#
            />

            <CodeBlock
                language="bash"
                title="watch"
                code=r#"infinity-msfs watch                              # full pipeline on change
infinity-msfs watch --rust-only
infinity-msfs watch --js-only
infinity-msfs watch --release --only demo-gauge --debounce 500"#
            />

            <CodeBlock
                language="bash"
                title="create"
                code=r#"infinity-msfs create                # interactive picker + prompts
infinity-msfs create my-aircraft -t rust-wasm-gauge -n infinity-demo --no-input"#
            />

            <CodeBlock
                language="bash"
                title="introspection"
                code=r#"infinity-msfs projects     # list every configured rust + js entry
infinity-msfs doctor       # environment pre-flight check
infinity-msfs sdk path     # print the resolved SDK cache path
infinity-msfs sdk install  # download / refresh the SDK headers + libs
infinity-msfs sdk remove   # clear the cache"#
            />

            <List>
                <ListItem>
                    <Term>"Pre/post hooks."</Term>
                    " Add "
                    <InlineCode>"[hooks] pre = [...]"</InlineCode>
                    " / "
                    <InlineCode>"post = [...]"</InlineCode>
                    " for codegen, asset processing, or packaging steps that should run around every build. Hooks fire once around the unified pipeline, not per side."
                </ListItem>
                <ListItem>
                    <Term>"Stats."</Term>
                    " Each finish line shows artefact size, wall time, and a +/- size delta versus the previous build (stored in "
                    <InlineCode>"target/.infinity-msfs-stats.json"</InlineCode>
                    "). Useful for catching accidental bundle bloat in PRs."
                </ListItem>
                <ListItem>
                    <Term>
                        "Honors "
                        <InlineCode>"MSFS2024_SDK"</InlineCode>
                        "."
                    </Term>
                    " If you have the full SDK installed locally, point the env var at it and the build tool skips the cached-headers download. Required for "
                    <InlineCode>"infinity-msfs package"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>"--only matches both sides."</Term>
                    " The same flag filters "
                    <InlineCode>"[[rust.packages]].cargo_package"</InlineCode>
                    " and "
                    <InlineCode>"[[js.instruments]].name"</InlineCode>
                    ". If your filter hits only one side, the other still runs (unless gated by "
                    <InlineCode>"--rust-only"</InlineCode>
                    " / "
                    <InlineCode>"--js-only"</InlineCode>
                    ")."
                </ListItem>
            </List>

            <Callout tone=Tone::Accent icon=icons::INFO title="Reference projects">
                <List>
                    <ListItem>
                        <a href=REPO_BUILD_TOOLS target="_blank" rel="noreferrer" class=LINK>
                            "infinity-msfs-build-tools"
                        </a>
                        " — the CLI source."
                    </ListItem>
                    <ListItem>
                        <a href=REPO_INFINITY_RS target="_blank" rel="noreferrer" class=LINK>
                            "infinity-rs"
                        </a>
                        " — the Rust toolkit consumed by WASM gauges and native SimConnect clients."
                    </ListItem>
                    <ListItem>
                        <a href=REPO_RESCRIPT_MSFS target="_blank" rel="noreferrer" class=LINK>
                            "rescript-msfs"
                        </a>
                        " — canonical "
                        <InlineCode>"rescriptReact"</InlineCode>
                        " example with hook layer + benches."
                    </ListItem>
                </List>
            </Callout>
        }
    })
    .numbered("7.0")
    .summary("The commands you'll use day-to-day plus the flags that matter when something goes sideways.")
}
