use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("comm-bus", "Comm Bus", "Module · comm_bus", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Pub/sub message bus shared between WASM modules and the JS side of MSFS. Subscribe with a closure, publish raw bytes — and the `Subscription` handle deregisters cleanly when dropped.",
        )
        .tags(["Pub/Sub", "JS Bridge", "Subscription", "JSON"])
        .section(overview())
        .section(subscribe())
        .section(publish())
        .section(json())
        .section(errors())
}

fn overview() -> DocSection {
    DocSection::new("overview", "Overview", || {
        view! {
            <Lead>
                "The comm bus is MSFS's lightweight in-process message bus. Each event has a string name and an opaque payload ("
                <InlineCode>"&[u8]"</InlineCode>
                "); broadcast flags decide which sides see each call."
            </Lead>

            <List>
                <ListItem>
                    <Term>"Events are names."</Term>
                    " Use a stable namespace ("
                    <InlineCode>"\"mypackage/cmd\""</InlineCode>
                    ") so callers can subscribe by string without coupling to your code."
                </ListItem>
                <ListItem>
                    <Term>"Routing is per-call."</Term>
                    " "
                    <InlineCode>"BroadcastFlags::JS"</InlineCode>
                    ", "
                    <InlineCode>"BroadcastFlags::WASM"</InlineCode>
                    ", and "
                    <InlineCode>"BroadcastFlags::ALL"</InlineCode>
                    " control fan-out."
                </ListItem>
                <ListItem>
                    <Term>"Drop deregisters."</Term>
                    " The "
                    <InlineCode>"Subscription"</InlineCode>
                    " handle calls "
                    <InlineCode>"fsCommBusUnregisterOneEvent"</InlineCode>
                    " on drop."
                </ListItem>
            </List>
        }
    })
    .numbered("1.0")
    .summary("What the comm bus is and how routing works.")
}

fn subscribe() -> DocSection {
    DocSection::new("subscribe", "Subscribing", || {
        view! {
            <CodeBlock
                language="rust"
                title="subscribe"
                code=r#"use infinity_rs::prelude::*;

let sub = Subscription::subscribe("demo/cmd", |bytes: &[u8]| {
    let cmd = bytes.first().copied().unwrap_or(0);
    println!("got cmd byte = {cmd}");
})?;

// keep `sub` alive in your gauge/system state — drop = unsubscribe
struct DemoSystem { _sub_cmd: Subscription }"#
            />

            <Callout
                tone=Tone::Danger
                icon=icons::SHIELD_ALERT
                title="Don't `let _ =`"
            >
                "Writing "
                <InlineCode>"let _ = Subscription::subscribe(...)"</InlineCode>
                " drops the handle immediately and the host removes your callback. Always bind to a named field on a longer-lived state struct."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("Register a handler for an event name. Hold the returned `Subscription`.")
}

fn publish() -> DocSection {
    DocSection::new("publish", "Publishing", || {
        view! {
            <CodeBlock
                language="rust"
                title="call"
                code=r#"use infinity_rs::prelude::*;

// publish to JS subscribers only
commbus_call("demo/state", &[1u8], BroadcastFlags::JS)?;

// publish to JS + WASM
commbus_call("demo/state", &[1u8], BroadcastFlags::JS | BroadcastFlags::WASM)?;

// fan out to everyone
commbus_call("demo/state", &[1u8], BroadcastFlags::ALL)?;"#
            />
        }
    })
    .numbered("2.1")
    .summary("`call(name, payload, flags)` — fire-and-forget broadcast.")
}

fn json() -> DocSection {
    DocSection::new("json", "JSON helpers", || {
        view! {
            <CodeBlock
                language="rust"
                title="JSON request/reply"
                code=r#"use infinity_rs::comm_bus::{JsonBridge, reply_json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GetStateReq { id: u32 }

#[derive(Serialize)]
struct GetStateResp { id: u32, name: String, value: f64 }

let bridge = JsonBridge::new("demo/state/get");
let _sub = bridge.serve(|req: GetStateReq| {
    Ok::<_, &'static str>(GetStateResp {
        id: req.id,
        name: "demo".to_string(),
        value: 42.0,
    })
})?;"#
            />

            <P>
                <InlineCode>"JsonBridge"</InlineCode>
                " wraps the request/reply pattern: the JS caller sends a JSON request, your handler returns a serializable response, and the bridge ships the reply back along a paired event. "
                <InlineCode>"reply_json"</InlineCode>
                " is the lower-level primitive if you want to roll your own."
            </P>
        }
    })
    .numbered("3.0")
    .summary("Most JS-facing protocols are JSON-shaped. `JsonBridge` and `reply_json` keep the boilerplate out of your code.")
}

fn errors() -> DocSection {
    DocSection::new("errors", "Errors", || {
        view! {
            <CodeBlock
                language="rust"
                title="CommBusError"
                code=r#"pub enum CommBusError {
    NulByteInEventName,
    HostError(i32),
}"#
            />
        }
    })
    .numbered("4.0")
    .summary("`CommBusError` covers NUL bytes in event names and host-reported failures.")
}
