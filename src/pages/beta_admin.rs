//! Review queue for closed beta applications. Developer-gated.
//!
//! Everything here is a view over the worker's admin endpoints — there is no
//! local state that outlives a decision. The gate on this page is UX; the real
//! one is `requireDeveloper` on the token, checked worker-side on every call.
//!
//! The queue deliberately shows each wrong answer next to the right one rather
//! than just a percentage: a tester who missed the cargo door question but knows
//! the hydraulics is a different proposition from one who guessed throughout,
//! and that distinction is invisible in a score.

use infinity_ui::prelude::*;
use leptos::task::spawn_local;

use crate::api::{
    self, AdminApplication, AdminApplicationDetail, AdminSignature, BetaDecision, BetaStatus,
};
use crate::auth::use_auth;
use crate::components::{EYEBROW, HoverRule, SectionLabel};
use wasm_bindgen::JsCast;

/// Render an ISO timestamp as something readable. The worker stores UTC, so
/// say so rather than implying a local time we haven't converted to.
fn stamp(iso: &str) -> String {
    let chars: Vec<char> = iso.chars().collect();
    if chars.len() < 16 {
        return iso.to_string();
    }
    let date: String = chars[..10].iter().collect();
    let time: String = chars[11..16].iter().collect();
    format!("{date} {time} UTC")
}

/// Hand the browser a file it already has in memory.
///
/// The PDF arrives over an authenticated fetch, so it can't be a plain link —
/// the bytes have to be wrapped in a blob and clicked through a throwaway
/// anchor. The object URL is revoked straight after; the click has already
/// taken its own reference by then.
fn download_bytes(bytes: &[u8], filename: &str) -> Result<(), String> {
    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());

    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/pdf");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .map_err(|_| "Couldn't prepare the download.".to_string())?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Couldn't prepare the download.".to_string())?;

    let result = (|| {
        let anchor = document()
            .create_element("a")
            .map_err(|_| "Couldn't prepare the download.".to_string())?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "Couldn't prepare the download.".to_string())?;
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();
        Ok(())
    })();

    let _ = web_sys::Url::revoke_object_url(&url);
    result
}

fn status_label(status: BetaStatus) -> &'static str {
    match status {
        BetaStatus::None => "None",
        BetaStatus::Requested => "Awaiting review",
        BetaStatus::Accepted => "In paperwork",
        BetaStatus::Approved => "Approved",
        BetaStatus::Rejected => "Rejected",
        BetaStatus::Withdrawn => "Withdrawn",
    }
}

fn status_color(status: BetaStatus) -> &'static str {
    match status {
        BetaStatus::Approved => "text-accent-300",
        BetaStatus::Accepted | BetaStatus::Requested => "text-amber-300",
        BetaStatus::Rejected => "text-red-300",
        BetaStatus::None | BetaStatus::Withdrawn => "text-white/40",
    }
}

/// Mono status tag used in place of a badge.
const TAG: &str = "font-mono text-[11px] tracking-[0.2em] uppercase";

/// Accepted, everything signed, waiting on a human to look at the paperwork.
fn ready_to_approve(application: &AdminApplication) -> bool {
    application.status == BetaStatus::Accepted && application.documents_complete
}

#[component]
pub fn BetaAdmin() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Show
            when=move || !auth.loading.get()
            fallback=|| {
                view! {
                    <div class=cn!("flex min-h-screen items-center justify-center", EYEBROW)>
                        "Loading…"
                    </div>
                }
            }
        >
            <Show
                when=move || auth.entitlements.get().is_developer()
                fallback=|| view! { <crate::pages::NotFound /> }
            >
                <Queue />
            </Show>
        </Show>
    }
}

