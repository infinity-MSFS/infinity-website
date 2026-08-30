use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("vfx", "VFX", "Module · vfx", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Spawn, drive, and tear down particle effects in the world or attached to SimObjects. Owned `VfxInstance` handles clean up via `fsVfxDestroyInstance` on drop.",
        )
        .tags(["Particles", "World", "SimObject", "RPN"])
        .section(overview())
        .section(spawn_world())
        .section(spawn_simobject())
        .section(control())
}

fn overview() -> DocSection {
    DocSection::new("overview", "Overview", || {
        view! {
            <Lead>
                "MSFS VFX graphs are authored in the editor and referenced from code by their path (e.g. "
                <InlineCode>"\"/asobo-vfx-misc/smoke_engine_fire\""</InlineCode>
                "). Spawn helpers come in two flavours: world-anchored (lat/long/alt) or sim-object-attached."
            </Lead>

            <List>
                <ListItem>
                    <Term>
                        <InlineCode>"VfxParam"</InlineCode>
                    </Term>
                    " — one binding: "
                    <InlineCode>"name"</InlineCode>
                    " + "
                    <InlineCode>"rpn"</InlineCode>
                    "expression evaluated each frame."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"VfxInstance::leak()"</InlineCode>
                    </Term>
                    " detaches the host instance from the Rust handle so an effect can outlive your gauge."
                </ListItem>
            </List>
        }
    })
    .numbered("1.0")
    .summary("Effect graphs are referenced by string path; runtime params are RPN expressions.")
}

fn spawn_world() -> DocSection {
    DocSection::new("spawn-world", "Spawning in the world", || {
        view! {
            <CodeBlock
                language="rust"
                title="spawn_in_world"
                code=r#"use infinity_rs::prelude::*;

let smoke = VfxInstance::spawn_in_world(
    "/asobo-vfx-misc/smoke_dust",
    vec3d(47.45, -122.31, 800.0),         // lat / long / alt-ft
    vec3d(0.0, 0.0, 0.0),                  // pitch / bank / heading (rad)
    Some(5.0),                             // min emission time (s)
    &[
        VfxParam::new("intensity", "(L:WIND_SPEED, knots) 5 / clamp"),
        VfxParam::new("size",      "1.5"),
    ],
)?;"#
            />
        }
    })
    .numbered("2.0")
    .summary("Pass a lat/long/alt + pitch/bank/heading + parameter list.")
}

fn spawn_simobject() -> DocSection {
    DocSection::new("spawn-simobject", "Spawning on a SimObject", || {
        view! {
            <CodeBlock
                language="rust"
                title="spawn_on_simobject"
                code=r#"use infinity_rs::prelude::*;

let exhaust = VfxInstance::spawn_on_simobject(
    "/asobo-vfx-misc/engine_exhaust",
    SimObjId::user(),
    Some("EXHAUST_LEFT"),                  // node name; None for object origin
    vec3d(0.0, 0.0, 0.0),                  // local offset (m)
    vec3d(0.0, 0.0, 0.0),                  // local rotation (rad)
    None,
    &[],
)?;"#
            />
        }
    })
    .numbered("2.1")
    .summary("Attach to the user aircraft (or any SimObjId), optionally on a named node.")
}

fn control() -> DocSection {
    DocSection::new("control", "Controlling instances", || {
        view! {
            <CodeBlock
                language="rust"
                title="control"
                code=r#"smoke.play();                                  // begin / resume emission
smoke.stop();                                  // halt emission (still alive)
let _ = smoke.is_playing();                    // bool
let _ = smoke.is_min_time_passed();            // emission lifetime elapsed?
let _ = smoke.is_valid();                      // host still recognises the id?

// for sim-object-attached effects
exhaust.set_offset(vec3d(0.0, 0.1, 0.0));
exhaust.set_rotation(vec3d(0.0, 0.0, 0.5));

// for world-spawned effects
smoke.set_world_position(vec3d(47.46, -122.30, 850.0));"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Drop = destroy">
                "Holding the "
                <InlineCode>"VfxInstance"</InlineCode>
                " in your gauge state ties the effect lifetime to the gauge. Call "
                <InlineCode>"VfxInstance::leak()"</InlineCode>
                " if you want the host to manage it independently."
            </Callout>
        }
    })
    .numbered("3.0")
    .summary("Start/stop, query state, update offsets/rotations on the fly.")
}
