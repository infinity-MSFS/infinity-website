//! Closed beta application for the Aerodynamics DC-10.
//!
//! Two stages, both living on this route because the tester is told to "check
//! the beta page again" after a decision:
//!
//!   1. Apply — a short DC-10 systems questionnaire, then contact details and
//!      a case for why we should pick them. The questionnaire is graded on the
//!      worker: the answer key never reaches the browser, and the score rides
//!      along with the application as advisory context rather than a gate.
//!      Questions come one per screen, each timed while it is the one shown.
//!      The times go to reviewers and are never shown to the applicant.
//!   2. Sign — once accepted, the beta agreement (NDA and testing terms in one
//!      document) appears here as a card that links out to the e-signing page.
//!
//! Everything shown is driven by `api::beta_status`, so the stage the applicant
//! sees is the stage the worker says they are in. The stepper is presentation.

use std::collections::BTreeMap;

use infinity_ui::prelude::*;
use leptos::task::spawn_local;

use crate::api::{
    self, BetaApplication, BetaContact, BetaDocumentState, BetaState, BetaStatus, OpenAnswer,
    PostalAddress, QuizAnswer, QuizPaper, QuizScore,
};
use crate::auth::{User, use_auth};
use crate::components::{EYEBROW, HoverRule, SectionLabel};
use crate::nav;

const MIN_WHY_YOU: usize = 40;

/// The beta agreement's eligibility clause (Article 5, Section V) requires
/// testers to be at least this old.
const MIN_AGE: i32 = 18;

const STEPS: [&str; 4] = ["Systems check", "In your words", "Your details", "Review"];

/// Spec row: mono label on the left, value on the right.
const SPEC_ROW: &str = "grid gap-2 border-b border-white/10 py-4 md:grid-cols-[14rem_1fr] md:gap-8";
const SPEC_LABEL: &str = "font-mono text-xs tracking-wider text-white/45 uppercase";

/// Left-rule notices, in place of tinted alert boxes.
const NOTICE_ERROR: &str = "border-red-400/60 text-red-200";
const NOTICE_WARNING: &str = "border-amber-400/60 text-amber-100";

/// Mono uppercase status label above a stage heading.
const STATUS: &str = "mb-4 font-mono text-[11px] tracking-[0.25em] uppercase";

/// Secondary action: a text link with a leading arrow.
const BACK_LINK: &str = "group inline-flex items-center gap-2 self-start text-sm text-white/55 transition-colors hover:text-white disabled:pointer-events-none disabled:opacity-40";

