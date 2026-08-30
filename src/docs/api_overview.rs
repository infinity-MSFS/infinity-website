use infinity_ui::prelude::*;

use super::prose::{DocCard, Grid2, Term};

pub fn doc() -> Doc {
    Doc::new("api-overview", "API Overview", "Reference", "Reference")
        .scope("infinity-rs")
        .tagline(
            "A fast tour of every module in infinity-rs — what it does, how it's shaped, and a representative snippet you can copy. Each module also has its own deep-dive page in the picker above. The crate splits cleanly into a WASM-host surface (gated by the `wasm` feature, default-on) and a SimConnect surface that works on both wasm and native Windows.",
        )
        .tags(["Modules", "Traits", "Macros", "Idioms"])
        .section(crate_shape())
        .section(modules_and_systems())
        .section(vars())
        .section(comm_bus())
        .section(flow())
        .section(events())
        .section(io())
        .section(network())
        .section(charts())
        .section(planned_route())
        .section(map_view())
        .section(vfx())
        .section(nvg())
        .section(simconnect())
        .section(idioms())
}

fn crate_shape() -> DocSection {
    DocSection::new("crate-shape", "Crate shape", || {
        view! {
            <Lead>
                "Two cargo features carve the surface area: "
                <InlineCode>"wasm"</InlineCode>
                " (default-on) gates the MSFS WASM-host APIs (NanoVG, comm bus, IO, vars, etc.). "
                <InlineCode>"simconnect"</InlineCode>
                " opts into the SimConnect bindings, which work under either feature on either target."
            </Lead>

            <Grid2>
                <DocCard eyebrow="default" title="WASM gauge / system">
                    <P>
                        "Pull the crate with default features. "
                        <InlineCode>"prelude::*"</InlineCode>
                        " imports everything you need to write a "
                        <InlineCode>"Gauge"</InlineCode>
                        " or "
                        <InlineCode>"System"</InlineCode>
                        "."
                    </P>
                </DocCard>
                <DocCard eyebrow="default-features = false" title="Native SimConnect">
                    <P>
                        "Disable defaults and opt into "
                        <InlineCode>"simconnect"</InlineCode>
                        ". The wasm-host modules are excluded so the native binary doesn't pull in unresolved gauge-runtime symbols."
                    </P>
                </DocCard>
            </Grid2>

            <CodeBlock
                language="toml"
                title="feature pairs"
                code=r#"# WASM gauge — default
infinity-rs = "0.2.8"

# WASM gauge that also wants SimConnect (resolved by the host runtime)
infinity-rs = { version = "0.2.8", features = ["simconnect"] }

# Native SimConnect-only client (Windows for linking)
infinity-rs = { version = "0.2.8", default-features = false, features = ["simconnect"] }"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Always-on modules">
                <InlineCode>"sys"</InlineCode>
                ", "
                <InlineCode>"types"</InlineCode>
                ", and "
                <InlineCode>"abi"</InlineCode>
                " compile under every configuration — they're plain bindgen types and ABI shims with no runtime dependencies."
            </Callout>
        }
    })
    .numbered("0.0")
    .summary("The features you opt into determine which modules are compiled. This matters most for native consumers, who typically disable defaults.")
}

fn modules_and_systems() -> DocSection {
    DocSection::new("modules-and-systems", "Gauges & Systems", || {
        view! {
            <P>
                <InlineCode>"Gauge"</InlineCode>
                " and "
                <InlineCode>"System"</InlineCode>
                " are the two lifecycle traits. Returning "
                <InlineCode>"false"</InlineCode>
                " from any callback signals the host to stop calling you."
            </P>

            <CodeBlock
                language="rust"
                title="modules.rs"
                code=r#"pub trait System: 'static {
    fn init(&mut self, ctx: &Context, install: &SystemInstall) -> bool;
    fn update(&mut self, ctx: &Context, dt: f32) -> bool;
    fn kill(&mut self, ctx: &Context) -> bool;
}

pub trait Gauge: 'static {
    fn init(&mut self, ctx: &Context, install: &mut GaugeInstall) -> bool;
    fn update(&mut self, ctx: &Context, dt: f32) -> bool;
    fn draw(&mut self, ctx: &Context, draw: &mut GaugeDraw) -> bool;
    fn kill(&mut self, ctx: &Context) -> bool;
    fn mouse(&mut self, _ctx: &Context, _x: f32, _y: f32, _flags: i32) {}
}"#
            />

            <CodeBlock
                language="rust"
                title="exports.rs macros"
                code=r#"infinity_rs::export_system!(name = my_system, state = MySystem, ctor = MySystem::new());
infinity_rs::export_gauge!(name = my_gauge,  state = MyGauge,  ctor = MyGauge::new());"#
            />
        }
    })
    .numbered("1.0")
    .summary("Two traits that map onto the MSFS gauge / system entry points. The `export_*!` macros wire them up to the C ABI.")
}

