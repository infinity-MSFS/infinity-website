use infinity_ui::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, BetaAccess, BetaApplication, Hardware};
use crate::auth::use_auth;

const EXPERIENCE: [&str; 3] = ["Beginner", "Intermediate", "Power user"];
const USAGE: [&str; 3] = ["Casual", "Regular", "Heavy"];
const STORAGE: [&str; 4] = ["NVMe", "SSD", "HDD", "Mixed"];
const MONITORS: [&str; 4] = ["Single", "Dual", "Triple+", "Mixed/Other"];
const TESTING_STYLE: [&str; 3] = ["I follow steps", "I explore and break things", "Both"];

fn local_timezone() -> String {
    let options = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &js_sys::Object::new())
        .resolved_options();
    js_sys::Reflect::get(&options, &"timeZone".into())
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Status {
    Idle,
    Submitting,
    Error(String),
    Submitted { reference: Option<String> },
}

#[component]
pub fn BetaSignup() -> impl IntoView {
    let auth = use_auth();
    let info_open = RwSignal::new(false);
    let status = RwSignal::new(Status::Idle);
    let gate = RwSignal::new(None::<BetaAccess>);
    let gate_error = RwSignal::new(None::<String>);

    let display_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let discord = RwSignal::new(String::new());
    let timezone = RwSignal::new(local_timezone());
    let experience = RwSignal::new("Intermediate".to_string());
    let usage = RwSignal::new("Regular".to_string());
    let cpu = RwSignal::new(String::new());
    let gpu = RwSignal::new(String::new());
    let ram = RwSignal::new(String::new());
    let storage = RwSignal::new("SSD".to_string());
    let monitors = RwSignal::new("Single".to_string());
    let can_call = RwSignal::new(true);
    let can_share = RwSignal::new(true);
    let test_nightlies = RwSignal::new(true);
    let availability = RwSignal::new(String::new());
    let testing_style = RwSignal::new("Both".to_string());
    let why_you = RwSignal::new(String::new());
    let past_reports = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let ack_pre_release = RwSignal::new(false);
    let ack_contact = RwSignal::new(false);

    Effect::new(move |_| {
        let Some(user) = auth.user.get() else {
            return;
        };
        if display_name.get_untracked().is_empty()
            && let Some(name) = user.name.clone()
        {
            display_name.set(name);
        }
        if email.get_untracked().is_empty()
            && let Some(address) = user.email.clone()
        {
            email.set(address);
        }
    });

    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            gate.set(None);
            return;
        }
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                return;
            };
            match api::beta_status(&token).await {
                Ok(access) => gate.set(Some(access)),
                Err(err) => gate_error.set(Some(err)),
            }
        });
    });

    let locked_out = Signal::derive(move || {
        matches!(
            gate.get(),
            Some(BetaAccess::Requested | BetaAccess::Accepted)
        )
    });
    let submitting = Signal::derive(move || status.get() == Status::Submitting);
    let submitted = Signal::derive(move || matches!(status.get(), Status::Submitted { .. }));
    let show_form = Signal::derive(move || !locked_out.get() && !submitted.get());

    let validate = move || -> Option<String> {
        if !auth.is_authenticated.get_untracked() {
            return Some("Please sign in before applying.".into());
        }
        if display_name.get_untracked().trim().is_empty() {
            return Some("Please enter your name (or a display name).".into());
        }
        let address = email.get_untracked();
        if address.trim().is_empty() || !address.contains('@') {
            return Some("Please enter a valid email address.".into());
        }
        if discord.get_untracked().trim().is_empty() {
            return Some("Please enter your Discord username.".into());
        }
        if timezone.get_untracked().trim().is_empty() {
            return Some("Please enter your timezone.".into());
        }
        if !can_call.get_untracked() {
            return Some("Closed beta requires Discord voice availability.".into());
        }
        if !can_share.get_untracked() {
            return Some("Closed beta requires screen sharing availability.".into());
        }
        if !test_nightlies.get_untracked() {
            return Some("Closed beta requires willingness to test pre-release builds.".into());
        }
        if why_you.get_untracked().trim().chars().count() < 40 {
            return Some("Tell us why we should select you (at least ~40 characters).".into());
        }
        if !ack_pre_release.get_untracked() {
            return Some("Please acknowledge this is a pre-release build.".into());
        }
        if !ack_contact.get_untracked() {
            return Some("Please confirm we can contact you on Discord or email.".into());
        }
        None
    };

    let optional = |signal: RwSignal<String>| {
        let value = signal.get_untracked().trim().to_string();
        (!value.is_empty()).then_some(value)
    };

    let submit = move |()| {
        if let Some(message) = validate() {
            status.set(Status::Error(message));
            return;
        }
        status.set(Status::Submitting);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                status.set(Status::Error(
                    "Your session expired. Sign in again and resubmit.".into(),
                ));
                return;
            };
            let application = BetaApplication {
                auth0_user_id: auth.user.get_untracked().and_then(|u| u.sub),
                display_name: display_name.get_untracked().trim().to_string(),
                email: email.get_untracked().trim().to_string(),
                discord: discord.get_untracked().trim().to_string(),
                timezone: timezone.get_untracked().trim().to_string(),
                experience: experience.get_untracked(),
                expected_usage: usage.get_untracked(),
                hardware: Hardware {
                    cpu: optional(cpu),
                    gpu: optional(gpu),
                    ram_gb: optional(ram),
                    storage: storage.get_untracked(),
                    monitors: monitors.get_untracked(),
                },
                can_discord_call: can_call.get_untracked(),
                can_screen_share: can_share.get_untracked(),
                willing_test_nightlies: test_nightlies.get_untracked(),
                availability: optional(availability),
                testing_style: testing_style.get_untracked(),
                why_you: why_you.get_untracked().trim().to_string(),
                past_bug_reports: optional(past_reports),
                notes: optional(notes),
                pre_release_ack: ack_pre_release.get_untracked(),
                contact_ok: ack_contact.get_untracked(),
            };
            match api::submit_beta_application(&token, &application).await {
                Ok(reference) => {
                    gate.set(Some(BetaAccess::Requested));
                    status.set(Status::Submitted { reference });
                }
                Err(err) => status.set(Status::Error(err)),
            }
        });
    };

    view! {
        <div class="relative z-10 min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-8 py-12">
                <div class="mb-10">
                    <div class="flex flex-wrap items-center gap-2">
                        <Badge tone=Tone::Info icon=icons::SHIELD size=Size::Lg>
                            "Closed beta · selective, debug-focused"
                        </Badge>
                        <Button
                            variant=Variant::Glass
                            size=Size::Sm
                            icon=icons::CIRCLE_ALERT
                            on_click=Callback::new(move |()| info_open.set(true))
                        >
                            "How selection works"
                        </Button>
                    </div>

                    <h1 class="mt-4 mb-3 text-4xl font-bold">"Apply for the Closed Beta"</h1>
                    <p class="max-w-3xl leading-relaxed text-white/80">
                        "We're picking a small group of testers who can give high-quality feedback and help us squash bugs fast."
                    </p>
                </div>

                {move || {
                    gate_error
                        .get()
                        .map(|message| {
                            view! {
                                <Alert
                                    tone=Tone::Danger
                                    title="Couldn't check your application status"
                                    description=format!(
                                        "{message} You can still apply; status tracking may be unavailable until this resolves.",
                                    )
                                    class="mb-8"
                                />
                            }
                        })
                }}

                {move || match gate.get() {
                    Some(BetaAccess::Requested) => {
                        view! {
                            <Alert
                                tone=Tone::Info
                                title="Application already submitted"
                                description="Your account is marked as requested. If selected, we'll reach out via email or Discord."
                                class="mb-8"
                            />
                        }
                            .into_any()
                    }
                    Some(BetaAccess::Accepted) => {
                        view! {
                            <Alert
                                tone=Tone::Accent
                                title="You're accepted"
                                description="Your account is marked as accepted. Check Discord for next steps."
                                class="mb-8"
                            />
                        }
                            .into_any()
                    }
                    _ => ().into_any(),
                }}

                <div class="grid items-start gap-10 lg:grid-cols-2">
                    <div class="space-y-6">
                        <Card title="Selection criteria" icon=icons::USERS tone=Tone::Info>
                            <div class="space-y-4">
                                <Criterion
                                    icon=icons::USERS
                                    title="Communicates clearly"
                                    body="You can describe what happened, what you expected, and how to reproduce it."
                                />
                                <Criterion
                                    icon=icons::MESSAGE_CIRCLE
                                    title="Can debug with us"
                                    body="Willing to jump into Discord voice and screen share when we need to chase a bug."
                                />
                                <Criterion
                                    icon=icons::FILE_TEXT
                                    title="Actually tests"
                                    body="You'll run real flights and workflows and report issues consistently, not a one-and-done."
                                />
                                <Criterion
                                    icon=icons::SHIELD
                                    title="Pre-release mindset"
                                    body="The beta may crash or break. You're fine with that and won't panic when it does."
                                />
                            </div>
                        </Card>

                        <Card title="What we'll ask of you" icon=icons::SHIELD tone=Tone::Warning>
                            <List>
                                <ListItem>"Test real scenarios, not one launch."</ListItem>
                                <ListItem>
                                    "Report issues with clear steps and what you expected."
                                </ListItem>
                                <ListItem>
                                    "Join Discord voice and screen share when needed."
                                </ListItem>
                                <ListItem>"Try hotfixes quickly and confirm if they work."</ListItem>
                            </List>

                            <Callout
                                tone=Tone::Neutral
                                icon=icons::FILE_TEXT
                                title="Logs and system info"
                                class="mt-4"
                            >
                                "We collect debug info only "
                                <strong class="text-white/90">"after"</strong>
                                " bugs are submitted, or when troubleshooting a specific issue."
                            </Callout>

                            <Show when=move || !auth.is_authenticated.get()>
                                <Callout
                                    tone=Tone::Info
                                    icon=icons::LOG_IN
                                    title="Sign in required"
                                    class="mt-4"
                                >
                                    "We require sign-in to prevent abuse and to track your application status."
                                    <div class="mt-3">
                                        <Button
                                            icon=icons::LOG_IN
                                            variant=Variant::Solid
                                            tone=Tone::Info
                                            on_click=Callback::new(move |()| auth.login())
                                        >
                                            "Sign in"
                                        </Button>
                                    </div>
                                </Callout>
                            </Show>
                        </Card>

                        <Card title="Discord required" icon=icons::MESSAGE_CIRCLE tone=Tone::Accent>
                            "Closed beta is collaborative. If you're not comfortable with Discord calls and screen share, this isn't the right test group."
                        </Card>
                    </div>

                    <Panel class="p-6">
                        <div class="mb-4 flex items-center gap-3">
                            <Icon icon=icons::FILE_TEXT class="h-5 w-5 text-accent-400" />
                            <h2 class="text-xl font-bold">"Application"</h2>
                        </div>

                        {move || match status.get() {
                            Status::Error(message) => {
                                view! {
                                    <Alert
                                        tone=Tone::Danger
                                        title="Not submitted"
                                        description=message
                                        class="mb-4"
                                        on_dismiss=Callback::new(move |()| status.set(Status::Idle))
                                    />
                                }
                                    .into_any()
                            }
                            Status::Submitted { reference } => {
                                view! {
                                    <Alert
                                        tone=Tone::Accent
                                        title="Application submitted"
                                        description=match reference {
                                            Some(id) => {
                                                format!(
                                                    "If selected, we'll contact you via email or Discord. Reference: {id}",
                                                )
                                            }
                                            None => {
                                                "If selected, we'll contact you via email or Discord with next steps."
                                                    .to_string()
                                            }
                                        }
                                    />
                                }
                                    .into_any()
                            }
                            _ => ().into_any(),
                        }}

                        <Show when=move || locked_out.get() && !submitted.get()>
                            <Well class="p-6 text-sm text-white/75">
                                "Your beta status is "
                                <strong class="text-white/90">
                                    {move || match gate.get() {
                                        Some(BetaAccess::Accepted) => "accepted",
                                        Some(BetaAccess::Requested) => "requested",
                                        _ => "none",
                                    }}
                                </strong>
                                ". You can't submit another application from this account."
                            </Well>
                        </Show>

                        <Show when=move || show_form.get()>
                            <div class="grid gap-4 md:grid-cols-2">
                                <Field label="Name / display name" required=true>
                                    <TextInput
                                        value=display_name
                                        on_input=Callback::new(move |v| display_name.set(v))
                                        placeholder="Your name"
                                        disabled=submitting
                                    />
                                </Field>
                                <Field label="Email" required=true>
                                    <TextInput
                                        value=email
                                        on_input=Callback::new(move |v| email.set(v))
                                        placeholder="you@example.com"
                                        input_type="email"
                                        disabled=submitting
                                    />
                                </Field>
                                <Field
                                    label="Discord username"
                                    required=true
                                    hint="We'll coordinate here: voice and screen share."
                                >
                                    <TextInput
                                        value=discord
                                        on_input=Callback::new(move |v| discord.set(v))
                                        placeholder="username"
                                        disabled=submitting
                                    />
                                </Field>
                                <Field label="Timezone" required=true>
                                    <TextInput
                                        value=timezone
                                        on_input=Callback::new(move |v| timezone.set(v))
                                        placeholder="America/Denver"
                                        disabled=submitting
                                    />
                                </Field>
                                <Field
                                    label="Experience"
                                    required=true
                                    hint="Helps us balance the group."
                                >
                                    <Select
                                        value=experience
                                        on_change=Callback::new(move |v| experience.set(v))
                                        groups=Signal::derive(|| SelectGroup::flat(EXPERIENCE))
                                    />
                                </Field>
                                <Field label="Usage" required=true>
                                    <Select
                                        value=usage
                                        on_change=Callback::new(move |v| usage.set(v))
                                        groups=Signal::derive(|| SelectGroup::flat(USAGE))
                                    />
                                </Field>
                            </div>

                            <Panel class="mt-6 p-4">
                                <div class="mb-2 flex items-center gap-2 font-semibold">
                                    <Icon icon=icons::CPU class="h-5 w-5 text-accent-400" />
                                    "Basic setup (self-reported)"
                                </div>
                                <p class="mb-3 text-sm text-white/60">
                                    "This helps us pick a diverse group. Nothing is collected from your machine automatically."
                                </p>
                                <div class="grid gap-4 md:grid-cols-3">
                                    <Field label="CPU (optional)">
                                        <TextInput
                                            value=cpu
                                            on_input=Callback::new(move |v| cpu.set(v))
                                            placeholder="e.g. Ryzen 7 7800X3D"
                                            disabled=submitting
                                        />
                                    </Field>
                                    <Field label="GPU (optional)">
                                        <TextInput
                                            value=gpu
                                            on_input=Callback::new(move |v| gpu.set(v))
                                            placeholder="e.g. RTX 4070 Ti"
                                            disabled=submitting
                                        />
                                    </Field>
                                    <Field label="RAM in GB (optional)">
                                        <TextInput
                                            value=ram
                                            on_input=Callback::new(move |v| ram.set(v))
                                            placeholder="e.g. 32"
                                            disabled=submitting
                                        />
                                    </Field>
                                    <Field label="Storage">
                                        <Select
                                            value=storage
                                            on_change=Callback::new(move |v| storage.set(v))
                                            groups=Signal::derive(|| SelectGroup::flat(STORAGE))
                                        />
                                    </Field>
                                    <Field label="Monitors">
                                        <Select
                                            value=monitors
                                            on_change=Callback::new(move |v| monitors.set(v))
                                            groups=Signal::derive(|| SelectGroup::flat(MONITORS))
                                        />
                                    </Field>
                                </div>
                            </Panel>

                            <Panel class="mt-6 p-4">
                                <div class="mb-3 flex items-center gap-2 font-semibold">
                                    <Icon
                                        icon=icons::MESSAGE_CIRCLE
                                        class="h-5 w-5 text-accent-400"
                                    />
                                    "Commitments (required)"
                                </div>
                                <div class="grid gap-3 sm:grid-cols-2">
                                    <Checkbox
                                        checked=can_call
                                        on_change=Callback::new(move |v| can_call.set(v))
                                        label="I can do Discord voice calls"
                                    />
                                    <Checkbox
                                        checked=can_share
                                        on_change=Callback::new(move |v| can_share.set(v))
                                        label="I can screen share for debugging"
                                    />
                                    <Checkbox
                                        checked=test_nightlies
                                        on_change=Callback::new(move |v| test_nightlies.set(v))
                                        label="I can test pre-release builds quickly"
                                    />
                                </div>
                            </Panel>

                            <div class="mt-6 grid gap-4 md:grid-cols-2">
                                <Field
                                    label="Availability (optional)"
                                    hint="When can you hop on calls or test?"
                                >
                                    <TextInput
                                        value=availability
                                        on_input=Callback::new(move |v| availability.set(v))
                                        placeholder="Weeknights, weekends…"
                                        disabled=submitting
                                    />
                                </Field>
                                <Field label="Testing style" required=true>
                                    <Select
                                        value=testing_style
                                        on_change=Callback::new(move |v| testing_style.set(v))
                                        groups=Signal::derive(|| SelectGroup::flat(TESTING_STYLE))
                                    />
                                </Field>
                            </div>

                            <Field
                                label="Why should we select you?"
                                required=true
                                hint="Be specific: your workflows, how you report bugs, willingness to hop on calls."
                                class="mt-6"
                            >
                                <TextArea
                                    value=why_you
                                    on_input=Callback::new(move |v| why_you.set(v))
                                    rows=5
                                    placeholder="I fly 3–4x a week, can reproduce issues with clear steps, and I'm available for Discord calls to help debug…"
                                    disabled=submitting
                                />
                            </Field>

                            <Field
                                label="Have you submitted bug reports before? (optional)"
                                hint="Links or a quick description help: GitHub issues, Discord reports."
                                class="mt-4"
                            >
                                <TextArea
                                    value=past_reports
                                    on_input=Callback::new(move |v| past_reports.set(v))
                                    rows=3
                                    placeholder="I've filed detailed reports for MSFS add-ons with repro steps and logs…"
                                    disabled=submitting
                                />
                            </Field>

                            <Field label="Anything else? (optional)" class="mt-4">
                                <TextArea
                                    value=notes
                                    on_input=Callback::new(move |v| notes.set(v))
                                    rows=3
                                    placeholder="Edge setups, add-on conflicts, VR, peripherals…"
                                    disabled=submitting
                                />
                            </Field>

                            <Panel class="mt-6 p-4">
                                <div class="mb-3 flex items-center gap-2 font-semibold">
                                    <Icon icon=icons::SHIELD class="h-5 w-5 text-amber-300" />
                                    "Acknowledgements (required)"
                                </div>
                                <div class="space-y-3">
                                    <Checkbox
                                        checked=ack_pre_release
                                        on_change=Callback::new(move |v| ack_pre_release.set(v))
                                        label="I understand this is a pre-release build and may crash or break."
                                    />
                                    <Checkbox
                                        checked=ack_contact
                                        on_change=Callback::new(move |v| ack_contact.set(v))
                                        label="You can contact me via email or Discord if I'm selected."
                                    />
                                </div>
                            </Panel>

                            <div class="mt-6 flex flex-col gap-3 sm:flex-row">
                                {move || {
                                    if auth.is_authenticated.get() {
                                        view! {
                                            <Button
                                                variant=Variant::Solid
                                                icon=icons::CIRCLE_CHECK
                                                loading=submitting
                                                disabled=locked_out
                                                on_click=Callback::new(submit)
                                                class="flex-1"
                                            >
                                                "Submit application"
                                            </Button>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <Button
                                                variant=Variant::Solid
                                                icon=icons::LOG_IN
                                                on_click=Callback::new(move |()| auth.login())
                                                class="flex-1"
                                            >
                                                "Sign in to apply"
                                            </Button>
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>

                            <p class="mt-4 text-xs leading-relaxed text-white/50">
                                "Applying doesn't guarantee acceptance. We'll pick a small group to keep the feedback loop tight."
                            </p>
                        </Show>
                    </Panel>
                </div>
            </div>

            <Modal
                open=info_open
                on_close=Callback::new(move |()| info_open.set(false))
                title="How selection works"
                icon=icons::CIRCLE_ALERT
                tone=Tone::Info
                width="max-w-2xl"
                footer=ViewFn::from(move || {
                    view! {
                        <Button
                            variant=Variant::Solid
                            tone=Tone::Info
                            on_click=Callback::new(move |()| info_open.set(false))
                        >
                            "Got it"
                        </Button>
                    }
                })
            >
                <div class="space-y-4 text-sm leading-7 text-white/80">
                    <p>
                        "We're selecting a small group to keep the feedback loop tight. The goal is fast iteration, quick hotfix validation, and high quality bug reports."
                    </p>
                    <Well class="p-4">
                        <div class="mb-2 font-semibold text-white">"We prioritise:"</div>
                        <ul class="list-inside list-disc space-y-1 text-white/70">
                            <li>"Clear communication and reproducible bug reports"</li>
                            <li>"Availability for Discord call and screen share"</li>
                            <li>"Diverse setups: VR, multi-monitor, different hardware"</li>
                            <li>"Consistent testers, not one weekend then gone"</li>
                        </ul>
                    </Well>
                    <Callout tone=Tone::Warning title="Important">
                        "We do not collect logs or system info at signup. Those are requested only when bugs are submitted or when troubleshooting."
                    </Callout>
                </div>
            </Modal>
        </div>
    }
}

#[component]
fn Criterion(icon: IconData, title: &'static str, body: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-start gap-3">
            <div class="mt-0.5 rounded-lg border border-white/10 bg-white/5 p-2">
                <Icon icon=icon class="h-5 w-5 text-accent-400" />
            </div>
            <div>
                <div class="font-semibold text-white">{title}</div>
                <div class="text-sm leading-relaxed text-white/70">{body}</div>
            </div>
        </div>
    }
}
