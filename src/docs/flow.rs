use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("flow", "Flow Events", "Module · flow", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Coarse-grained sim lifecycle events: flight start/end, pause boundaries, RTC and replay transitions, teleports, plane crashes. Subscribe once, branch on the variant.",
        )
        .tags(["Lifecycle", "Pause", "Replay", "RTC"])
        .section(events())
        .section(subscribe())
        .section(lifecycle())
}

fn events() -> DocSection {
    DocSection::new("events", "Events", || {
        view! {
            <Lead>
                <InlineCode>"FlowEvent"</InlineCode>
                " is a Rust enum that mirrors the C "
                <InlineCode>"FsFlowEvent"</InlineCode>
                " values. Convenience helpers group events by semantic category so you don't need to enumerate every variant by hand."
            </Lead>

            <CodeBlock
                language="rust"
                title="FlowEvent"
                code=r#"pub enum FlowEvent {
    None,
    FltLoad,
    FltLoaded,
    TeleportStart,
    TeleportDone,
    BackOnTrackStart,
    BackOnTrackDone,
    SkipStart,
    SkipDone,
    BackToMainMenu,
    RTCStart,
    RTCEnd,
    ReplayStart,
    ReplayEnd,
    FlightStart,
    FlightEnd,
    PlaneCrash,
}"#
            />

            <List>
                <ListItem>
                    <Term>
                        <InlineCode>"is_pause_boundary()"</InlineCode>
                    </Term>
                    " — TeleportStart, BackOnTrackStart, SkipStart, RTCStart, ReplayStart, BackToMainMenu."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"is_resume_boundary()"</InlineCode>
                    </Term>
                    " — TeleportDone, BackOnTrackDone, SkipDone, RTCEnd, ReplayEnd, FltLoaded."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"is_shutdown_boundary()"</InlineCode>
                    </Term>
                    " — FlightEnd, PlaneCrash, BackToMainMenu."
                </ListItem>
            </List>
        }
    })
    .numbered("1.0")
    .summary("The full enum of flow events the host emits.")
}

fn subscribe() -> DocSection {
    DocSection::new("subscribe", "Subscribing", || {
        view! {
            <CodeBlock
                language="rust"
                title="subscribe"
                code=r#"use infinity_rs::prelude::*;

let sub: FlowSubscription = flow_subscribe(|msg| {
    if msg.event.is_pause_boundary() {
        // freeze your sim-clock state, pause network polling, etc.
    }

    if msg.event.is_resume_boundary() {
        // resync — sim time may have jumped
    }

    if msg.event.is_shutdown_boundary() {
        // persist state before unload
    }

    match msg.event {
        FlowEvent::PlaneCrash    => log::warn!("plane crash"),
        FlowEvent::FlightStart   => self.on_flight_start(),
        _ => {}
    }
})?;"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Payload">
                <InlineCode>"msg.payload"</InlineCode>
                " is a borrowed "
                <InlineCode>"&[u8]"</InlineCode>
                " for events that carry per-event data (currently rare; most events are pure signals)."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("One handler. Branch on `msg.event` (and inspect `msg.payload` if relevant).")
}

fn lifecycle() -> DocSection {
    DocSection::new("lifecycle", "Lifecycle", || {
        view! {
            <P>
                "Drop the "
                <InlineCode>"FlowSubscription"</InlineCode>
                " to deregister your handler. If you need a hard reset (for example, before reloading scripts), call "
                <InlineCode>"flow::unregister_all()"</InlineCode>
                "."
            </P>
        }
    })
    .numbered("3.0")
    .summary("`FlowSubscription` is RAII over `fsFlowUnregister`. `unregister_all()` is available as an escape hatch.")
}