fn vars() -> DocSection {
    DocSection::new("vars", "Vars (A-Vars & L-Vars)", || {
        view! {
            <P>
                "All variable IDs are looked up once at construction; subsequent "
                <InlineCode>"get"</InlineCode>
                " / "
                <InlineCode>"set"</InlineCode>
                " calls go through the cached handle. Indexed and target-scoped variants are available without falling out of the high-level API."
            </P>

            <CodeBlock
                language="rust"
                title="vars"
                code=r#"use infinity_rs::prelude::*;
use infinity_rs::sys::FS_OBJECT_ID_USER_AIRCRAFT;

let airspeed = AVar::new("A:AIRSPEED INDICATED", "Knots")?;
let kts = airspeed.get()?;                       // user aircraft, default target
let kts_cur = airspeed.get_target(FS_OBJECT_ID_USER_AIRCRAFT)?;

let eng_rpm = AVar::new("A:GENERAL ENG RPM", "RPM")?;
let eng2 = eng_rpm.get_indexed(2)?;              // index → param array under the hood

let toggle = LVar::new("L:DEMO_TOGGLE", "Bool")?;
toggle.set(1.0)?;"#
            />

            <P>
                "The "
                <InlineCode>"#[derive(VarStruct)]"</InlineCode>
                " macro fans a struct out into a single "
                <InlineCode>"Self::get()"</InlineCode>
                " / "
                <InlineCode>"self.set()"</InlineCode>
                " pair so multi-var snapshots stay declarative."
            </P>

            <CodeBlock
                language="rust"
                title="VarStruct"
                code=r#"#[derive(Debug, Clone, Copy, VarStruct)]
struct Snapshot {
    #[var(name = "A:PLANE ALTITUDE",       unit = "Feet",    kind = "A")]
    altitude_ft: f64,
    #[var(name = "A:GENERAL ENG RPM",      unit = "RPM",     kind = "A", index = 1)]
    eng1_rpm: f64,
    #[var(name = "L:DEMO_CUSTOM",          unit = "Number",  kind = "L")]
    custom: f64,
}

let s = Snapshot::get()?;          // one call → one struct
let _ = s.set();                   // writes LVars; AVar writes are no-ops"#
            />
        }
    })
    .numbered("2.0")
    .summary("`AVar` and `LVar` are unit-typed wrappers over `Var<K>`. Both are `Copy`, so handles can be cloned freely into closures.")
}

fn comm_bus() -> DocSection {
    DocSection::new("comm-bus", "Comm Bus", || {
        view! {
            <CodeBlock
                language="rust"
                title="comm_bus"
                code=r#"use infinity_rs::prelude::*;

let sub = Subscription::subscribe("demo/cmd", move |bytes| {
    // bytes: &[u8] — caller-defined wire format
    let cmd = bytes.first().copied().unwrap_or(0);
    // …handle…
})?;

// publish
commbus_call("demo/state", &[1u8], BroadcastFlags::JS | BroadcastFlags::WASM)?;"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Drop unregisters">
                "Hold the "
                <InlineCode>"Subscription"</InlineCode>
                " in your "
                <InlineCode>"System"</InlineCode>
                " state — when the gauge tears down, the destructor calls "
                <InlineCode>"fsCommBusUnregisterOneEvent"</InlineCode>
                " for you."
            </Callout>
        }
    })
    .numbered("2.1")
    .summary("Wasm ↔ JS ↔ wasm pub/sub bus. Subscribe with a closure; the returned `Subscription` deregisters on drop.")
}

