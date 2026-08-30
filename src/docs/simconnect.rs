use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("simconnect", "SimConnect", "Module · simconnect", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Safe wrapper over the SimConnect SDK. Works on both WASM (resolved by the host runtime) and native Windows (linked against the SimConnect import library bundled with the SDK). The same module gives you data definitions, dispatch, AI objects, weather, input events, facility data, and client-data channels.",
        )
        .tags(["SimConnect", "DataDef", "Recv", "Native", "WASM"])
        .section(feature())
        .section(connection())
        .section(data_definitions())
        .section(dispatch())
        .section(writing())
        .section(events_and_input())
        .section(client_data())
        .section(facilities_and_misc())
}

fn feature() -> DocSection {
    DocSection::new("feature", "Enabling the module", || {
        view! {
            <CodeBlock
                language="toml"
                title="Cargo.toml — WASM gauge with SimConnect"
                code=r#"infinity-rs = { version = "0.2.8", features = ["simconnect"] }"#
            />

            <CodeBlock
                language="toml"
                title="Cargo.toml — native client"
                code=r#"infinity-rs = { version = "0.2.8", default-features = false, features = ["simconnect"] }"#
            />
        }
    })
    .numbered("1.0")
    .summary("`simconnect` is opt-in via cargo feature. Native consumers also disable defaults.")
}

fn connection() -> DocSection {
    DocSection::new("connection", "Connection", || {
        view! {
            <Lead>
                <InlineCode>"SimConnect::open"</InlineCode>
                " calls "
                <InlineCode>"SimConnect_Open"</InlineCode>
                " with the local config index. Drop the handle to disconnect, or call "
                <InlineCode>"close()"</InlineCode>
                " explicitly when you want the close to be part of a normal control-flow path."
            </Lead>

            <CodeBlock
                language="rust"
                title="open"
                code=r#"use infinity_rs::simconnect::SimConnect;

let mut sc = SimConnect::open("my-client")?;        // local config
let mut sc = SimConnect::open_config(2, "my-client")?; // index from SimConnect.cfg"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Threading">
                <InlineCode>"SimConnect"</InlineCode>
                " is "
                <InlineCode>"Send"</InlineCode>
                " but not "
                <InlineCode>"Sync"</InlineCode>
                ". Move the handle between threads if you must, but don't share it across threads without external synchronisation."
            </Callout>
        }
    })
    .numbered("2.0")
    .summary("`SimConnect::open(name)` returns an owned client. The handle implements `Send` but not `Sync`.")
}

fn data_definitions() -> DocSection {
    DocSection::new("data-definitions", "Data definitions", || {
        view! {
            <P>
                "The struct's field order, sizes, and "
                <InlineCode>"#[repr(C, packed)]"</InlineCode>
                " attribute must match the registered definition exactly. Otherwise SimConnect surfaces a "
                <InlineCode>"SIMCONNECT_EXCEPTION_DATA_ERROR"</InlineCode>
                " asynchronously."
            </P>

            <CodeBlock
                language="rust"
                title="data def + request"
                code=r#"use infinity_rs::simconnect::*;

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
struct Aircraft {
    altitude_ft: f64,
    airspeed_kt: f64,
    heading_deg: f64,
}

const DEF: DataDefinitionId = DataDefinitionId::new(1);
const REQ: DataRequestId    = DataRequestId::new(1);

sc.add_to_data_definition(DEF, "PLANE ALTITUDE",   "feet",
    DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED)?;
sc.add_to_data_definition(DEF, "AIRSPEED INDICATED", "knots",
    DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED)?;
sc.add_to_data_definition(DEF, "PLANE HEADING DEGREES TRUE", "degrees",
    DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED)?;

sc.request_data_on_sim_object(
    REQ, DEF, ObjectId::USER,
    Period::SIMCONNECT_PERIOD_SIM_FRAME,
    DataRequestFlag::CHANGED, 0, 0, 0,
)?;"#
            />
        }
    })
    .numbered("3.0")
    .summary("Build a layout description that mirrors a packed Rust struct, then request data on a SimObject.")
}

fn dispatch() -> DocSection {
    DocSection::new("dispatch", "Dispatching messages", || {
        view! {
            <CodeBlock
                language="rust"
                title="dispatch"
                code=r#"use infinity_rs::simconnect::*;

loop {
    sc.dispatch(|msg| match msg {
        Recv::SimObjectData(d)
            if SimObjectDataView(d).dwRequestID() == REQ.raw() =>
        {
            let v: Aircraft = unsafe { SimObjectDataView(d).data_as() };
            println!("alt={} ias={} hdg={}", v.altitude_ft, v.airspeed_kt, v.heading_deg);
        }
        Recv::Open(open)        => println!("opened: {:?}", open),
        Recv::Quit(_)           => return,
        Recv::Exception(e) => {
            let view = ExceptionView(e);
            eprintln!("exception {:?} (sendID={})", view.exception(), view.dwSendID());
        }
        _ => {}
    })?;
    std::thread::sleep(std::time::Duration::from_millis(50));
}"#
            />

            <List>
                <ListItem>
                    <Term>
                        <InlineCode>"dispatch(closure)"</InlineCode>
                    </Term>
                    " drains until the queue is empty or an error occurs."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"next_message()"</InlineCode>
                    </Term>
                    " pulls one message; returns "
                    <InlineCode>"Ok(None)"</InlineCode>
                    " when nothing's pending."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"set_dispatch_proc(...)"</InlineCode>
                    </Term>
                    " hooks an internal C-style callback (advanced; usually not needed)."
                </ListItem>
            </List>
        }
    })
    .numbered("4.0")
    .summary("`dispatch` drains the queue through a typed `Recv` enum. `next_message` pulls one at a time.")
}

