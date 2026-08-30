use infinity_ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new(
        "planned-route",
        "Planned Route",
        "Module · planned_route",
        "Modules",
    )
    .scope("infinity-rs")
    .tagline(
        "Read the EFB's active flight plan, subscribe to broadcast updates, or respond to in-sim route requests.",
    )
    .tags(["EFB", "Route", "Legs", "Broadcast"])
    .section(current())
    .section(leg_types())
    .section(broadcast())
    .section(requests())
}

fn current() -> DocSection {
    DocSection::new("current", "Current EFB route", || {
        view! {
            <Lead>
                <InlineCode>"current_efb_route()"</InlineCode>
                " returns a borrowed view — "
                <InlineCode>"PlannedRoute<'static>"</InlineCode>
                " for ergonomics — that is only valid until the next host poke. Re-fetch each frame instead of caching across lifecycle callbacks."
            </Lead>

            <CodeBlock
                language="rust"
                title="current_efb_route"
                code=r#"use infinity_rs::prelude::*;

if let Some(route) = current_efb_route() {
    for leg in route.enroute_legs() {
        let id   = leg.identifier().to_string_lossy();
        let kind = leg.leg_type();           // EnrouteLegType
        let alt  = leg.altitude();           // FlightAltitude
        println!("{id} ({:?}) {alt:?}", kind);
    }

    if let Some(vp) = route.visual_pattern() {
        println!("visual pattern: {vp:?}");
    }
}"#
            />
        }
    })
    .numbered("1.0")
    .summary("Pull the active route on demand. Returns `None` when nothing is loaded.")
}

fn leg_types() -> DocSection {
    DocSection::new("leg-types", "Leg types & altitudes", || {
        view! {
            <CodeBlock
                language="rust"
                title="enums"
                code=r#"pub enum EnrouteLegType { /* SID, STAR, Approach, Enroute, … */ }

pub enum FlightAltitudeType { Feet, FlightLevel }

pub struct FlightAltitude {
    pub kind:  FlightAltitudeType,
    pub value: i32,   // feet or FL units, per kind
}"#
            />
        }
    })
    .numbered("1.1")
    .summary("Typed enums over the C leg-type and altitude-kind values.")
}

fn broadcast() -> DocSection {
    DocSection::new("broadcast", "Subscribing to broadcasts", || {
        view! {
            <CodeBlock
                language="rust"
                title="broadcast"
                code=r#"use infinity_rs::prelude::*;

let _bcast = planned_route_subscribe_broadcast(|route| {
    println!("EFB pushed a new route with {} legs", route.enroute_leg_count());
})?;"#
            />
        }
    })
    .numbered("2.0")
    .summary("Get notified when the EFB publishes a new route. Hold the returned subscription.")
}

fn requests() -> DocSection {
    DocSection::new("requests", "Responding to requests", || {
        view! {
            <CodeBlock
                language="rust"
                title="serve requests"
                code=r#"use infinity_rs::prelude::*;

let _req_sub = planned_route_subscribe_requests(|req: RouteRequest| {
    if let Some(route) = current_efb_route() {
        let _ = req.respond(&route);
    }
    // dropping req without responding lets the host time the requester out
})?;"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Which side are you?">
                "Use "
                <InlineCode>"subscribe_broadcast"</InlineCode>
                " when you want to consume routes; use "
                <InlineCode>"subscribe_requests"</InlineCode>
                " when your module is the source of truth and others are asking you for it."
            </Callout>
        }
    })
    .numbered("3.0")
    .summary("Other modules can ask the host who's holding the canonical route. Respond via `RouteRequest::respond`.")
}