fn flow() -> DocSection {
    DocSection::new("flow", "Flow Events", || {
        view! {
            <CodeBlock
                language="rust"
                title="flow"
                code=r#"use infinity_rs::prelude::*;

let sub = flow_subscribe(|msg| {
    if msg.event.is_pause_boundary()    { /* freeze your sim-clock state */ }
    if msg.event.is_resume_boundary()   { /* resync */ }
    if msg.event.is_shutdown_boundary() { /* persist */ }
})?;"#
            />
        }
    })
    .numbered("2.2")
    .summary("Sim-flow lifecycle hooks (flight start/end, pause boundaries, RTC, replay, teleport, plane crash).")
}

fn events() -> DocSection {
    DocSection::new("events", "Key Events", || {
        view! {
            <CodeBlock
                language="rust"
                title="events"
                code=r#"use infinity_rs::prelude::*;
use infinity_rs::events;

const KEY_TOGGLE_MASTER_BATTERY: u32 = /* … */;

let sub = events::subscribe(|event| {
    if event.id == KEY_TOGGLE_MASTER_BATTERY {
        let battery_index = event.params.first_index().unwrap_or(1);
        // …
    }
});

events::trigger1(KEY_TOGGLE_MASTER_BATTERY, FsParamArg::Index(1))?;"#
            />
        }
    })
    .numbered("2.3")
    .summary("Subscribe to and trigger MSFS key events with typed parameter packs.")
}

fn io() -> DocSection {
    DocSection::new("io", "Async IO", || {
        view! {
            <CodeBlock
                language="rust"
                title="io"
                code=r#"use infinity_rs::io;
use infinity_rs::prelude::*;

let req = io::read("data/state.json", |buf| {
    // buf: &[u8]; called once when the read finishes
    let _ = serde_json::from_slice::<MyState>(buf);
})?;

io::write("data/state.json", b"{\"hello\":1}", |result| {
    let _ = result; // io::WriteResult
})?;"#
            />

            <Callout
                tone=Tone::Warning
                icon=icons::TRIANGLE_ALERT
                title="Hold the request handle"
            >
                "The returned request handle keeps the closure alive. Drop it to abort."
            </Callout>
        }
    })
    .numbered("2.4")
    .summary("Non-blocking sandboxed file IO. Open, read, and write panel-relative files with closures or polling handles.")
}

fn network() -> DocSection {
    DocSection::new("network", "HTTP Network", || {
        view! {
            <CodeBlock
                language="rust"
                title="network"
                code=r#"use infinity_rs::prelude::*;

let params = HttpParams {
    headers: vec!["Accept: application/json".to_string()],
    post_field: None,
    body: vec![],
};

http_request(Method::Get, "https://example.com/data.json", params, move |resp| {
    if resp.error_code == 0 {
        let _ = commbus_call("demo/data", &resp.data, BroadcastFlags::JS);
    }
})?;"#
            />
        }
    })
    .numbered("2.5")
    .summary("Make HTTP requests through MSFS's networking subsystem — bypasses the WASI sandbox.")
}

fn charts() -> DocSection {
    DocSection::new("charts", "Charts", || {
        view! {
            <CodeBlock
                language="rust"
                title="charts"
                code=r#"use infinity_rs::prelude::*;
use infinity_rs::charts;

let icao = charts::make_icao("KSEA");
let index: ChartIndex = charts::request_chart_index(icao)?;

for category in index.categories() {
    for chart in category.charts() {
        println!("{} — {}", chart.identifier(), chart.title());
    }
}"#
            />
        }
    })
    .numbered("2.6")
    .summary("Pull airport / approach / SID / STAR chart indices and pages by ICAO.")
}

