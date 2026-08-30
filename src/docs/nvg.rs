use infinity_ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("nvg", "NanoVG (drawing)", "Module · nvg", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Idiomatic Rust wrapper over the MSFS NanoVG renderer. Every drawing op routes through `NvgContext`; transforms scope cleanly via closures, paths build via a fluent API, and shapes are reusable Drop-safe descriptors.",
        )
        .tags(["NanoVG", "Drawing", "Path", "Color", "Transform"])
        .section(context())
        .section(frames_and_scope())
        .section(color_and_paint())
        .section(shapes_and_paths())
        .section(text())
        .section(transforms())
        .section(images())
}

fn context() -> DocSection {
    DocSection::new("context", "Context", || {
        view! {
            <Lead>
                <InlineCode>"NvgContext"</InlineCode>
                " wraps an "
                <InlineCode>"NVGcontext*"</InlineCode>
                " created from your gauge's "
                <InlineCode>"FsContext"</InlineCode>
                ". Build it once in "
                <InlineCode>"init"</InlineCode>
                ", store it in your gauge state, draw inside "
                <InlineCode>"frame"</InlineCode>
                " closures from "
                <InlineCode>"draw"</InlineCode>
                "."
            </Lead>

            <CodeBlock
                language="rust"
                title="construction"
                code=r#"use infinity_rs::prelude::*;

struct AttitudeGauge { nvg: Option<NvgContext> }

impl Gauge for AttitudeGauge {
    fn init(&mut self, ctx: &Context, _i: &mut GaugeInstall) -> bool {
        self.nvg = Some(NvgContext::new(ctx).expect("nvg create"));
        true
    }

    fn draw(&mut self, _ctx: &Context, draw: &mut GaugeDraw) -> bool {
        let Some(nvg) = &self.nvg else { return false };
        nvg.frame(draw.winWidth as f32, draw.winHeight as f32, draw.fbWidth as f32 / draw.winWidth as f32, |nvg| {
            // … draw …
        });
        true
    }

    fn kill(&mut self, _ctx: &Context) -> bool { self.nvg = None; true }
    fn update(&mut self, _ctx: &Context, _dt: f32) -> bool { true }
}"#
            />
        }
    })
    .numbered("1.0")
    .summary("Construct once per gauge in `init`. Holds the `NVGcontext*` and frees it on drop.")
}

