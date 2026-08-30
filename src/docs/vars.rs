use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("vars", "Vars (A-Vars & L-Vars)", "Module · vars", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Typed handles over MSFS aircraft variables (A:) and local variables (L:). Both compile down to a single registered ID and a unit ID, so reads/writes are essentially free after construction.",
        )
        .tags(["AVar", "LVar", "VarStruct", "Units"])
        .section(model())
        .section(construction())
        .section(reading())
        .section(writing())
        .section(varstruct())
        .section(param_array())
}

fn model() -> DocSection {
    DocSection::new("model", "Model", || {
        view! {
            <Lead>
                <InlineCode>"Var<K>"</InlineCode>
                " is a "
                <InlineCode>"Copy"</InlineCode>
                " handle bundling a "
                <InlineCode>"K::Id"</InlineCode>
                " (the variable id the host returned at registration time) and a "
                <InlineCode>"UnitId"</InlineCode>
                ". "
                <InlineCode>"AVar"</InlineCode>
                " and "
                <InlineCode>"LVar"</InlineCode>
                " are unit-typed aliases for the two var kinds."
            </Lead>

            <CodeBlock
                language="rust"
                title="vars/mod.rs"
                code=r#"pub struct Var<K: VarKind> { id: K::Id, unit: UnitId, _k: PhantomData<K> }

pub type AVar = Var<AVarKind>;
pub type LVar = Var<LVarKind>;"#
            />

            <List>
                <ListItem>
                    <Term>"Construction"</Term>
                    " hits the host once to look up the var name and unit. After that, every "
                    <InlineCode>"get"</InlineCode>
                    "/"
                    <InlineCode>"set"</InlineCode>
                    " goes through the cached IDs."
                </ListItem>
                <ListItem>
                    <Term>"Copy"</Term>
                    " — "
                    <InlineCode>"let h2 = h1"</InlineCode>
                    " just duplicates the IDs. Useful when capturing a handle into a closure."
                </ListItem>
                <ListItem>
                    <Term>"Errors"</Term>
                    " are "
                    <InlineCode>"VarResult<T>"</InlineCode>
                    "; "
                    <InlineCode>"VarError"</InlineCode>
                    " covers lookup failures, NUL bytes in names, and host-reported access errors."
                </ListItem>
            </List>
        }
    })
    .numbered("1.0")
    .summary("How `Var<K>` wraps a registered ID + unit pair.")
}