fn planned_route() -> DocSection {
    DocSection::new("planned-route", "Planned Route", || {
        view! {
            <CodeBlock
                language="rust"
                title="planned_route"
                code=r#"use infinity_rs::prelude::*;

if let Ok(Some(route)) = current_efb_route() {
    for leg in route.legs() {
        // leg.identifier(), leg.altitude(), leg.leg_type() …
    }
}

let _bcast = planned_route_subscribe_broadcast(|route| {
    // called whenever the EFB pushes a new route
})?;"#
            />
        }
    })
    .numbered("2.7")
    .summary("Read the active EFB route, subscribe to broadcast updates, or respond to in-sim route requests.")
}

fn map_view() -> DocSection {
    DocSection::new("map-view", "Map View", || {
        view! {
            <CodeBlock
                language="rust"
                title="map_view"
                code=r#"use infinity_rs::prelude::*;

let map = MapView::new(ctx, 512, 512)?;
map.set_view_mode(ViewMode::Top);
map.set_altitude_reference(AltitudeReference::AGL);
map.set_weather_radar_mode(WeatherRadarMode::Precipitation);
map.set_weather_radar_cone_angle_in_radians(60_f32.to_radians());"#
            />
        }
    })
    .numbered("2.8")
    .summary("Off-screen map renderer for ND/MFD overlays — terrain altitude colorisation, weather radar cone, view modes.")
}

fn vfx() -> DocSection {
    DocSection::new("vfx", "VFX", || {
        view! {
            <CodeBlock
                language="rust"
                title="vfx"
                code=r#"use infinity_rs::prelude::*;

let smoke = VfxInstance::spawn_on_simobject(
    "/asobo-vfx-misc/smoke_engine_fire",
    SimObjId::user(),
    vec3d(0.0, 0.5, 2.0),
    VfxParam::default(),
)?;

smoke.set_offset(vec3d(0.0, 0.0, 0.0));
// drop(smoke) → fsVfxDestroyInstance"#
            />
        }
    })
    .numbered("2.9")
    .summary("Spawn, control, and tear down particle effects in the world or on a SimObject.")
}

fn nvg() -> DocSection {
    DocSection::new("nvg", "NanoVG (drawing)", || {
        view! {
            <CodeBlock
                language="rust"
                title="nvg"
                code=r#"use infinity_rs::prelude::*;
use infinity_rs::nvg::{Color, NvgContext, ImageFlags, Shape, Winding, deg_to_rad};

let nvg = NvgContext::new(ctx).expect("no nvg ctx");
nvg.frame(win_w, win_h, px_ratio, |nvg| {
    nvg.scoped(|nvg| {
        nvg.translate(win_w * 0.5, win_h * 0.5);
        nvg.rotate(bank.to_radians());

        Shape::rect(-half, -half, size, fh)
            .fill(Color::rgb(0, 191, 255))
            .draw(nvg);
    });

    nvg.stroke_color(Color::rgb(255, 255, 0));
    nvg.stroke_width(15.0);
    nvg.begin_path();
    nvg.move_to(-50.0, 0.0);
    nvg.arc(0.0, 0.0, 25.0, deg_to_rad(180.0), 0.0, Winding::Ccw);
    nvg.line_to(50.0, 0.0);
    nvg.stroke();
});"#
            />
        }
    })
    .numbered("2.10")
    .summary("Idiomatic Rust wrapper over the MSFS NanoVG renderer. Frames, scoped transforms, paths, paints, and text.")
}

