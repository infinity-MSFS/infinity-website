use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("events", "Key Events", "Module · events", "Modules")
        .scope("infinity-rs")
        .tagline(
            "MSFS key events (the same things mapped to keybinds, joystick buttons, and the SimConnect K: namespace). Subscribe to incoming events with typed parameter access, and trigger them with one of several typed helpers.",
        )
        .tags(["KeyEvent", "EventId", "Trigger", "Params"])
        .section(ids())
        .section(subscribe())
        .section(trigger())
}

fn ids() -> DocSection {
    DocSection::new("ids", "Event IDs", || {
        view! {
            <CodeBlock
                language="rust"
                title="EventId"
                code=r#"pub struct EventId(pub FsEventId);

impl From<u32> for EventId { /* ... */ }
impl PartialEq<u32>     for EventId { /* ... */ }
impl PartialEq<EventId> for u32      { /* ... */ }"#
            />

            <P>
                "Use the named constants from "
                <InlineCode>"infinity_rs::sys"</InlineCode>
                " (e.g. "
                <InlineCode>"KEY_TOGGLE_MASTER_BATTERY"</InlineCode>
                ") when available; fall back to bare "
                <InlineCode>"u32"</InlineCode>
                " only when bridging legacy IDs."
            </P>
        }
    })
    .numbered("1.0")
    .summary("`EventId` wraps the host's `FsEventId` and converts cleanly from `u32`.")
}

fn subscribe() -> DocSection {
    DocSection::new("subscribe", "Subscribing", || {
        view! {
            <CodeBlock
                language="rust"
                title="subscribe"
                code=r#"use infinity_rs::prelude::*;
use infinity_rs::events;

const KEY_TOGGLE_MASTER_BATTERY: u32 = /* from sys */ 0;

let sub = events::subscribe(|event| {
    if event.id == KEY_TOGGLE_MASTER_BATTERY {
        match event.params.first() {
            Some(FsParamArg::Index(idx))  => println!("battery #{idx} toggled"),
            Some(FsParamArg::Double(v))   => println!("double = {v}"),
            Some(FsParamArg::Crc(c))      => println!("crc = {c:?}"),
            Some(FsParamArg::Str(p))      => println!("str ptr = {p:?}"),
            None                          => println!("no params"),
        }
    }
});

// Convenience accessors:
let battery_index = event.params.first_index().unwrap_or(1);"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Param shape">
                <InlineCode>"EventParamsRef"</InlineCode>
                " wraps the raw "
                <InlineCode>"FsVarParamArray"</InlineCode>
                ". Use "
                <InlineCode>"first()"</InlineCode>
                " / "
                <InlineCode>"first_index()"</InlineCode>
                " / "
                <InlineCode>"iter()"</InlineCode>
                " rather than poking at the raw pointer."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("`events::subscribe` takes an `FnMut(KeyEvent)` and gives back a `Subscription`. Drop = deregister.")
}

fn trigger() -> DocSection {
    DocSection::new("trigger", "Triggering events", || {
        view! {
            <List>
                <ListItem>
                    <Term>
                        <InlineCode>"trigger0"</InlineCode>
                    </Term>
                    " — no parameters."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"trigger1(id, FsParamArg)"</InlineCode>
                    </Term>
                    " — one typed argument (most common)."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"trigger_fmt(id, fmt, &args)"</InlineCode>
                    </Term>
                    " — format-string driven, like SimConnect's RPN-friendly args."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"trigger"</InlineCode>
                        " /"
                        <InlineCode>"trigger_raw"</InlineCode>
                    </Term>
                    " — pass an already-built "
                    <InlineCode>"FsVarParamArrayOwned"</InlineCode>
                    " / "
                    <InlineCode>"FsVarParamArray"</InlineCode>
                    " respectively."
                </ListItem>
            </List>

            <CodeBlock
                language="rust"
                title="trigger family"
                code=r#"use infinity_rs::events;
use infinity_rs::prelude::*;

events::trigger0(KEY_PARKING_BRAKE)?;
events::trigger1(KEY_TOGGLE_MASTER_BATTERY, FsParamArg::Index(1))?;

// "if" = index then double — fmt drives variant selection per char
events::trigger_fmt(KEY_FOO, "id", &[FsParamArg::Index(1), FsParamArg::Double(2.5)])?;"#
            />
        }
    })
    .numbered("3.0")
    .summary("Pick the helper that matches the parameter shape you have on hand.")
}