fn writing() -> DocSection {
    DocSection::new("writing", "Writing data", || {
        view! {
            <CodeBlock
                language="rust"
                title="write"
                code=r#"#[repr(C, packed)]
struct Throttle { value: f64 }

const DEF_THR: DataDefinitionId = DataDefinitionId::new(2);
sc.add_to_data_definition(DEF_THR, "GENERAL ENG THROTTLE LEVER POSITION:1", "percent",
    DataType::SIMCONNECT_DATATYPE_FLOAT64, 0.0, UNUSED)?;

let payload = Throttle { value: 65.0 };
unsafe { sc.set_data_on_sim_object(DEF_THR, ObjectId::USER, &[payload])? };"#
            />

            <Callout
                tone=Tone::Danger
                icon=icons::SHIELD_ALERT
                title="Layout matters"
            >
                <InlineCode>"set_data_on_sim_object"</InlineCode>
                " is unsafe — the type's memory layout must match the registered data definition or you'll get silent corruption / async exceptions."
            </Callout>
        }
    })
    .numbered("5.0")
    .summary("`set_data_on_sim_object` pushes a buffer to the sim using a registered definition.")
}

fn events_and_input() -> DocSection {
    DocSection::new(
        "events-and-input",
        "Events, inputs, and notification groups",
        || {
            view! {
                <CodeBlock
                    language="rust"
                    title="events"
                    code=r#"use infinity_rs::simconnect::*;

const GRP_INPUT: NotificationGroupId = NotificationGroupId::new(1);
const EVT_TOGGLE: ClientEventId      = ClientEventId::new(100);

sc.map_client_event_to_sim_event(EVT_TOGGLE, "TOGGLE_MASTER_BATTERY")?;
sc.add_client_event_to_notification_group(GRP_INPUT, EVT_TOGGLE, false)?;
sc.set_notification_group_priority(GRP_INPUT, NotificationGroupPriority::Highest)?;

// fire it later
sc.transmit_client_event(ObjectId::USER, EVT_TOGGLE, 0,
    NotificationGroupId::DEFAULT, EventFlag::GROUPID_IS_PRIORITY)?;"#
                />
            }
        },
    )
    .numbered("6.0")
    .summary("Map client event IDs to sim event names, group them, and arm via input definitions.")
}

fn client_data() -> DocSection {
    DocSection::new("client-data", "Client data", || {
        view! {
            <CodeBlock
                language="rust"
                title="client data"
                code=r#"use infinity_rs::simconnect::*;

const CDATA: ClientDataId = ClientDataId::new(1);
const CDATA_DEF: ClientDataDefinitionId = ClientDataDefinitionId::new(1);

sc.create_client_data(CDATA, 64, CreateClientDataFlag::READ_ONLY)?;
sc.add_to_client_data_definition(CDATA_DEF, 0, 64, 0.0, UNUSED)?;

#[repr(C, packed)]
struct Telemetry { altitude_ft: f64, airspeed_kt: f64 /* ... padded to 64 ... */ }
let payload = Telemetry { altitude_ft: 25_000.0, airspeed_kt: 450.0 };
unsafe { sc.set_client_data(CDATA, CDATA_DEF, ClientDataSetFlag::default(), 0, &[payload])? };"#
            />
        }
    })
    .numbered("7.0")
    .summary("Cross-process shared blocks of bytes. Useful for streaming structured state out of a gauge to a companion native app.")
}

fn facilities_and_misc() -> DocSection {
    DocSection::new(
        "facilities-and-misc",
        "Facilities, AI, weather, text",
        || {
            view! {
                <List>
                    <ListItem>
                        <Term>"Facility data"</Term>
                        " — request airports, VORs, NDBs, waypoints by ICAO; subscribe to facility list updates."
                    </ListItem>
                    <ListItem>
                        <Term>"AI objects"</Term>
                        " — create/release AI aircraft, ground objects, helicopters, simulated objects via "
                        <InlineCode>"ai_create_*"</InlineCode>
                        " / "
                        <InlineCode>"ai_remove_object"</InlineCode>
                        "."
                    </ListItem>
                    <ListItem>
                        <Term>"Weather"</Term>
                        " — request and inject metar / weather observations at stations or coordinates."
                    </ListItem>
                    <ListItem>
                        <Term>"Text"</Term>
                        " — "
                        <InlineCode>"text(text_type, lifetime, &str)"</InlineCode>
                        " displays in-sim toasts via the SDK's "
                        <InlineCode>"SIMCONNECT_TEXT_TYPE_*"</InlineCode>
                        " machinery."
                    </ListItem>
                    <ListItem>
                        <Term>"Diagnostic"</Term>
                        " — "
                        <InlineCode>"last_sent_packet_id()"</InlineCode>
                        " and "
                        <InlineCode>"request_response_times"</InlineCode>
                        " help correlate exceptions back to the offending call."
                    </ListItem>
                </List>
            }
        },
    )
    .numbered("8.0")
    .summary("The SimConnect SDK is large — `infinity-rs` exposes the full surface, even if you only need a slice. A few highlights:")
}