#[component]
fn Queue() -> impl IntoView {
    let auth = use_auth();

    let applications = RwSignal::new(None::<Vec<AdminApplication>>);
    let error = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let filter = RwSignal::new("requested".to_string());
    let expanded = RwSignal::new(None::<String>);

    let refresh = move || {
        loading.set(true);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                loading.set(false);
                return;
            };
            match api::beta_applications(&token).await {
                Ok(page) => {
                    error.set(None);
                    applications.set(Some(page.applications));
                }
                Err(err) => error.set(Some(err)),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            return;
        }
        refresh();
    });

    let all = Signal::derive(move || applications.get().unwrap_or_default());
    let count = move |status: BetaStatus| {
        all.get()
            .iter()
            .filter(|application| application.status == status)
            .count()
    };
    let visible = Signal::derive(move || {
        let wanted = filter.get();
        all.get()
            .into_iter()
            .filter(|application| match wanted.as_str() {
                "requested" => application.status == BetaStatus::Requested,
                "paperwork" => application.status == BetaStatus::Accepted,
                "approved" => application.status == BetaStatus::Approved,
                "rejected" => application.status == BetaStatus::Rejected,
                _ => true,
            })
            .collect::<Vec<_>>()
    });

    let ready_count =
        Signal::derive(move || all.get().iter().filter(|a| ready_to_approve(a)).count());

    let tabs = Signal::derive(move || {
        vec![
            TabItem::new("requested", "Awaiting review")
                .with_badge(count(BetaStatus::Requested).to_string()),
            TabItem::new("paperwork", "In paperwork")
                .with_badge(count(BetaStatus::Accepted).to_string()),
            TabItem::new("approved", "Approved")
                .with_badge(count(BetaStatus::Approved).to_string()),
            TabItem::new("rejected", "Rejected")
                .with_badge(count(BetaStatus::Rejected).to_string()),
            TabItem::new("all", "All").with_badge(all.get().len().to_string()),
        ]
    });

    view! {
        <AnimatedPage class="min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <header class="mb-16">
                    <p class=cn!("mb-10", EYEBROW)>"Infinity / Beta / Admin"</p>
                    <div class="flex flex-col gap-6 md:flex-row md:items-end md:justify-between">
                        <div>
                            <h1 class="text-4xl leading-[1.05] font-extralight tracking-tight md:text-6xl">
                                "Beta "
                                <span class="font-semibold text-accent-300">"applications"</span>
                            </h1>
                            <p class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 font-light text-white/65">
                                {format!("Aerodynamics DC-10 · programme \"{}\"", api::BETA_PROGRAM)}
                            </p>
                            <p class="mt-4 font-mono text-[11px] tracking-[0.2em] text-amber-300 uppercase">
                                "Staff"
                            </p>
                        </div>
                        <button
                            type="button"
                            disabled=move || loading.get()
                            on:click=move |_| refresh()
                            class="inline-flex items-center gap-2 self-start text-sm text-white/55 transition-colors hover:text-white disabled:opacity-40 md:self-end"
                        >
                            <Icon icon=icons::REFRESH_CW class="h-4 w-4" />
                            "Refresh"
                        </button>
                    </div>
                </header>

                {move || {
                    error
                        .get()
                        .map(|message| {
                            view! {
                                <p class="mb-10 border-l border-red-400/60 pl-4 text-sm text-red-200">
                                    "Couldn't load the queue. "
                                    <span class="text-red-200/70">{message}</span>
                                </p>
                            }
                        })
                }}

                <section class="mb-20">
                    <SectionLabel num="01" label="Overview" />
                    <div class="grid border-t border-white/10 sm:grid-cols-2 lg:grid-cols-4">
                        {move || {
                            view! {
                                <QueueStat
                                    label="Awaiting review"
                                    value=count(BetaStatus::Requested)
                                />
                                <QueueStat
                                    label="In paperwork"
                                    value=count(BetaStatus::Accepted)
                                />
                                <QueueStat
                                    label="Ready to approve"
                                    value=ready_count.get()
                                    detail="Signed, waiting on you"
                                    highlight=true
                                />
                                <QueueStat
                                    label="Approved"
                                    value=count(BetaStatus::Approved)
                                />
                            }
                        }}
                    </div>
                </section>

                <section class="mb-20">
                    <SectionLabel num="02" label="Applications" />
                    <div class="mb-8 overflow-x-auto">
                        <Tabs
                            value=Signal::derive(move || filter.get())
                            on_change=Callback::new(move |value| filter.set(value))
                            items=tabs
                        />
                    </div>

                    {move || {
                        if applications.with(Option::is_none) {
                            return view! {
                                <p class=cn!("border-t border-white/10 py-8", EYEBROW)>
                                    "Loading…"
                                </p>
                            }
                                .into_any();
                        }
                        let rows = visible.get();
                        if rows.is_empty() {
                            return view! {
                                <p class="border-t border-white/10 py-8 font-light text-white/55">
                                    "No applications match this filter yet."
                                </p>
                            }
                                .into_any();
                        }
                        view! {
                            <div class=cn!("flex items-center gap-4 pb-3", EYEBROW)>
                                <span class="flex-1">"Applicant"</span>
                                <span>"Score"</span>
                                <span class="w-4"></span>
                            </div>
                            <div class="border-b border-white/10">
                                {rows
                                    .into_iter()
                                    .map(|application| {
                                        view! {
                                            <ApplicationCard
                                                application=application
                                                expanded=expanded
                                                on_changed=Callback::new(move |()| {
                                                    expanded.set(None);
                                                    refresh();
                                                })
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                            .into_any()
                    }}
                </section>
            </div>
        </AnimatedPage>
    }
}

#[component]
fn ApplicationCard(
    application: AdminApplication,
    expanded: RwSignal<Option<String>>,
    #[prop(into)] on_changed: Callback<()>,
) -> impl IntoView {
    let auth = use_auth();

    let id = application.application_id.clone();
    let toggle_id = id.clone();
    let open = Signal::derive(move || expanded.get().as_deref() == Some(id.as_str()));

    let note = RwSignal::new(application.decision_note.clone().unwrap_or_default());
    let busy = RwSignal::new(false);
    let outcome = RwSignal::new(None::<String>);
    let failure = RwSignal::new(None::<String>);
    let detail = RwSignal::new(None::<AdminApplicationDetail>);

    let confirm_remove = RwSignal::new(false);
    let exporting = RwSignal::new(false);

    let sub = StoredValue::new(application.sub.clone());
    let status = application.status;
    let score = application.score;
    let can_approve = application.documents_complete;

    let decide = move |next: BetaStatus| {
        busy.set(true);
        failure.set(None);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                busy.set(false);
                failure.set(Some("Your session expired. Sign in again.".into()));
                return;
            };
            let trimmed = note.get_untracked().trim().to_string();
            let decision = BetaDecision {
                sub: sub.get_value(),
                status: next,
                note: (!trimmed.is_empty()).then_some(trimmed),
            };
            match api::decide_beta_application(&token, &decision).await {
                Ok(result) => {
                    busy.set(false);
                    if result.role_synced {
                        on_changed.run(());
                        return;
                    }
                    // The record is written before Auth0 is touched, so a
                    // role failure means "decided, but not yet entitled".
                    // Say that rather than letting it read as a failed
                    // decision — the fix is to repeat it, not rethink it.
                    let role = result.role.unwrap_or_else(|| "beta".to_string());
                    let why = result
                        .role_error
                        .unwrap_or_else(|| "the Auth0 call didn't complete".to_string());
                    outcome.set(Some(format!(
                        "Decision saved, but the {role} role didn't sync: {why}. Repeat the same decision to retry it.",
                    )));
                }
                Err(err) => {
                    busy.set(false);
                    failure.set(Some(err));
                }
            }
        });
    };

    // `None` exports everything they have signed; `Some(id)` exports one.
    let export = move |document_id: Option<&'static str>| {
        exporting.set(true);
        failure.set(None);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                exporting.set(false);
                failure.set(Some("Your session expired. Sign in again.".into()));
                return;
            };
            match api::beta_export_pdf(&token, &sub.get_value(), document_id).await {
                Ok((bytes, filename)) => {
                    exporting.set(false);
                    if let Err(message) = download_bytes(&bytes, &filename) {
                        failure.set(Some(message));
                    }
                }
                Err(err) => {
                    exporting.set(false);
                    failure.set(Some(err));
                }
            }
        });
    };

    let remove = move |()| {
        confirm_remove.set(false);
        busy.set(true);
        failure.set(None);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                busy.set(false);
                failure.set(Some("Your session expired. Sign in again.".into()));
                return;
            };
            match api::remove_beta_application(&token, &sub.get_value()).await {
                Ok(()) => {
                    busy.set(false);
                    on_changed.run(());
                }
                Err(err) => {
                    busy.set(false);
                    failure.set(Some(err));
                }
            }
        });
    };

    let load_detail = move |()| {
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                return;
            };
            match api::beta_application(&token, &sub.get_value()).await {
                Ok(loaded) => detail.set(Some(loaded)),
                Err(err) => failure.set(Some(err)),
            }
        });
    };

    // The expanded body is a `Show` child, so it is rebuilt on every toggle and
    // anything it reads has to survive being read more than once. Header fields
    // render once and can stay plain.
    let why_you = StoredValue::new(application.why_you.clone());
    let answers = StoredValue::new(application.answers.clone());
    let open_answers = StoredValue::new(application.open_answers.clone());
    let signatures = StoredValue::new(application.signatures.clone());
    let email = StoredValue::new(application.contact.email.clone());
    let discord = StoredValue::new(application.contact.discord.clone());
    let timezone = StoredValue::new(application.contact.timezone.clone());
    let reference = StoredValue::new(application.application_id.clone());
    let decided_at = StoredValue::new(application.decided_at.clone());

    let documents_complete = application.documents_complete;
    let header_name = application.contact.display_name.clone();
    let header_name_for_dialog = application.contact.display_name.clone();
    let header_discord = application.contact.discord.clone();
    let header_timezone = application.contact.timezone.clone();
    let submitted = application.submitted_at.clone();

    view! {
        <div class=move || {
            cn!("border-t border-white/10 transition-colors", open.get().then_some("bg-white/[0.02]"))
        }>
            <button
                type="button"
                class="group relative flex w-full items-center gap-4 py-4 text-left transition-colors hover:bg-white/[0.03]"
                on:click=move |_| {
                    expanded
                        .update(|current| {
                            *current = if current.as_deref() == Some(toggle_id.as_str()) {
                                None
                            } else {
                                Some(toggle_id.clone())
                            };
                        });
                }
            >
                <HoverRule />
                <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                        <span class="truncate text-base font-medium text-white">{header_name}</span>
                        <span class=cn!(TAG, status_color(status))>{status_label(status)}</span>
                        {(documents_complete && status == BetaStatus::Accepted)
                            .then(|| {
                                view! {
                                    <span class=cn!(TAG, "text-amber-300")>"Ready to approve"</span>
                                }
                            })}
                        {(documents_complete && status == BetaStatus::Approved)
                            .then(|| {
                                view! { <span class=cn!(TAG, "text-accent-300")>"Signed"</span> }
                            })}
                    </div>
                    <div class="mt-1 truncate font-mono text-[11px] text-white/45">
                        {format!(
                            "{header_discord} · {header_timezone} · applied {}",
                            stamp(&submitted),
                        )}
                    </div>
                </div>

                <div class="shrink-0 text-right">
                    <div class=cn!(
                        "text-2xl font-extralight tabular-nums",
                        if score.percent >= 70 { "text-accent-300" } else { "text-white/70" },
                    )>{format!("{}%", score.percent)}</div>
                    <div class="font-mono text-[11px] text-white/35">
                        {format!("{}/{}", score.correct, score.total)}
                    </div>
                </div>

                {move || {
                    let class = cn!(
                        "h-4 w-4 shrink-0 text-white/40 transition-transform",
                        open.get().then_some("rotate-180"),
                    );
                    view! { <Icon icon=icons::CHEVRON_DOWN class=class /> }
                }}
            </button>

            <Show when=move || open.get()>
                <div class="space-y-12 border-t border-white/10 py-8 md:pl-6">
                    <dl class="grid gap-x-10 sm:grid-cols-2">
                        <DetailRow label="Email" value=email.get_value() />
                        <DetailRow label="Discord" value=discord.get_value() />
                        <DetailRow label="Timezone" value=timezone.get_value() />
                        <DetailRow label="Reference" value=reference.get_value() />
                        <DetailRow label="Auth0 subject" value=sub.get_value() />
                        {decided_at
                            .get_value()
                            .map(|at| {
                                view! { <DetailRow label="Decided" value=stamp(&at) /> }
                            })}
                    </dl>

                    <section>
                        <SectionHeading text="Why we should choose them" />
                        <p class="max-w-3xl border-l border-white/15 pl-5 text-sm leading-relaxed font-light whitespace-pre-wrap text-white/75">
                            {why_you.get_value()}
                        </p>
                    </section>

                    <section>
                        <SectionHeading text=format!(
                            "Systems check — {}/{} correct",
                            score.correct,
                            score.total,
                        ) />
                        <div class="border-b border-white/10">
                            {answers
                                .get_value()
                                .into_iter()
                                .map(|answer| view! { <AnswerRow answer=answer /> })
                                .collect_view()}
                        </div>
                    </section>

                    <Show when=move || !open_answers.with_value(Vec::is_empty)>
                        <section>
                            <SectionHeading text="In their words" />
                            <div class="border-b border-white/10">
                                {open_answers
                                    .get_value()
                                    .into_iter()
                                    .map(|answer| {
                                        view! {
                                            <div class="border-t border-white/10 py-4">
                                                <div class="text-xs leading-5 text-white/45">
                                                    {answer
                                                        .prompt
                                                        .clone()
                                                        .unwrap_or_else(|| answer.question_id.clone())}
                                                </div>
                                                <div class="mt-1.5 text-sm leading-relaxed font-light whitespace-pre-wrap text-white/80">
                                                    {if answer.answer.trim().is_empty() {
                                                        "— left blank —".to_string()
                                                    } else {
                                                        answer.answer.clone()
                                                    }}
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </section>
                    </Show>

                    <Show when=move || !signatures.with_value(Vec::is_empty)>
                        <section>
                            <SectionHeading text="Signatures" />
                            <div class="border-b border-white/10">
                                {move || {
                                    // Once the detail call has run, prefer its
                                    // signatures — they carry the presigned ink.
                                    let rows = detail
                                        .get()
                                        .map(|d| d.signatures)
                                        .filter(|s| !s.is_empty())
                                        .unwrap_or_else(|| signatures.get_value());
                                    rows.into_iter()
                                        .map(|signature| {
                                            view! { <SignatureRow signature=signature /> }
                                        })
                                        .collect_view()
                                }}
                            </div>
                            <div class="mt-5 flex flex-wrap gap-2">
                                <Show when=move || detail.with(Option::is_none)>
                                    <Button
                                        variant=Variant::Ghost
                                        size=Size::Sm
                                        icon=icons::EYE
                                        on_click=Callback::new(load_detail)
                                    >
                                        "Load signature images"
                                    </Button>
                                </Show>
                                <Button
                                    variant=Variant::Ghost
                                    size=Size::Sm
                                    icon=icons::DOWNLOAD
                                    loading=exporting
                                    on_click=Callback::new(move |()| export(None))
                                >
                                    "Export all as PDF"
                                </Button>
                                <Button
                                    variant=Variant::Ghost
                                    size=Size::Sm
                                    icon=icons::FILE_TEXT
                                    disabled=exporting
                                    on_click=Callback::new(move |()| export(Some("nda")))
                                >
                                    "NDA only"
                                </Button>
                                <Button
                                    variant=Variant::Ghost
                                    size=Size::Sm
                                    icon=icons::FILE_TEXT
                                    disabled=exporting
                                    on_click=Callback::new(move |()| export(Some("beta-agreement")))
                                >
                                    "Contract only"
                                </Button>
                            </div>
                            <p class="mt-3 max-w-2xl text-xs leading-5 font-light text-white/40">
                                "The PDF carries the document text as signed, the signer's legal name and address, their signature, and the audit trail — timestamp, IP, and the hash of the exact wording they agreed to."
                            </p>
                        </section>
                    </Show>

                    {move || {
                        detail
                            .get()
                            .and_then(|d| {
                                let meta = d.client_meta;
                                let parts: Vec<String> = [
                                    meta.ip.map(|v| format!("IP {v}")),
                                    meta.country.map(|v| format!("from {v}")),
                                    meta.user_agent,
                                ]
                                    .into_iter()
                                    .flatten()
                                    .collect();
                                (!parts.is_empty())
                                    .then(|| {
                                        view! {
                                            <div class="font-mono text-[11px] leading-5 break-all text-white/35">
                                                {format!("Applied from {}", parts.join(" · "))}
                                            </div>
                                        }
                                    })
                            })
                    }}

                    {move || {
                        failure
                            .get()
                            .map(|message| {
                                view! {
                                    <p class="border-l border-red-400/60 pl-4 text-sm text-red-200">
                                        "Decision failed. "
                                        <span class="text-red-200/70">{message}</span>
                                    </p>
                                }
                            })
                    }}
                    {move || {
                        outcome
                            .get()
                            .map(|message| {
                                view! {
                                    <p class="border-l border-amber-400/60 pl-4 text-sm font-light text-amber-200/90">
                                        "Partly applied. "
                                        <span class="text-amber-200/70">{message}</span>
                                    </p>
                                }
                            })
                    }}

                    <section>
                        <SectionHeading text="Decision" />
                        <Field
                            label="Note (optional)"
                            hint="Shown to the applicant on the beta page. Worth writing for a rejection."
                        >
                            <TextArea
                                value=note
                                on_input=Callback::new(move |v| note.set(v))
                                rows=2
                                placeholder="We're keeping this round small — reapply next time."
                                disabled=busy
                            />
                        </Field>
                        <div class="mt-4 flex flex-wrap gap-3">
                            // First gate: unlocks the documents, grants nothing.
                            <Show when=move || status != BetaStatus::Approved>
                                <Button
                                    variant=if can_approve { Variant::Glass } else { Variant::Solid }
                                    icon=icons::FILE_TEXT
                                    loading=busy
                                    disabled=Signal::derive(move || status == BetaStatus::Accepted)
                                    on_click=Callback::new(move |()| decide(BetaStatus::Accepted))
                                >
                                    "Accept — send to paperwork"
                                </Button>
                            </Show>

                            // Second gate. The worker refuses this outright while
                            // anything is unsigned, so the disabled state here is
                            // a courtesy rather than the enforcement.
                            <Show when=move || status == BetaStatus::Accepted>
                                <Button
                                    variant=Variant::Solid
                                    icon=icons::CIRCLE_CHECK
                                    loading=busy
                                    disabled=Signal::derive(move || !can_approve)
                                    on_click=Callback::new(move |()| decide(BetaStatus::Approved))
                                >
                                    {if can_approve {
                                        "Approve — release the build"
                                    } else {
                                        "Approve (paperwork outstanding)"
                                    }}
                                </Button>
                            </Show>

                            <Button
                                variant=Variant::Soft
                                tone=Tone::Danger
                                icon=icons::CIRCLE_X
                                disabled=Signal::derive(move || {
                                    busy.get() || status == BetaStatus::Rejected
                                })
                                on_click=Callback::new(move |()| decide(BetaStatus::Rejected))
                            >
                                "Reject"
                            </Button>
                            <Show when=move || status != BetaStatus::Requested>
                                <Button
                                    variant=Variant::Ghost
                                    icon=icons::CLOCK
                                    disabled=busy
                                    on_click=Callback::new(move |()| decide(BetaStatus::Requested))
                                >
                                    "Move back to review"
                                </Button>
                            </Show>
                        </div>
                        <p class="mt-4 max-w-2xl text-xs leading-5 font-light text-white/40">
                            "Accepting only unlocks the NDA and test agreement on their beta page — it grants nothing. The Auth0 beta role is assigned on approval, once both documents are signed, and removed by any other decision."
                        </p>

                        <div class="mt-8 flex flex-col gap-3 border-t border-white/10 pt-5 sm:flex-row sm:items-center">
                            <Button
                                variant=Variant::Ghost
                                tone=Tone::Danger
                                size=Size::Sm
                                icon=icons::TRASH_2
                                disabled=busy
                                on_click=Callback::new(move |()| confirm_remove.set(true))
                                class="shrink-0 self-start"
                            >
                                "Remove application"
                            </Button>
                            <span class="max-w-2xl text-xs leading-5 font-light text-white/40">
                                "Deletes the record and signatures and revokes the role, so the account can apply again from scratch. For putting a test account back to square one — reject a real applicant instead, so the evidence survives."
                            </span>
                        </div>

                        <ConfirmDialog
                            open=confirm_remove
                            on_close=Callback::new(move |()| confirm_remove.set(false))
                            on_confirm=Callback::new(remove)
                            title="Remove this application?"
                            message=format!(
                                "This permanently deletes {}'s application, every signature on file, and their beta role. It cannot be undone, and they will be able to apply again from scratch.",
                                header_name_for_dialog,
                            )
                            confirm_label="Delete permanently"
                            tone=Tone::Danger
                        />
                    </section>
                </div>
            </Show>
        </div>
    }
}

/// One figure in the overview strip.
#[component]
fn QueueStat(
    label: &'static str,
    value: usize,
    #[prop(optional)] detail: Option<&'static str>,
    #[prop(optional)] highlight: bool,
) -> impl IntoView {
    view! {
        <div class="border-b border-white/10 py-6 sm:pr-8">
            <p class=EYEBROW>{label}</p>
            <p class=cn!(
                "mt-3 text-5xl font-extralight tabular-nums",
                if highlight && value > 0 { "text-amber-300" } else { "text-white" },
            )>{format!("{value:02}")}</p>
            {detail
                .map(|detail| {
                    view! { <p class="mt-2 font-mono text-[11px] text-white/40">{detail}</p> }
                })}
        </div>
    }
}

/// Mono label with a hairline, for the subsections inside an expanded row.
#[component]
fn SectionHeading(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <div class=cn!("mb-5 flex items-center gap-4", EYEBROW)>
            <span>{text}</span>
            <span class="h-px flex-1 bg-white/10"></span>
        </div>
    }
}

#[component]
fn DetailRow(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    // Long values (an Auth0 subject, a full email) are truncated in the cell,
    // so keep the whole thing on the tooltip.
    let title = value.clone();
    view! {
        <div class="grid grid-cols-[8rem_1fr] items-baseline gap-4 border-t border-white/10 py-3">
            <dt class="font-mono text-[11px] tracking-[0.2em] text-white/45 uppercase">{label}</dt>
            <dd class="truncate text-sm font-light text-white/85" title=title>
                {value}
            </dd>
        </div>
    }
}

#[component]
fn AnswerRow(answer: api::AdminAnswer) -> impl IntoView {
    let correct = answer.correct;
    let chosen = answer
        .chosen
        .clone()
        .unwrap_or_else(|| "Left blank".to_string());

    view! {
        <div class="flex items-start gap-4 border-t border-white/10 py-4">
            <Icon
                icon=if correct { icons::CIRCLE_CHECK } else { icons::CIRCLE_X }
                class=cn!(
                    "mt-0.5 h-4 w-4 shrink-0",
                    if correct { "text-accent-300" } else { "text-red-300" },
                )
            />
            <div class="min-w-0 flex-1">
                {answer
                    .topic
                    .clone()
                    .map(|topic| {
                        view! {
                            <div class="mb-1 font-mono text-[11px] tracking-[0.2em] text-white/35 uppercase">
                                {topic}
                            </div>
                        }
                    })}
                <div class="text-sm font-medium text-white/85">
                    {answer.prompt.clone().unwrap_or_else(|| answer.question_id.clone())}
                </div>
                <div class="mt-1 text-sm leading-6 font-light text-white/60">
                    {format!("Answered: {chosen}")}
                </div>
                <Show when=move || !correct>
                    {answer
                        .correct_choice
                        .clone()
                        .map(|right| {
                            view! {
                                <div class="mt-0.5 text-sm leading-6 font-light text-accent-300/90">
                                    {format!("Correct: {right}")}
                                </div>
                            }
                        })}
                    {answer
                        .explanation
                        .clone()
                        .map(|why| {
                            view! {
                                <div class="mt-1 text-xs leading-5 font-light text-white/40">
                                    {why}
                                </div>
                            }
                        })}
                </Show>
            </div>
        </div>
    }
}

#[component]
fn SignatureRow(signature: AdminSignature) -> impl IntoView {
    view! {
        <div class="border-t border-white/10 py-5">
            <div class="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                <span class="text-sm font-medium text-white/85">
                    {signature.document_id.clone()}
                </span>
                <span class="font-mono text-[11px] text-white/50">
                    {format!("v{}", signature.document_version)}
                </span>
                <span class="font-mono text-[11px] tracking-[0.2em] text-white/50 uppercase">
                    {signature.method.clone()}
                </span>
                <span class="font-mono text-[11px] text-white/40">
                    {stamp(&signature.signed_at)}
                </span>
            </div>

            {signature
                .typed_name
                .clone()
                .map(|name| {
                    view! {
                        <div class="mt-1 text-sm font-light text-white/60">
                            {format!("Signed as {name}")}
                        </div>
                    }
                })}

            {signature
                .signer
                .clone()
                .map(|signer| {
                    let address = [
                        Some(signer.address.line1),
                        signer.address.line2,
                        Some(signer.address.city),
                        signer.address.region,
                        Some(signer.address.postal_code),
                        Some(signer.address.country),
                    ]
                        .into_iter()
                        .flatten()
                        .filter(|part| !part.trim().is_empty())
                        .collect::<Vec<_>>()
                        .join(", ");
                    view! {
                        <div class="mt-3 border-l border-white/15 pl-4 text-xs leading-5 font-light text-white/55">
                            <div class="font-medium text-white/75">{signer.legal_name}</div>
                            <div>{signer.email}</div>
                            {signer.phone.map(|phone| view! { <div>{phone}</div> })}
                            <div>{address}</div>
                        </div>
                    }
                })}

            {signature
                .image_url
                .clone()
                .map(|url| {
                    view! {
                        <img
                            src=url
                            alt="Signature"
                            class="mt-4 max-h-40 rounded-sm border border-white/10 bg-white"
                        />
                    }
                })}

            <div class="mt-3 font-mono text-[11px] leading-5 break-all text-white/30">
                {format!("receipt {}", signature.receipt_id)}
                {signature
                    .document_hash
                    .clone()
                    .map(|hash| format!(" · sha256 {hash}"))}
                {signature.ip.clone().map(|ip| format!(" · {ip}"))}
            </div>
        </div>
    }
}