fn simconnect() -> DocSection {
    DocSection::new("simconnect", "SimConnect", || {
        view! {
            <P>
                "The same API surface is available on both targets. WASM builds resolve the SimConnect symbols through the host runtime; native Windows builds link against the SimConnect import library bundled with the SDK."
            </P>

            <List>
                <ListItem>
                    <Term>"Connections"</Term>
                    " — "
                    <InlineCode>"SimConnect::open(\"name\")"</InlineCode>
                    " returns an owned client; drop it to disconnect."
                </ListItem>
                <ListItem>
                    <Term>"Data definitions"</Term>
                    " — "
                    <InlineCode>"add_to_data_definition"</InlineCode>
                    " lays out "
                    <InlineCode>"#[repr(C, packed)]"</InlineCode>
                    " structs field-by-field."
                </ListItem>
                <ListItem>
                    <Term>"Requests"</Term>
                    " — "
                    <InlineCode>"request_data_on_sim_object"</InlineCode>
                    " with a "
                    <InlineCode>"Period"</InlineCode>
                    ", "
                    <InlineCode>"DataRequestFlag"</InlineCode>
                    ", and a request-id you choose."
                </ListItem>
                <ListItem>
                    <Term>"Dispatch"</Term>
                    " — "
                    <InlineCode>"sc.dispatch(|msg| match msg { … })"</InlineCode>
                    " drains the message queue through a typed "
                    <InlineCode>"Recv"</InlineCode>
                    " enum."
                </ListItem>
            </List>

            <CodeBlock
                language="rust"
                title="simconnect — minimum viable client"
                code=r#"use infinity_rs::simconnect::{
    DataDefinitionId, DataRequestFlag, DataRequestId, DataType, ObjectId,
    Period, Recv, SimConnect, SimObjectDataView, UNUSED,
};

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
struct AircraftData {
    altitude_ft: f64,
    airspeed_kt: f64,
}

const DEF: DataDefinitionId = DataDefinitionId::new(1);
const REQ: DataRequestId    = DataRequestId::new(1);

let mut sc = SimConnect::open("demo")?;

sc.add_to_data_definition(DEF, "PLANE ALTITUDE",   "feet",
    DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED)?;
sc.add_to_data_definition(DEF, "AIRSPEED INDICATED", "knots",
    DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED)?;

sc.request_data_on_sim_object(
    REQ, DEF, ObjectId::USER,
    Period::SIMCONNECT_PERIOD_SIM_FRAME,
    DataRequestFlag::CHANGED, 0, 0, 0,
)?;

loop {
    sc.dispatch(|msg| match msg {
        Recv::SimObjectData(d)
            if SimObjectDataView(d).dwRequestID() == REQ.raw() =>
        {
            let v: AircraftData = unsafe { SimObjectDataView(d).data_as() };
            println!("alt={} ias={}", v.altitude_ft, v.airspeed_kt);
        }
        Recv::Quit(_) => break,
        Recv::Exception(e) => eprintln!("exception: {:?}", e),
        _ => {}
    })?;
}"#
            />
        }
    })
    .numbered("3.0")
    .summary("Safe wrapper around the SimConnect SDK — works on both WASM (host-resolved) and native Windows.")
}

fn idioms() -> DocSection {
    DocSection::new("idioms", "Idioms & gotchas", || {
        view! {
            <List>
                <ListItem>
                    <Term>"Vars are Copy."</Term>
                    " You can clone an "
                    <InlineCode>"LVar"</InlineCode>
                    " handle into multiple closures freely; they share the underlying registered ID."
                </ListItem>
                <ListItem>
                    <Term>"Subscriptions tear down on drop."</Term>
                    " Hold them in your gauge/system state. Don't "
                    <InlineCode>"let _ = subscribe(…)"</InlineCode>
                    " or the handler immediately deregisters."
                </ListItem>
                <ListItem>
                    <Term>
                        "Native builds need "
                        <InlineCode>"default-features = false"</InlineCode>
                        "."
                    </Term>
                    " Otherwise the WASM-host modules pull in "
                    <InlineCode>"nvg*"</InlineCode>
                    ", "
                    <InlineCode>"fs*"</InlineCode>
                    ", "
                    <InlineCode>"fsCommBus*"</InlineCode>
                    ", etc. as unresolved externs at link time."
                </ListItem>
                <ListItem>
                    <Term>
                        "SimConnect data structs must be "
                        <InlineCode>"#[repr(C, packed)]"</InlineCode>
                        "."
                    </Term>
                    " The field order has to match the "
                    <InlineCode>"add_to_data_definition"</InlineCode>
                    " call order exactly."
                </ListItem>
                <ListItem>
                    <Term>"Recompile-from-sim is unsupported."</Term>
                    " MSFS's Edit-and-Continue feature requires the patched SDK "
                    <InlineCode>"wasm-ld.exe"</InlineCode>
                    " which would lock the toolchain to Windows. Reload the package instead."
                </ListItem>
            </List>
        }
    })
    .numbered("4.0")
    .summary("A short list of things to know that don't fit neatly into a per-module section.")
}