fn local_timezone() -> String {
    let options = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &js_sys::Object::new())
        .resolved_options();
    js_sys::Reflect::get(&options, &"timeZone".into())
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// Whole years from an ISO `YYYY-MM-DD` date to today — negative for a date
/// still to come — or `None` if it isn't a full date.
pub(super) fn age_on(iso: &str) -> Option<i32> {
    let mut parts = iso.trim().splitn(3, '-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let today = js_sys::Date::new_0();
    let mut age = today.get_full_year() as i32 - year;
    if (today.get_month() + 1, today.get_date()) < (month, day) {
        age -= 1;
    }
    Some(age)
}

/// First and last name from the Auth0 profile. Social logins usually carry
/// both. Failing that, only a plain two-word name is split: anything longer
/// can't be split reliably, and a database user's name is often their email.
fn profile_names(user: &User) -> (Option<String>, Option<String>) {
    if user.given_name.is_some() || user.family_name.is_some() {
        return (user.given_name.clone(), user.family_name.clone());
    }
    let name = user.name.as_deref().unwrap_or_default();
    match name.split_whitespace().collect::<Vec<_>>().as_slice() {
        [first, last] if !name.contains('@') => {
            (Some((*first).to_string()), Some((*last).to_string()))
        }
        _ => (None, None),
    }
}

/// Everything the details step collects. Signals are `Copy`, so the whole set
/// travels to each step that edits or shows it.
#[derive(Clone, Copy)]
struct Details {
    first_name: RwSignal<String>,
    last_name: RwSignal<String>,
    date_of_birth: RwSignal<String>,
    faa_id: RwSignal<String>,
    email: RwSignal<String>,
    discord: RwSignal<String>,
    timezone: RwSignal<String>,
    line1: RwSignal<String>,
    line2: RwSignal<String>,
    city: RwSignal<String>,
    region: RwSignal<String>,
    postal_code: RwSignal<String>,
    country: RwSignal<String>,
    why_you: RwSignal<String>,
}

impl Details {
    fn new() -> Self {
        let empty = || RwSignal::new(String::new());
        Self {
            first_name: empty(),
            last_name: empty(),
            date_of_birth: empty(),
            faa_id: empty(),
            email: empty(),
            discord: empty(),
            timezone: RwSignal::new(local_timezone()),
            line1: empty(),
            line2: empty(),
            city: empty(),
            region: empty(),
            postal_code: empty(),
            country: empty(),
            why_you: empty(),
        }
    }

    fn full_name(&self) -> String {
        format!(
            "{} {}",
            self.first_name.get().trim(),
            self.last_name.get().trim()
        )
        .trim()
        .to_string()
    }

    /// The address as it would be written on an envelope, one line per row.
    fn address_block(&self) -> String {
        let value = |field: RwSignal<String>| field.get().trim().to_string();
        let mut locality = value(self.city);
        let region = value(self.region);
        if !region.is_empty() {
            locality = if locality.is_empty() {
                region
            } else {
                format!("{locality}, {region}")
            };
        }
        let postal_code = value(self.postal_code);
        if !postal_code.is_empty() {
            locality = format!("{locality} {postal_code}").trim().to_string();
        }
        [value(self.line1), value(self.line2), locality, value(self.country)]
            .into_iter()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The first thing stopping the details going in, in the order the fields
    /// appear. Mirrors the worker's rules, which it re-checks on submit, so
    /// this is only here to fail early rather than after a round trip.
    fn problem(&self) -> Option<String> {
        let blank = |field: RwSignal<String>| field.get().trim().is_empty();
        if blank(self.first_name) {
            return Some("Enter your first name.".into());
        }
        if blank(self.last_name) {
            return Some("Enter your last name.".into());
        }
        let born = self.date_of_birth.get();
        if born.trim().is_empty() {
            return Some("Enter your date of birth.".into());
        }
        match age_on(&born) {
            None => return Some("Enter your date of birth as a full date.".into()),
            Some(age) if !(0..=120).contains(&age) => {
                return Some("Check your date of birth.".into());
            }
            Some(age) if age < MIN_AGE => {
                return Some(format!(
                    "You need to be {MIN_AGE} or older to join the beta — the beta agreement requires it."
                ));
            }
            Some(_) => {}
        }
        let address = self.email.get();
        if address.trim().is_empty() || !address.contains('@') {
            return Some("Enter a valid email address.".into());
        }
        if blank(self.discord) {
            return Some("Enter your Discord username — that's where we coordinate.".into());
        }
        if blank(self.timezone) {
            return Some("Enter your timezone.".into());
        }
        if blank(self.line1) {
            return Some("Enter your street address.".into());
        }
        if blank(self.city) {
            return Some("Enter your city or town.".into());
        }
        if blank(self.postal_code) {
            return Some("Enter your postal or ZIP code.".into());
        }
        if blank(self.country) {
            return Some("Enter your country.".into());
        }
        if self.why_you.get().trim().chars().count() < MIN_WHY_YOU {
            return Some(format!(
                "Tell us why we should pick you — at least {MIN_WHY_YOU} characters."
            ));
        }
        None
    }

    /// The contact block as submitted. Untracked: it is read once, on submit.
    fn contact(&self) -> BetaContact {
        let value = |field: RwSignal<String>| field.get_untracked().trim().to_string();
        let optional = |field: RwSignal<String>| Some(value(field)).filter(|v| !v.is_empty());
        let first_name = value(self.first_name);
        let last_name = value(self.last_name);
        BetaContact {
            display_name: format!("{first_name} {last_name}"),
            first_name,
            last_name,
            email: value(self.email),
            discord: value(self.discord),
            timezone: value(self.timezone),
            date_of_birth: optional(self.date_of_birth),
            address: Some(PostalAddress {
                line1: value(self.line1),
                line2: optional(self.line2),
                city: value(self.city),
                region: optional(self.region),
                postal_code: value(self.postal_code),
                country: value(self.country),
            }),
            faa_id: optional(self.faa_id),
            country: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Submit {
    Idle,
    Sending,
    Failed(String),
}

#[component]
pub fn BetaSignup() -> impl IntoView {
    let auth = use_auth();
    let go = nav::use_go();

    let state = RwSignal::new(None::<BetaState>);
    let state_error = RwSignal::new(None::<String>);
    let paper = RwSignal::new(None::<QuizPaper>);
    let paper_error = RwSignal::new(None::<String>);

    let step = RwSignal::new(0usize);
    let answers = RwSignal::new(BTreeMap::<String, String>::new());
    // Which systems-check question is on screen. Kept here rather than in the
    // step, so coming back from the next step lands where they left off.
    let quiz_index = RwSignal::new(0usize);
    // Milliseconds each question has been on screen, keyed by question id,
    // and the one whose clock is running with when it started.
    let quiz_times = RwSignal::new(BTreeMap::<String, f64>::new());
    let quiz_clock = StoredValue::new(None::<(String, f64)>);
    let open_answers = RwSignal::new(BTreeMap::<String, String>::new());
    let submit_state = RwSignal::new(Submit::Idle);

    let details = Details::new();

    // Prefill from the Auth0 profile, but only where the applicant hasn't
    // already typed something — this effect re-runs when the profile resolves.
    Effect::new(move |_| {
        let Some(user) = auth.user.get() else {
            return;
        };
        let fill = |field: RwSignal<String>, value: Option<String>| {
            if field.get_untracked().is_empty()
                && let Some(value) = value
            {
                field.set(value);
            }
        };
        let (first, last) = profile_names(&user);
        fill(details.first_name, first);
        fill(details.last_name, last);
        fill(details.email, user.email.clone());
    });

    let refresh = move || {
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                return;
            };
            match api::beta_status(&token).await {
                Ok(current) => {
                    state_error.set(None);
                    state.set(Some(current));
                }
                Err(err) => state_error.set(Some(err)),
            }
        });
    };

    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            state.set(None);
            return;
        }
        refresh();
    });

    // The paper is only fetched for someone who can still apply — there's no
    // reason to pull it for a tester who is already through the door.
    Effect::new(move |_| {
        let can_apply = matches!(
            state.get().map(|s| s.status),
            Some(BetaStatus::None | BetaStatus::Withdrawn)
        );
        if !can_apply || paper.with(Option::is_some) {
            return;
        }
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                return;
            };
            match api::beta_quiz(&token).await {
                Ok(loaded) => {
                    paper_error.set(None);
                    paper.set(Some(loaded));
                }
                Err(err) => paper_error.set(Some(err)),
            }
        });
    });

    let questions = Signal::derive(move || {
        paper.with(|p| p.as_ref().map(|p| p.questions.clone()).unwrap_or_default())
    });
    let answered = Signal::derive(move || {
        let ids: Vec<String> = questions.get().into_iter().map(|q| q.id).collect();
        answers.with(|a| ids.iter().filter(|id| a.contains_key(*id)).count())
    });
    let total_questions = Signal::derive(move || questions.get().len());
    let open_questions = Signal::derive(move || {
        paper.with(|p| {
            p.as_ref()
                .map(|p| p.open_questions.clone())
                .unwrap_or_default()
        })
    });
    let quiz_complete = Signal::derive(move || {
        total_questions.get() > 0 && answered.get() == total_questions.get()
    });

    // The per-question clock. Whenever the question on screen changes — Next,
    // Previous, or leaving the step altogether — the outgoing question banks
    // its time and the incoming one starts. Nothing here is ever rendered.
    let on_screen = Memo::new(move |_| {
        if step.get() != 0 {
            return None;
        }
        let index = quiz_index.get();
        questions.with(|qs| qs.get(index).map(|q| q.id.clone()))
    });
    Effect::new(move |_| {
        let current = on_screen.get();
        let now = js_sys::Date::now();
        if let Some((id, since)) = quiz_clock.get_value() {
            quiz_times.update(|times| *times.entry(id).or_default() += now - since);
        }
        quiz_clock.set_value(current.map(|id| (id, now)));
    });

    // Mirrors the worker's rule. It re-checks on submit, so this is only here
    // to fail early rather than after a round trip.
    let open_problem = move || -> Option<String> {
        let current = open_answers.get();
        open_questions.get().into_iter().find_map(|question| {
            if !question.required {
                return None;
            }
            let answer = current
                .get(&question.id)
                .map(|a| a.trim())
                .unwrap_or_default();
            let length = answer.chars().count();
            if length == 0 {
                return Some("Answer every question before continuing.".to_string());
            }
            (length < question.min_length).then(|| {
                format!(
                    "\"{}\" needs at least {} characters.",
                    question.prompt, question.min_length,
                )
            })
        })
    };
    let open_ready = Signal::derive(move || open_problem().is_none());

    let sending = Signal::derive(move || submit_state.get() == Submit::Sending);

    let details_ready = Signal::derive(move || details.problem().is_none());

    let submit = move |()| {
        submit_state.set(Submit::Sending);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                submit_state.set(Submit::Failed(
                    "Your session expired. Sign in again and resubmit.".into(),
                ));
                return;
            };
            let application = BetaApplication {
                contact: details.contact(),
                why_you: details.why_you.get_untracked().trim().to_string(),
                answers: answers
                    .get_untracked()
                    .into_iter()
                    .map(|(question_id, choice_id)| {
                        let time_ms = quiz_times
                            .with_untracked(|times| times.get(&question_id).copied())
                            .unwrap_or_default()
                            .max(0.0)
                            .round() as u64;
                        QuizAnswer {
                            question_id,
                            choice_id,
                            time_ms,
                        }
                    })
                    .collect(),
                open_answers: open_answers
                    .get_untracked()
                    .into_iter()
                    .map(|(question_id, answer)| OpenAnswer {
                        question_id,
                        answer: answer.trim().to_string(),
                    })
                    .collect(),
            };
            match api::submit_beta_application(&token, &application).await {
                Ok(receipt) => {
                    submit_state.set(Submit::Idle);
                    // Move to the "under review" stage from the receipt rather
                    // than waiting on the refresh — otherwise the form flashes
                    // back up for as long as the status round-trip takes.
                    state.update(|current| {
                        let mut next = current.clone().unwrap_or_default();
                        next.status = BetaStatus::Requested;
                        next.application_id = Some(receipt.application_id);
                        next.score = Some(receipt.score);
                        *current = Some(next);
                    });
                    // …then reconcile with whatever the worker actually stored.
                    refresh();
                }
                Err(err) => submit_state.set(Submit::Failed(err)),
            }
        });
    };

    view! {
        <AnimatedPage class="min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <Header />

                {move || {
                    state_error
                        .get()
                        .map(|message| {
                            view! {
                                <Notice
                                    tone=NOTICE_ERROR
                                    title="Couldn't load your beta status"
                                    message=message
                                    spacing="mb-10"
                                />
                            }
                        })
                }}

                <Show
                    when=move || auth.is_authenticated.get()
                    fallback=move || view! { <SignedOut /> }
                >
                    {move || {
                        let Some(current) = state.get() else {
                            return view! { <Loading /> }.into_any();
                        };
                        match current.status {
                            BetaStatus::Requested => {
                                view! { <UnderReview state=current.clone() /> }.into_any()
                            }
                            BetaStatus::Rejected => {
                                view! { <NotSelected note=current.decision_note.clone() /> }
                                    .into_any()
                            }
                            BetaStatus::Accepted => {
                                view! { <Paperwork state=current.clone() go=go /> }.into_any()
                            }
                            BetaStatus::Approved => {
                                view! { <Approved state=current.clone() go=go /> }.into_any()
                            }
                            BetaStatus::None | BetaStatus::Withdrawn => {
                                view! {
                                    <Steps current=step />

                                    {move || {
                                        paper_error
                                            .get()
                                            .map(|message| {
                                                view! {
                                                    <Notice
                                                        tone=NOTICE_ERROR
                                                        title="Couldn't load the questionnaire"
                                                        message=message
                                                        spacing="mb-10"
                                                    />
                                                }
                                            })
                                    }}

                                    <Show when=move || step.get() == 0>
                                        <QuizStep
                                            questions=questions
                                            answers=answers
                                            index=quiz_index
                                            answered=answered
                                            total=total_questions
                                            complete=quiz_complete
                                            on_next=Callback::new(move |()| step.set(1))
                                        />
                                    </Show>

                                    <Show when=move || step.get() == 1>
                                        <OpenStep
                                            questions=open_questions
                                            answers=open_answers
                                            problem=Signal::derive(open_problem)
                                            ready=open_ready
                                            on_back=Callback::new(move |()| step.set(0))
                                            on_next=Callback::new(move |()| step.set(2))
                                        />
                                    </Show>

                                    <Show when=move || step.get() == 2>
                                        <DetailsStep
                                            details=details
                                            problem=Signal::derive(move || details.problem())
                                            ready=details_ready
                                            on_back=Callback::new(move |()| step.set(1))
                                            on_next=Callback::new(move |()| step.set(3))
                                        />
                                    </Show>

                                    <Show when=move || step.get() == 3>
                                        <ReviewStep
                                            details=details
                                            answered=answered
                                            total=total_questions
                                            failure=Signal::derive(move || match submit_state.get() {
                                                Submit::Failed(message) => Some(message),
                                                _ => None,
                                            })
                                            sending=sending
                                            on_back=Callback::new(move |()| step.set(2))
                                            on_submit=Callback::new(submit)
                                        />
                                    </Show>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </Show>
            </div>
        </AnimatedPage>
    }
}

#[component]
fn Header() -> impl IntoView {
    let auth = use_auth();
    let go = nav::use_go();

    view! {
        <header class="mb-16 max-w-3xl">
            <p class=cn!("mb-10", EYEBROW)>"Infinity / Beta"</p>
            <h1 class="text-5xl leading-[1.02] font-extralight tracking-tight md:text-7xl">
                "Fly the DC-10"
                <br />
                <span class="font-semibold text-accent-300">"before anyone else."</span>
            </h1>
            <p class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg">
                "We're picking a small group who know the aircraft and can report a bug properly. Start with a short systems questionnaire, then tell us who you are and why you."
            </p>
            <div class="mt-6 flex flex-wrap items-center gap-x-6 gap-y-2 font-mono text-[11px]">
                <span class="tracking-[0.2em] text-white/40 uppercase">
                    "Closed beta  /  Aerodynamics DC-10"
                </span>
                // Staff shortcut into the review queue. The route gates itself
                // and the worker gates the data; this only saves typing a URL.
                <Show when=move || auth.entitlements.get().is_developer()>
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::BETA_ADMIN.to_string())
                        class="group inline-flex items-center gap-2 tracking-[0.2em] text-amber-300 uppercase transition-colors hover:text-amber-200"
                    >
                        "Review applications"
                        <Icon
                            icon=icons::ARROW_RIGHT
                            class="h-3 w-3 transition-transform group-hover:translate-x-0.5"
                        />
                    </button>
                </Show>
            </div>
        </header>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! {
        <div class=cn!("flex items-center gap-3 border-t border-white/10 py-10", EYEBROW)>
            <Spinner size=Size::Sm />
            "Loading…"
        </div>
    }
}

/// A left-ruled status line: mono title over the message.
#[component]
fn Notice(
    tone: &'static str,
    #[prop(into)] title: String,
    #[prop(into)] message: String,
    #[prop(optional)] spacing: &'static str,
) -> impl IntoView {
    view! {
        <div class=cn!("border-l pl-4 text-sm leading-relaxed", tone, spacing)>
            <p class="mb-1 font-mono text-[11px] tracking-[0.2em] uppercase">{title}</p>
            <p class="font-light opacity-80">{message}</p>
        </div>
    }
}

/// Small print block: mono label over light copy, under a hairline.
#[component]
fn Note(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="mt-10 max-w-2xl border-t border-white/10 pt-5">
            <p class="mb-2 font-mono text-[11px] tracking-[0.25em] text-accent-400/80 uppercase">
                {label}
            </p>
            <p class="text-sm leading-relaxed font-light text-white/65">{children()}</p>
        </div>
    }
}

#[component]
fn SignedOut() -> impl IntoView {
    let auth = use_auth();
    view! {
        <section class="border-t border-white/10 pt-10">
            <h2 class="mb-4 text-3xl font-extralight tracking-tight md:text-4xl">
                "Sign in "
                <span class="font-semibold text-accent-300">"to apply."</span>
            </h2>
            <p class="mb-8 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                "Applications are tied to your account so we can track your status and, if you're selected, hand you the paperwork on this same page."
            </p>
            <button
                type="button"
                on:click=move |_| auth.login()
                class="group inline-flex items-center gap-2 text-lg font-medium transition-colors hover:text-accent-300"
            >
                "Sign in"
                <Icon
                    icon=icons::ARROW_RIGHT
                    class="h-4 w-4 transition-transform group-hover:translate-x-1"
                />
            </button>
        </section>
    }
}

#[component]
fn Steps(#[prop(into)] current: Signal<usize>) -> impl IntoView {
    view! {
        <ol class="mb-16 grid grid-cols-2 gap-x-6 gap-y-4 md:grid-cols-4">
            {STEPS
                .iter()
                .enumerate()
                .map(|(index, label)| {
                    let done = Signal::derive(move || current.get() > index);
                    let active = Signal::derive(move || current.get() == index);
                    view! {
                        <li class=move || {
                            cn!(
                                "border-t pt-3 transition-colors",
                                if active.get() {
                                    "border-accent-400"
                                } else if done.get() {
                                    "border-accent-400/40"
                                } else {
                                    "border-white/10"
                                },
                            )
                        }>
                            <p class="mb-1 font-mono text-[11px] text-accent-400/80">
                                {format!("{:02}", index + 1)}
                            </p>
                            <p class=move || {
                                cn!(
                                    "truncate text-sm",
                                    if active.get() {
                                        "font-medium text-white"
                                    } else {
                                        "font-light text-white/45"
                                    },
                                )
                            }>{*label}</p>
                        </li>
                    }
                })
                .collect_view()}
        </ol>
    }
}

/// The multiple-choice half, one question per screen. That is what makes the
/// per-question clock in [`BetaSignup`] mean anything: with every question on
/// one page there is no telling which one someone was looking at.
#[component]
fn QuizStep(
    #[prop(into)] questions: Signal<Vec<api::QuizQuestion>>,
    answers: RwSignal<BTreeMap<String, String>>,
    index: RwSignal<usize>,
    #[prop(into)] answered: Signal<usize>,
    #[prop(into)] total: Signal<usize>,
    #[prop(into)] complete: Signal<bool>,
    #[prop(into)] on_next: Callback<()>,
) -> impl IntoView {
    let current = Signal::derive(move || questions.with(|qs| qs.get(index.get()).cloned()));
    let current_answered = Signal::derive(move || {
        current.with(|q| {
            q.as_ref()
                .is_some_and(|q| answers.with(|a| a.contains_key(&q.id)))
        })
    });
    let last = Signal::derive(move || index.get() + 1 >= total.get());

    view! {
        <section>
            <SectionLabel num="01" label="Systems check" />
            <p class="mb-10 max-w-2xl text-sm leading-relaxed font-light text-white/65">
                "A short set of questions on how the DC-10 actually works, one at a time. We're not looking for a perfect score — we're looking for someone who can tell a hydraulic problem from an electrical one when they file a report. Your score goes to us with the application; it doesn't block you from submitting."
            </p>

            <div class="mb-10 flex items-center gap-4">
                <div class="relative h-px flex-1 bg-white/10">
                    <div
                        class="absolute inset-y-0 left-0 bg-accent-400 transition-all duration-300"
                        style:width=move || {
                            let total = total.get();
                            let fraction = if total == 0 {
                                0.0
                            } else {
                                answered.get() as f64 / total as f64
                            };
                            format!("{}%", fraction * 100.0)
                        }
                    ></div>
                </div>
                <div class="shrink-0 font-mono text-[11px] tabular-nums text-white/50">
                    {move || {
                        let total = total.get();
                        format!("Question {:02} / {total:02}", (index.get() + 1).min(total))
                    }}
                </div>
            </div>

            <Show when=move || current.with(Option::is_some) fallback=|| view! { <Loading /> }>
                {move || {
                    current
                        .get()
                        .map(|question| {
                            view! {
                                <QuizQuestionRow
                                    question=question
                                    number=index.get() + 1
                                    answers=answers
                                />
                            }
                        })
                }}
            </Show>

            <div class="mt-8 flex flex-col-reverse gap-6 sm:flex-row sm:items-center sm:justify-between">
                <button
                    type="button"
                    disabled=move || index.get() == 0
                    on:click=move |_| index.update(|i| *i = i.saturating_sub(1))
                    class=BACK_LINK
                >
                    <Icon
                        icon=icons::ARROW_LEFT
                        class="h-4 w-4 transition-transform group-hover:-translate-x-0.5"
                    />
                    "Previous question"
                </button>
                <Button
                    variant=Variant::Solid
                    icon=icons::ARROW_RIGHT
                    disabled=Signal::derive(move || {
                        if last.get() { !complete.get() } else { !current_answered.get() }
                    })
                    on_click=Callback::new(move |()| {
                        if last.get() {
                            on_next.run(());
                        } else {
                            index.update(|i| *i += 1);
                        }
                    })
                >
                    {move || if last.get() { "Continue" } else { "Next question" }}
                </Button>
            </div>
        </section>
    }
}

#[component]
fn QuizQuestionRow(
    question: api::QuizQuestion,
    number: usize,
    answers: RwSignal<BTreeMap<String, String>>,
) -> impl IntoView {
    let id = question.id.clone();
    let selected_id = question.id.clone();
    let selected =
        Signal::derive(move || answers.with(|a| a.get(&selected_id).cloned().unwrap_or_default()));
    let on_change = Callback::new(move |choice: String| {
        answers.update(|a| {
            a.insert(id.clone(), choice);
        });
    });

    view! {
        <div class="grid gap-4 border-y border-white/10 py-6 md:grid-cols-[14rem_1fr] md:gap-8">
            <div class="font-mono text-[11px] tracking-[0.2em] uppercase">
                // The number lights up once the question is answered.
                <span class=move || {
                    if selected.get().is_empty() { "text-white/30" } else { "text-accent-400/80" }
                }>{format!("{number:02}")}</span>
                <span class="ml-3 text-white/45">{question.topic.clone()}</span>
            </div>
            <div>
                <p class="mb-4 text-base font-medium text-white">{question.prompt.clone()}</p>
                <div class="space-y-1">
                    {question
                        .choices
                        .iter()
                        .map(|choice| {
                            view! {
                                <Radio
                                    name=question.id.clone()
                                    value=choice.id.clone()
                                    selected=selected
                                    on_change=on_change
                                    label=choice.text.clone()
                                    class="w-full rounded-sm px-3 py-2 leading-6 font-light transition-colors hover:bg-white/[0.03]"
                                />
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

/// The written half of the questionnaire. Nothing here is scored — it is what
/// a reviewer actually reads, so the UI gives it room rather than treating it
/// as an afterthought under the multiple choice.
#[component]
fn OpenStep(
    #[prop(into)] questions: Signal<Vec<api::OpenQuestion>>,
    answers: RwSignal<BTreeMap<String, String>>,
    #[prop(into)] problem: Signal<Option<String>>,
    #[prop(into)] ready: Signal<bool>,
    #[prop(into)] on_back: Callback<()>,
    #[prop(into)] on_next: Callback<()>,
) -> impl IntoView {
    view! {
        <section>
            <SectionLabel num="02" label="In your own words" />
            <p class="mb-10 max-w-2xl text-sm leading-relaxed font-light text-white/65">
                "This is the part we actually read. Short and specific beats long and general — we are trying to work out whether you would be useful in a bug thread at 1am, not whether you can write."
            </p>

            <div class="border-b border-white/10">
                {move || {
                    questions
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(index, question)| {
                            let id = question.id.clone();
                            let value_id = question.id.clone();
                            let value = Signal::derive(move || {
                                answers.with(|a| a.get(&value_id).cloned().unwrap_or_default())
                            });
                            let on_input = Callback::new(move |text: String| {
                                answers
                                    .update(|a| {
                                        a.insert(id.clone(), text);
                                    });
                            });
                            let min_length = question.min_length;
                            let required = question.required;
                            view! {
                                <div class="grid gap-4 border-t border-white/10 py-6 md:grid-cols-[14rem_1fr] md:gap-8">
                                    <div class="font-mono text-[11px] tracking-[0.2em] uppercase">
                                        <span class="text-accent-400/80">
                                            {format!("{:02}", index + 1)}
                                        </span>
                                        {(!required)
                                            .then(|| {
                                                view! {
                                                    <span class="ml-3 text-white/40">"Optional"</span>
                                                }
                                            })}
                                    </div>
                                    <div>
                                        <p class="mb-3 text-base font-medium text-white">
                                            {question.prompt.clone()}
                                        </p>
                                        <TextArea
                                            value=value
                                            on_input=on_input
                                            rows=5
                                            placeholder=question.hint.clone().unwrap_or_default()
                                        />
                                        <div class="mt-1.5 flex items-center justify-between gap-3 text-xs">
                                            <span class="font-light text-white/40">
                                                {question.hint.clone().unwrap_or_default()}
                                            </span>
                                            <span class=move || {
                                                let length = value.get().trim().chars().count();
                                                cn!(
                                                    "shrink-0 font-mono text-[11px] tabular-nums",
                                                    if length >= min_length {
                                                        "text-white/35"
                                                    } else {
                                                        "text-amber-300/70"
                                                    },
                                                )
                                            }>
                                                {move || {
                                                    let length = value.get().trim().chars().count();
                                                    if min_length > 0 && length < min_length {
                                                        format!("{length} / {min_length}")
                                                    } else {
                                                        format!("{length}")
                                                    }
                                                }}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>

            {move || {
                problem
                    .get()
                    .map(|message| {
                        view! {
                            <Notice
                                tone=NOTICE_WARNING
                                title="Not quite there"
                                message=message
                                spacing="mt-6"
                            />
                        }
                    })
            }}

            <div class="mt-8 flex flex-col-reverse gap-6 sm:flex-row sm:items-center sm:justify-between">
                <button type="button" on:click=move |_| on_back.run(()) class=BACK_LINK>
                    <Icon
                        icon=icons::ARROW_LEFT
                        class="h-4 w-4 transition-transform group-hover:-translate-x-0.5"
                    />
                    "Back to the questions"
                </button>
                <Button
                    variant=Variant::Solid
                    icon=icons::ARROW_RIGHT
                    disabled=Signal::derive(move || !ready.get())
                    on_click=on_next
                >
                    "Continue to your details"
                </Button>
            </div>
        </section>
    }
}

#[component]
fn DetailsStep(
    details: Details,
    #[prop(into)] problem: Signal<Option<String>>,
    #[prop(into)] ready: Signal<bool>,
    #[prop(into)] on_back: Callback<()>,
    #[prop(into)] on_next: Callback<()>,
) -> impl IntoView {
    // Every input here writes straight to one signal.
    let set = |field: RwSignal<String>| Callback::new(move |value| field.set(value));

    view! {
        <section>
            <SectionLabel num="03" label="Your details" />
            <h2 class="mb-8 text-3xl font-extralight tracking-tight md:text-4xl">
                "Who "
                <span class="font-semibold">"you are"</span>
            </h2>

            <div class="grid gap-4 md:grid-cols-2">
                <Field label="First name" required=true>
                    <TextInput
                        value=details.first_name
                        on_input=set(details.first_name)
                        placeholder="Jane"
                    />
                </Field>
                <Field label="Last name" required=true>
                    <TextInput
                        value=details.last_name
                        on_input=set(details.last_name)
                        placeholder="Public"
                    />
                </Field>
                <Field
                    label="Date of birth"
                    required=true
                    hint=format!("Testers need to be {MIN_AGE} or older.")
                >
                    <TextInput
                        value=details.date_of_birth
                        on_input=set(details.date_of_birth)
                        input_type="date"
                        // Light picker icon and calendar on the dark field.
                        class="[color-scheme:dark]"
                    />
                </Field>
                <Field
                    label="FAA ID number (optional)"
                    hint="Your FAA Tracking Number (FTN), if you hold a certificate."
                >
                    <TextInput value=details.faa_id on_input=set(details.faa_id) />
                </Field>
            </div>

            <p class=cn!("mt-10 mb-4", EYEBROW)>"How we reach you"</p>
            <div class="grid gap-4 md:grid-cols-3">
                <Field label="Email" required=true>
                    <TextInput
                        value=details.email
                        on_input=set(details.email)
                        placeholder="you@example.com"
                        input_type="email"
                    />
                </Field>
                <Field
                    label="Discord username"
                    required=true
                    hint="Where the beta actually happens: voice, screen share, hotfix pings."
                >
                    <TextInput
                        value=details.discord
                        on_input=set(details.discord)
                        placeholder="username"
                    />
                </Field>
                <Field label="Timezone" required=true hint="So we know when you're reachable.">
                    <TextInput
                        value=details.timezone
                        on_input=set(details.timezone)
                        placeholder="America/Denver"
                    />
                </Field>
            </div>

            <p class=cn!("mt-10 mb-4", EYEBROW)>"Home address"</p>
            <div class="grid gap-4">
                <Field label="Street address" required=true>
                    <TextInput
                        value=details.line1
                        on_input=set(details.line1)
                        placeholder="1 Example Street"
                    />
                </Field>
                <Field label="Apartment, suite, etc. (optional)">
                    <TextInput
                        value=details.line2
                        on_input=set(details.line2)
                        placeholder="Apartment 4"
                    />
                </Field>
            </div>
            <div class="mt-4 grid gap-4 md:grid-cols-3">
                <Field label="City or town" required=true>
                    <TextInput value=details.city on_input=set(details.city) placeholder="Denver" />
                </Field>
                <Field label="State / region" hint="If your country uses one.">
                    <TextInput value=details.region on_input=set(details.region) placeholder="CO" />
                </Field>
                <Field label="Postal / ZIP code" required=true>
                    <TextInput
                        value=details.postal_code
                        on_input=set(details.postal_code)
                        placeholder="80202"
                    />
                </Field>
            </div>
            <Field label="Country" required=true class="mt-4">
                <TextInput
                    value=details.country
                    on_input=set(details.country)
                    placeholder="United States"
                />
            </Field>

            <Field
                label="Why should we choose you?"
                required=true
                hint="Be specific. What you fly, how you report a bug, what you'd stress-test on a DC-10."
                class="mt-10"
            >
                <TextArea
                    value=details.why_you
                    on_input=set(details.why_you)
                    rows=7
                    placeholder="I've flown the DC-10 in every sim that's had one and I know the systems well enough to tell you which one is misbehaving. I file reports with steps, expected vs actual, and a log…"
                />
            </Field>
            <div class="mt-1 text-right font-mono text-[11px] tabular-nums text-white/35">
                {move || format!("{} characters", details.why_you.get().trim().chars().count())}
            </div>

            {move || {
                problem
                    .get()
                    .map(|message| {
                        view! {
                            <Notice
                                tone=NOTICE_WARNING
                                title="Not quite there"
                                message=message
                                spacing="mt-6"
                            />
                        }
                    })
            }}

            <div class="mt-8 flex flex-col-reverse gap-6 sm:flex-row sm:items-center sm:justify-between">
                <button type="button" on:click=move |_| on_back.run(()) class=BACK_LINK>
                    <Icon
                        icon=icons::ARROW_LEFT
                        class="h-4 w-4 transition-transform group-hover:-translate-x-0.5"
                    />
                    "Back to the questions"
                </button>
                <Button
                    variant=Variant::Solid
                    icon=icons::ARROW_RIGHT
                    disabled=Signal::derive(move || !ready.get())
                    on_click=on_next
                >
                    "Review"
                </Button>
            </div>
        </section>
    }
}

#[component]
fn ReviewStep(
    details: Details,
    #[prop(into)] answered: Signal<usize>,
    #[prop(into)] total: Signal<usize>,
    #[prop(into)] failure: Signal<Option<String>>,
    #[prop(into)] sending: Signal<bool>,
    #[prop(into)] on_back: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
) -> impl IntoView {
    view! {
        <section>
            <SectionLabel num="04" label="Review" />
            <h2 class="mb-8 text-3xl font-extralight tracking-tight md:text-4xl">
                "Check it "
                <span class="font-semibold">"over"</span>
            </h2>

            <dl class="border-t border-white/10">
                <ReviewRow label="Name" value=Signal::derive(move || details.full_name()) />
                <ReviewRow label="Date of birth" value=details.date_of_birth />
                <ReviewRow
                    label="FAA ID"
                    value=Signal::derive(move || {
                        let id = details.faa_id.get().trim().to_string();
                        if id.is_empty() { "Not provided".to_string() } else { id }
                    })
                />
                <ReviewRow label="Email" value=details.email />
                <ReviewRow label="Discord" value=details.discord />
                <ReviewRow label="Timezone" value=details.timezone />
                <div class=SPEC_ROW>
                    <dt class=SPEC_LABEL>"Home address"</dt>
                    <dd class="text-sm leading-6 font-light whitespace-pre-line text-white/80">
                        {move || details.address_block()}
                    </dd>
                </div>
                <ReviewRow
                    label="Questionnaire"
                    value=Signal::derive(move || {
                        format!("{} of {} answered", answered.get(), total.get())
                    })
                />
                <div class=SPEC_ROW>
                    <dt class=SPEC_LABEL>"Why you"</dt>
                    <dd class="max-h-56 overflow-y-auto text-sm leading-6 font-light whitespace-pre-wrap text-white/75">
                        {move || details.why_you.get()}
                    </dd>
                </div>
            </dl>

            <Note label="What happens next">
                "We grade the questionnaire and read every application. If you're selected you'll hear from us on Discord or email, and this page will switch over to the NDA and beta agreement for you to sign."
            </Note>

            {move || {
                failure
                    .get()
                    .map(|message| {
                        view! {
                            <Notice
                                tone=NOTICE_ERROR
                                title="Not submitted"
                                message=message
                                spacing="mt-6"
                            />
                        }
                    })
            }}

            <div class="mt-8 flex flex-col-reverse gap-6 sm:flex-row sm:items-center sm:justify-between">
                <button
                    type="button"
                    disabled=move || sending.get()
                    on:click=move |_| on_back.run(())
                    class=BACK_LINK
                >
                    <Icon
                        icon=icons::ARROW_LEFT
                        class="h-4 w-4 transition-transform group-hover:-translate-x-0.5"
                    />
                    "Back"
                </button>
                <Button
                    variant=Variant::Solid
                    icon=icons::CIRCLE_CHECK
                    loading=sending
                    on_click=on_submit
                >
                    "Submit application"
                </Button>
            </div>

            <p class="mt-6 text-xs leading-relaxed font-light text-white/40">
                "Applying doesn't guarantee a place. We're keeping the group small so the feedback loop stays tight."
            </p>
        </section>
    }
}

#[component]
fn ReviewRow(#[prop(into)] label: String, #[prop(into)] value: Signal<String>) -> impl IntoView {
    view! {
        <div class=SPEC_ROW>
            <dt class=SPEC_LABEL>{label}</dt>
            <dd class="truncate text-sm font-light text-white/80">{move || value.get()}</dd>
        </div>
    }
}

#[component]
fn UnderReview(state: BetaState) -> impl IntoView {
    let reference = state.application_id.clone();
    let score = state.score;
    view! {
        <section>
            <SectionLabel num="01" label="Your application" />
            <p class=cn!(STATUS, "text-accent-300")>"Under review"</p>
            <h2 class="text-3xl font-extralight tracking-tight md:text-4xl">
                "Application "
                <span class="font-semibold">"received"</span>
            </h2>
            <p class="mt-2 font-light text-white/50">"We're reading it."</p>

            <p class="mt-6 mb-10 max-w-xl border-l border-accent-400/40 pl-5 text-sm leading-relaxed font-light text-white/65">
                "If you're selected we'll reach out on Discord or by email, and this page will switch over to the NDA and beta agreement for you to sign. Nothing else to do for now."
            </p>

            <dl class="border-t border-white/10">
                {reference
                    .map(|id| {
                        view! { <ReviewRow label="Reference" value=Signal::derive(move || id.clone()) /> }
                    })}
                {score
                    .map(|score| {
                        view! { <ScoreRow score=score /> }
                    })}
            </dl>
        </section>
    }
}

#[component]
fn ScoreRow(score: QuizScore) -> impl IntoView {
    let value = format!("{}/{} ({}%)", score.correct, score.total, score.percent);
    view! { <ReviewRow label="Systems check" value=Signal::derive(move || value.clone()) /> }
}

#[component]
fn NotSelected(note: Option<String>) -> impl IntoView {
    view! {
        <section>
            <SectionLabel num="01" label="Your application" />
            <p class=cn!(STATUS, "text-white/50")>"Closed"</p>
            <h2 class="mb-6 text-3xl font-extralight tracking-tight md:text-4xl">
                "Not selected "
                <span class="font-semibold">"this time"</span>
            </h2>
            <p class="max-w-xl border-l border-white/20 pl-5 text-sm leading-relaxed font-light text-white/65">
                "We had more good applications than places. This isn't a judgement on you, and it doesn't rule you out of a later test round."
            </p>
            {note
                .map(|note| {
                    view! {
                        <p class="mt-8 max-w-2xl border-t border-white/10 pt-5 text-sm leading-6 font-light whitespace-pre-wrap text-white/70">
                            {note}
                        </p>
                    }
                })}
        </section>
    }
}

/// Stage two: the tester was accepted and now has documents waiting.
#[component]
fn Paperwork(state: BetaState, go: Callback<String>) -> impl IntoView {
    let complete = state.documents_complete;
    let documents = state.documents.clone();
    // The worker decides what there is to sign — today one combined agreement —
    // so the copy follows the count rather than assuming one.
    let single = documents.len() == 1;
    let to_sign = if single {
        "Your application was accepted. One agreement to sign.".to_string()
    } else {
        format!(
            "Your application was accepted. {} documents to sign.",
            documents.len()
        )
    };

    view! {
        <section class="mb-20">
            <SectionLabel num="01" label="Your application" />
            <p class=cn!(STATUS, if complete { "text-amber-300" } else { "text-accent-300" })>
                {if complete { "Final check" } else { "Accepted" }}
            </p>
            <h2 class="max-w-3xl text-3xl font-extralight tracking-tight md:text-4xl">
                {if complete { "Paperwork received — final check".to_string() } else { to_sign }}
            </h2>
            <p class="mt-6 max-w-xl border-l border-accent-400/40 pl-5 text-sm leading-relaxed font-light text-white/65">
                {match (complete, single) {
                    (true, _) => {
                        "Your paperwork is signed and with us. We check it over and then release your build — nothing else for you to do."
                    }
                    (false, true) => {
                        "Read it through and e-sign it below. Once it's in we do a final check and then you're a tester."
                    }
                    (false, false) => {
                        "Read and e-sign each one below. Once they're in we do a final check and then you're a tester."
                    }
                }}
            </p>
        </section>

        <section>
            <SectionLabel num="02" label="Documents" />
            <div class="border-b border-white/10">
                {documents
                    .into_iter()
                    .map(|doc| {
                        view! { <DocumentRow doc=doc go=go /> }
                    })
                    .collect_view()}
            </div>

            <Note label="Before you sign">
                "Read it properly — the NDA is what stops the aircraft leaking before release, and it binds you. If anything in it doesn't work for you, tell us before signing rather than after."
            </Note>
        </section>
    }
}

/// Past the second gate: paperwork checked, role granted, build released.
#[component]
fn Approved(state: BetaState, go: Callback<String>) -> impl IntoView {
    // Read twice: once to decide whether the section renders, once to fill it.
    let documents = StoredValue::new(state.documents.clone());

    view! {
        <section class="mb-20">
            <SectionLabel num="01" label="Your application" />
            <p class=cn!(STATUS, "text-accent-300")>"Approved"</p>
            <h2 class="text-3xl font-extralight tracking-tight md:text-5xl">
                "You're a DC-10 "
                <span class="font-semibold text-accent-300">"beta tester"</span>
            </h2>
            <p class="mt-3 font-light text-white/50">
                "Paperwork checked and your account is entitled."
            </p>

            <p class="mt-6 max-w-xl border-l border-accent-400/40 pl-5 text-sm leading-relaxed font-light text-white/65">
                "The build shows up in the Infinity Manager under the beta channel. If it is not there yet, sign out and back in once — your entitlement is baked into the token."
            </p>

            <Notice
                tone=NOTICE_WARNING
                title="The NDA is live"
                message="No screenshots, no video, no streaming, nothing outside the tester channels. That holds until we say otherwise in writing."
                spacing="mt-8 max-w-xl"
            />
        </section>

        <Show when=move || !documents.with_value(Vec::is_empty)>
            <section>
                <SectionLabel num="02" label="Your signed paperwork" />
                <div class="border-b border-white/10">
                    {documents
                        .get_value()
                        .into_iter()
                        .map(|doc| view! { <DocumentRow doc=doc go=go /> })
                        .collect_view()}
                </div>
            </section>
        </Show>
    }
}

/// One document, as a clickable row out to its e-signing page.
#[component]
fn DocumentRow(doc: BetaDocumentState, go: Callback<String>) -> impl IntoView {
    let signed = doc.signed;
    let target = nav::beta_sign(&doc.id);
    let superseded = doc.superseded_version.clone();

    view! {
        <button
            type="button"
            on:click=move |_| go.run(target.clone())
            class="group relative grid w-full gap-3 border-t border-white/10 py-6 text-left transition-colors hover:bg-white/[0.03] md:grid-cols-[1fr_auto] md:items-center md:gap-8"
        >
            <HoverRule />
            <div class="min-w-0">
                <div class="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                    <h3 class="text-lg font-medium">{doc.title.clone()}</h3>
                    <span class="font-mono text-[11px] text-white/40">
                        {format!("v{}", doc.version)}
                    </span>
                    {signed
                        .then(|| {
                            view! {
                                <span class="font-mono text-[11px] tracking-[0.2em] text-accent-300 uppercase">
                                    "Signed"
                                </span>
                            }
                        })}
                </div>
                <p class="mt-1 max-w-2xl text-sm leading-relaxed font-light text-white/55">
                    {doc.summary.clone()}
                </p>
                {doc
                    .signed_at
                    .clone()
                    .map(|at| {
                        view! {
                            <p class="mt-2 font-mono text-[11px] text-white/40">
                                {format!("Signed {at}")}
                            </p>
                        }
                    })}
                {superseded
                    .map(|old| {
                        view! {
                            <p class="mt-2 border-l border-amber-400/60 pl-3 text-xs text-amber-200/90">
                                {format!(
                                    "You signed v{old}. This document has been updated — please sign the current version.",
                                )}
                            </p>
                        }
                    })}
            </div>
            <span class=cn!(
                "inline-flex shrink-0 items-center gap-2 text-sm font-medium",
                if signed { "text-white/60" } else { "text-accent-300" },
            )>
                {if signed { "View" } else { "Read and sign" }}
                <Icon
                    icon=icons::ARROW_RIGHT
                    class="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5"
                />
            </span>
        </button>
    }
}