fn frames_and_scope() -> DocSection {
    DocSection::new("frames-and-scope", "Frames & scoped state", || {
        view! {
            <CodeBlock
                language="rust"
                title="frames"
                code=r#"nvg.frame(win_w, win_h, px_ratio, |nvg| {
    nvg.scoped(|nvg| {
        nvg.translate(win_w * 0.5, win_h * 0.5);
        nvg.rotate(bank.to_radians());
        // ... drawing inside this scope is local
    });
    // state restored here
});"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Scoped is your friend">
                "Don't reach for "
                <InlineCode>"save"</InlineCode>
                " /"
                <InlineCode>"restore"</InlineCode>
                " by hand — the closure form pairs them and is panic-safe."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("`frame` brackets one render. `scoped` saves/restores transform + style state automatically.")
}

fn color_and_paint() -> DocSection {
    DocSection::new("color-and-paint", "Colour & paint", || {
        view! {
            <CodeBlock
                language="rust"
                title="color & paints"
                code=r#"use infinity_rs::nvg::{Color, Gradient, ImagePattern, FillStyle};

let blue   = Color::rgb(0, 191, 255);
let amber  = Color::rgba(255, 191, 0, 200);
let warm   = Color::lerp(blue, amber, 0.4);

let grad = Gradient::linear(0.0, 0.0, 0.0, 100.0, blue, amber);
nvg.fill_paint(grad);

let img_paint = ImagePattern::new(nvg, x, y, w, h, /* angle */ 0.0, image_id, 1.0);
Shape::rect(x, y, w, h).fill(FillStyle::from(img_paint)).draw(nvg);"#
            />
        }
    })
    .numbered("3.0")
    .summary("`Color` plus `FillStyle` / `Gradient` / `ImagePattern` for fills and strokes.")
}

fn shapes_and_paths() -> DocSection {
    DocSection::new("shapes-and-paths", "Shapes & paths", || {
        view! {
            <CodeBlock
                language="rust"
                title="shapes"
                code=r#"use infinity_rs::nvg::{Shape, Color, Winding};

Shape::rect(0.0, 0.0, 200.0, 100.0)
    .fill(Color::rgb(20, 30, 40))
    .draw(nvg);

Shape::circle(100.0, 50.0, 40.0)
    .stroke(Color::rgb(255, 255, 0), 4.0)
    .draw(nvg);

Shape::rounded_rect(0.0, 0.0, 200.0, 100.0, 16.0)
    .fill(Color::rgb(40, 80, 120))
    .draw(nvg);"#
            />

            <CodeBlock
                language="rust"
                title="paths"
                code=r#"use infinity_rs::nvg::{Color, Winding, deg_to_rad};

nvg.stroke_color(Color::rgb(255, 255, 0));
nvg.stroke_width(15.0);
nvg.begin_path();
nvg.move_to(-50.0, 0.0);
nvg.line_to(-15.0, 0.0);
nvg.arc(0.0, 0.0, 15.0, deg_to_rad(180.0), 0.0, Winding::Ccw);
nvg.line_to(50.0, 0.0);
nvg.stroke();"#
            />
        }
    })
    .numbered("4.0")
    .summary("Reusable `Shape` descriptors for primitives; `PathBuilder` for free-form geometry.")
}

fn text() -> DocSection {
    DocSection::new("text", "Text", || {
        view! {
            <CodeBlock
                language="rust"
                title="text"
                code=r#"let font_id = nvg.create_font("sans", "./data/Roboto-Regular.ttf").unwrap();
nvg.font_face_id(font_id);
nvg.font_size(28.0);
nvg.fill_color(Color::rgb(255, 255, 255));
nvg.text(20.0, 40.0, "Hello, sim");

let bounds = nvg.text_bounds(0.0, 0.0, "Hello, sim");
let metrics = nvg.text_metrics();              // ascender / descender / lineh

// Layout a paragraph
let rows: Vec<TextRow> = nvg.text_break_lines("Lorem ipsum…", 200.0, 5);"#
            />
        }
    })
    .numbered("5.0")
    .summary("Register fonts, measure, lay out into rows.")
}

fn transforms() -> DocSection {
    DocSection::new("transforms", "Transforms", || {
        view! {
            <CodeBlock
                language="rust"
                title="transform"
                code=r#"use infinity_rs::nvg::{Transform, deg_to_rad, rad_to_deg};

let t = Transform::from_translate(100.0, 50.0)
    .rotate(deg_to_rad(45.0))
    .scale(2.0);

let pt = t.apply(10.0, 0.0);          // (x, y) after the chain
let inv = t.inverse();"#
            />
        }
    })
    .numbered("6.0")
    .summary("`Transform` is a stand-alone matrix type for composing transforms outside of the nvg context.")
}

fn images() -> DocSection {
    DocSection::new("images", "Images", || {
        view! {
            <CodeBlock
                language="rust"
                title="images"
                code=r#"let img      = nvg.create_image("./data/logo.png", ImageFlags::NONE);
let img_mem  = nvg.create_image_mem(ImageFlags::NONE, &png_bytes);
let img_rgba = nvg.create_image_rgba(width, height, ImageFlags::NONE, &rgba_buffer);

let (w, h) = nvg.image_size(img.unwrap());
nvg.delete_image(img.unwrap());"#
            />
        }
    })
    .numbered("7.0")
    .summary(
        "Load images from disk, RGBA buffers, or in-memory blobs; query size; delete when done.",
    )
}
