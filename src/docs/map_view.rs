use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("map-view", "Map View", "Module · map_view", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Off-screen map renderer for ND/MFD overlays. Configurable view mode, altitude colorisation, weather radar cone, and direct integration with NanoVG image patterns.",
        )
        .tags(["MapView", "NavDisplay", "WeatherRadar", "Texture"])
        .section(lifecycle())
        .section(view_and_altitude())
        .section(weather())
        .section(rendering())
        .section(size_and_visibility())
}

fn lifecycle() -> DocSection {
    DocSection::new("lifecycle", "Lifecycle", || {
        view! {
            <Lead>
                <InlineCode>"MapView"</InlineCode>
                " wraps an "
                <InlineCode>"FsTextureId"</InlineCode>
                " issued by "
                <InlineCode>"fsMapViewCreate"</InlineCode>
                ". Drop releases it via "
                <InlineCode>"fsMapViewDelete"</InlineCode>
                "."
            </Lead>

            <CodeBlock
                language="rust"
                title="construct"
                code=r#"use infinity_rs::prelude::*;

let map = MapView::new(ctx, 512, 512).expect("alloc failed");
let texture_id = map.texture_id();         // FsTextureId — useful with NVG

// ... later, drop(map) frees the texture"#
            />
        }
    })
    .numbered("1.0")
    .summary("Allocate once with a context + size; drop to release. Re-create on size changes (the texture is fixed-size).")
}

fn view_and_altitude() -> DocSection {
    DocSection::new(
        "view-and-altitude",
        "View mode & altitude colorisation",
        || {
            view! {
                <CodeBlock
                    language="rust"
                    title="view modes"
                    code=r#"use infinity_rs::prelude::*;
use infinity_rs::map_view::{ViewMode, AltitudeReference, View3DOrientation};

map.set_view_mode(ViewMode::Top);                  // or ViewMode::Altitude
map.set_view_3d_orientation(View3DOrientation::TrackUp);
map.set_altitude_reference(AltitudeReference::AGL);
map.set_altitude_range_in_feet(0.0, 12_000.0);

map.set_altitude_color_list(&[
    (0,    Color::rgb( 30,  60,  10)),
    (3000, Color::rgb(120, 110,  40)),
    (6000, Color::rgb(180, 100,  60)),
    (9000, Color::rgb(200, 200, 200)),
]);"#
                />
            }
        },
    )
    .numbered("2.0")
    .summary("Switch between top-down and 3D, and configure terrain altitude colour ramps.")
}

fn weather() -> DocSection {
    DocSection::new("weather", "Weather radar", || {
        view! {
            <CodeBlock
                language="rust"
                title="weather radar"
                code=r#"use infinity_rs::prelude::*;
use infinity_rs::map_view::{WeatherRadarMode, RainRateColor};

map.set_weather_radar_mode(WeatherRadarMode::Precipitation);
map.set_weather_radar_cone_angle_in_radians(60_f32.to_radians());
map.set_weather_radar_visibility(true);

map.set_weather_radar_rain_colors(&[
    RainRateColor { rain_rate: 0.0, color: Color::rgb(  0,   0,   0) },
    RainRateColor { rain_rate: 1.0, color: Color::rgb( 40, 200,  60) },
    RainRateColor { rain_rate: 4.0, color: Color::rgb(255, 220,  20) },
    RainRateColor { rain_rate: 8.0, color: Color::rgb(220,  40,  40) },
]);"#
            />

            <Callout
                tone=Tone::Accent
                icon=icons::INFO
                title="Layout-compatible colours"
            >
                <InlineCode>"RainRateColor"</InlineCode>
                " is "
                <InlineCode>"#[repr(C)]"</InlineCode>
                " with the same layout as "
                <InlineCode>"FsRainRateColor"</InlineCode>
                ", so a "
                <InlineCode>"&[RainRateColor]"</InlineCode>
                " is passed straight through to the C API."
            </Callout>
        }
    })
    .numbered("3.0")
    .summary("Cone angle + rain rate colour ramp + visibility toggles.")
}

fn rendering() -> DocSection {
    DocSection::new("rendering", "Rendering with NanoVG", || {
        view! {
            <CodeBlock
                language="rust"
                title="render"
                code=r#"use infinity_rs::prelude::*;

let paint = map.image_pattern(nvg, x, y, w, h, /* angle */ 0.0, /* alpha */ 1.0);
Shape::rect(x, y, w, h).fill(paint).draw(nvg);"#
            />

            <P>
                "If you want raw control, "
                <InlineCode>"map.texture_id()"</InlineCode>
                " hands you the "
                <InlineCode>"FsTextureId"</InlineCode>
                " directly so you can call "
                <InlineCode>"nvgImagePattern"</InlineCode>
                " yourself."
            </P>
        }
    })
    .numbered("4.0")
    .summary("Use the texture id directly as an NVG image pattern.")
}

fn size_and_visibility() -> DocSection {
    DocSection::new(
        "size-and-visibility",
        "Size, isolines, background, visibility",
        || {
            view! {
                <List>
                    <ListItem>
                        <Term>
                            <InlineCode>"set_size(width, height)"</InlineCode>
                        </Term>
                        " — resize the underlying texture."
                    </ListItem>
                    <ListItem>
                        <Term>
                            <InlineCode>"set_visibility(bool)"</InlineCode>
                        </Term>
                        " — toggle the entire view."
                    </ListItem>
                    <ListItem>
                        <Term>
                            <InlineCode>"set_background_color(Color)"</InlineCode>
                        </Term>
                        " — pre-fill colour."
                    </ListItem>
                    <ListItem>
                        <Term>
                            <InlineCode>"set_map_isolines_visibility(bool)"</InlineCode>
                        </Term>
                        " — show/hide topo isolines."
                    </ListItem>
                </List>
            }
        },
    )
    .numbered("5.0")
    .summary("The full property surface in one place.")
}
