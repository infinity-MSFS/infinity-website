use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new(
        "gauges-systems",
        "Gauges & Systems",
        "Module · modules",
        "Modules",
    )
    .scope("infinity-rs")
    .tagline(
        "Two lifecycle traits — `Gauge` and `System` — and two macros (`export_gauge!` / `export_system!`) that wire them up to the C ABI MSFS expects. Every gauge or system module in your aircraft package is built around one of these.",
    )
    .tags(["Gauge", "System", "ABI", "Lifecycle"])
    .section(overview())
    .section(trait_shapes())
    .section(exporting())
    .section(context_and_install())
}

fn overview() -> DocSection {
    DocSection::new("overview", "Overview", || {
        view! {
            <Lead>
                "MSFS modules come in two flavours: "
                <Term>"gauges"</Term>
                " (visual cdylibs that draw and react to mouse input) and "
                <Term>"systems"</Term>
                " (logic-only modules that tick every frame). "
                <InlineCode>"infinity-rs"</InlineCode>
                " models both as plain Rust traits with "
                <InlineCode>"init"</InlineCode>
                " / "
                <InlineCode>"update"</InlineCode>
                " / "
                <InlineCode>"kill"</InlineCode>
                " hooks; the "
                <InlineCode>"export_*!"</InlineCode>
                " macros generate the FFI shims."
            </Lead>

            <List>
                <ListItem>
                    <Term>"One trait impl per module."</Term>
                    " The macros take a ctor expression that builds your initial state."
                </ListItem>
                <ListItem>
                    <Term>
                        "Returning "
                        <InlineCode>"false"</InlineCode>
                    </Term>
                    " from any callback signals MSFS to stop calling the rest of the lifecycle."
                </ListItem>
                <ListItem>
                    <Term>"Single-threaded."</Term>
                    " All callbacks run on the sim thread; you don't need "
                    <InlineCode>"Send"</InlineCode>
                    " bounds for state."
                </ListItem>
            </List>
        }
    })
    .numbered("1.0")
    .summary("Where these traits fit in the MSFS module model.")
}

fn trait_shapes() -> DocSection {
    DocSection::new("trait-shapes", "Trait shapes", || {
        view! {
            <CodeBlock
                language="rust"
                title="src/modules.rs"
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

            <P>
                <InlineCode>"Context"</InlineCode>
                " exposes the host gauge context (used by NanoVG and resource APIs). "
                <InlineCode>"SystemInstall"</InlineCode>
                " / "
                <InlineCode>"GaugeInstall"</InlineCode>
                " are bindgen-generated structs forwarded from the install-time data. "
                <InlineCode>"GaugeDraw"</InlineCode>
                " carries the framebuffer dimensions for the current draw call."
            </P>
        }
    })
    .numbered("2.0")
    .summary("Method signatures for both lifecycle traits.")
}

fn exporting() -> DocSection {
    DocSection::new("exporting", "Exporting modules", || {
        view! {
            <CodeBlock
                language="rust"
                title="export_system!"
                code=r#"pub struct MyTimerSystem { accum: f32 }

impl MyTimerSystem {
    pub fn new() -> Self { Self { accum: 0.0 } }
}

impl System for MyTimerSystem {
    fn init(&mut self, _ctx: &Context, _i: &SystemInstall) -> bool { true }
    fn update(&mut self, _ctx: &Context, dt: f32) -> bool {
        self.accum += dt;
        if self.accum >= 1.0 { self.accum = 0.0; /* tick */ }
        true
    }
    fn kill(&mut self, _ctx: &Context) -> bool { true }
}

infinity_rs::export_system!(
    name  = my_timer,
    state = MyTimerSystem,
    ctor  = MyTimerSystem::new(),
);"#
            />

            <CodeBlock
                language="rust"
                title="export_gauge!"
                code=r#"infinity_rs::export_gauge!(
    name  = my_gauge,
    state = MyGauge,
    ctor  = MyGauge::new(),
);"#
            />

            <Callout tone=Tone::Warning icon=icons::TRIANGLE_ALERT title="Crate type">
                "The crate must be built as "
                <InlineCode>"crate-type = [\"cdylib\"]"</InlineCode>
                " for the generated entry points to be exported."
            </Callout>
        }
    })
    .numbered("3.0")
    .summary("The macros that bridge your trait impl to the MSFS C ABI.")
}

fn context_and_install() -> DocSection {
    DocSection::new("context-and-install", "Context & install structs", || {
        view! {
            <P>
                <InlineCode>"Context"</InlineCode>
                " wraps an "
                <InlineCode>"FsContext"</InlineCode>
                " handle. You'll most often pass it through to "
                <InlineCode>"NvgContext::new"</InlineCode>
                ", "
                <InlineCode>"MapView::new"</InlineCode>
                ", or other resource constructors that need a host context."
            </P>

            <P>
                "The install structs are bindgen aliases: "
                <InlineCode>"SystemInstall"</InlineCode>
                " = "
                <InlineCode>"sSystemInstallData"</InlineCode>
                ", "
                <InlineCode>"GaugeInstall"</InlineCode>
                " = "
                <InlineCode>"sGaugeInstallData"</InlineCode>
                ", "
                <InlineCode>"GaugeDraw"</InlineCode>
                " = "
                <InlineCode>"sGaugeDrawData"</InlineCode>
                ". They expose the C-side install/draw fields (window size, framebuffer dimensions, scale factor) directly."
            </P>
        }
    })
    .numbered("4.0")
    .summary("What lives behind the `Context`, `SystemInstall`, and `GaugeInstall` types.")
}