fn construction() -> DocSection {
    DocSection::new("construction", "Construction", || {
        view! {
            <CodeBlock
                language="rust"
                title="construction"
                code=r#"use infinity_rs::prelude::*;

let airspeed = AVar::new("A:AIRSPEED INDICATED", "Knots")?;
let altitude = AVar::new("A:PLANE ALTITUDE", "Feet")?;

let toggle   = LVar::new("L:DEMO_TOGGLE", "Bool")?;
let custom   = LVar::new("L:DEMO_CUSTOM", "Number")?;"#
            />

            <Callout
                tone=Tone::Accent
                icon=icons::INFO
                title="Construct once, reuse everywhere"
            >
                "Cache "
                <InlineCode>"Var"</InlineCode>
                " handles in your gauge/system state. Constructing on every frame is wasteful — the lookup is the only expensive part."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("Looking up an A-Var or L-Var by name and unit.")
}

fn reading() -> DocSection {
    DocSection::new("reading", "Reading values", || {
        view! {
            <CodeBlock
                language="rust"
                title="reading"
                code=r#"use infinity_rs::prelude::*;
use infinity_rs::sys::{FS_OBJECT_ID_USER_AIRCRAFT, FS_OBJECT_ID_USER_CURRENT};

// 1) plain get — uses the default target for the var kind
let kts = airspeed.get()?;

// 2) explicit target
let kts_cur = airspeed.get_target(FS_OBJECT_ID_USER_CURRENT)?;

// 3) indexed (e.g. A:GENERAL ENG RPM:n)
let eng_rpm = AVar::new("A:GENERAL ENG RPM", "RPM")?;
let eng2 = eng_rpm.get_indexed(2)?;
let eng2_user = eng_rpm.get_indexed_target(2, FS_OBJECT_ID_USER_AIRCRAFT)?;

// 4) low-level — build your own param array (rarely needed)
let mut param = VarParamArray1::index(1);
let v = eng_rpm.get_with(param.as_raw_mut(), FS_OBJECT_ID_USER_AIRCRAFT)?;"#
            />
        }
    })
    .numbered("2.1")
    .summary("Default target, explicit target, indexed, and low-level variants.")
}

fn writing() -> DocSection {
    DocSection::new("writing", "Writing values", || {
        view! {
            <CodeBlock
                language="rust"
                title="writing"
                code=r#"toggle.set(1.0)?;
toggle.set_target(FS_OBJECT_ID_USER_CURRENT, 1.0)?;

custom.set(42.0)?;"#
            />

            <Callout
                tone=Tone::Warning
                icon=icons::TRIANGLE_ALERT
                title="A-Var writability"
            >
                "Most A-Vars are read-only; the host silently drops writes to those. Stick to L-Vars for state you own."
            </Callout>
        }
    })
    .numbered("2.2")
    .summary("Set L-Vars (and the rare writable A-Var). The same target/index variants apply.")
}

fn varstruct() -> DocSection {
    DocSection::new("varstruct", "#[derive(VarStruct)]", || {
        view! {
            <P>
                "The derive macro turns each "
                <InlineCode>"#[var(...)]"</InlineCode>
                " field into a registered "
                <InlineCode>"OnceLock"</InlineCode>
                "'d handle and emits a single struct-level "
                <InlineCode>"get()"</InlineCode>
                " / "
                <InlineCode>"set()"</InlineCode>
                " pair. Useful for snapshotting many vars at once."
            </P>

            <CodeBlock
                language="rust"
                title="VarStruct"
                code=r#"use infinity_rs::prelude::*;
use msfs_derive::VarStruct;
use infinity_rs::sys::FS_OBJECT_ID_USER_CURRENT;

#[derive(Debug, Clone, Copy, VarStruct)]
struct Snapshot {
    #[var(name = "A:PLANE ALTITUDE",  unit = "Feet",   kind = "A")]
    altitude_ft: f64,

    #[var(name = "A:PLANE HEADING DEGREES TRUE", unit = "Degrees", kind = "A")]
    heading_true_deg: f64,

    #[var(name = "A:GENERAL ENG RPM", unit = "RPM",    kind = "A", index = 1)]
    eng1_rpm: f64,

    #[var(name = "A:AIRSPEED INDICATED", unit = "Knots", kind = "A",
          target = "user_current")]
    ias_user_current: f64,

    #[var(name = "L:DEMO_CUSTOM", unit = "Number", kind = "L")]
    custom: f64,
}

let s = Snapshot::get()?;          // one call
println!("{} ft", s.altitude_ft);

let _ = s.set();                   // writes LVars, no-ops AVars"#
            />

            <List>
                <ListItem>
                    <Term>"Supported attributes:"</Term>
                    " "
                    <InlineCode>"name"</InlineCode>
                    ", "
                    <InlineCode>"unit"</InlineCode>
                    ", "
                    <InlineCode>"kind"</InlineCode>
                    " ("
                    <InlineCode>"\"A\""</InlineCode>
                    " or "
                    <InlineCode>"\"L\""</InlineCode>
                    "), "
                    <InlineCode>"index"</InlineCode>
                    " (optional), "
                    <InlineCode>"target"</InlineCode>
                    " (optional; "
                    <InlineCode>"\"user_aircraft\""</InlineCode>
                    ", "
                    <InlineCode>"\"user_avatar\""</InlineCode>
                    ", "
                    <InlineCode>"\"user_current\""</InlineCode>
                    ")."
                </ListItem>
                <ListItem>
                    <Term>"Field type"</Term>
                    " must currently be "
                    <InlineCode>"f64"</InlineCode>
                    "."
                </ListItem>
            </List>
        }
    })
    .numbered("3.0")
    .summary("Generate a `Self::get()` / `self.set()` pair that fans out across many vars in one declaration.")
}

fn param_array() -> DocSection {
    DocSection::new("param-array", "Param arrays", || {
        view! {
            <CodeBlock
                language="rust"
                title="VarParamArray1"
                code=r#"use infinity_rs::vars::VarParamArray1;

// build a 1-element param array
let mut param = VarParamArray1::index(2);
let v = avar.get_with(param.as_raw_mut(), FS_OBJECT_ID_USER_AIRCRAFT)?;"#
            />

            <P>
                "For arbitrary multi-arg pack/unpack, see "
                <InlineCode>"infinity_rs::utils::fs_create_param_array"</InlineCode>
                " and "
                <InlineCode>"FsVarParamArrayOwned"</InlineCode>
                "."
            </P>
        }
    })
    .numbered("4.0")
    .summary("Most callers should use the high-level helpers; the param-array API is here when you need to forward arbitrary RPN-friendly arguments.")
}
