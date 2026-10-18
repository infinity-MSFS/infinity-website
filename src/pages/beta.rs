//! Closed beta application for the Aerodynamics DC-10.
//!
//! Two stages, both living on this route because the tester is told to "check
//! the beta page again" after a decision:
//!
//!   1. Apply — a short DC-10 systems questionnaire, then contact details and
//!      a case for why we should pick them. The questionnaire is graded on the
//!      worker: the answer key never reaches the browser, and the score rides
//!      along with the application as advisory context rather than a gate.
//!   2. Sign — once accepted, the NDA and test agreement appear here as cards
//!      that link out to the e-signing page.
//!
//! Everything shown is driven by `api::beta_status`, so the stage the applicant
//! sees is the stage the worker says they are in. The stepper is presentation.

use std::collections::BTreeMap;

use infinity_ui::prelude::*;
use leptos::task::spawn_local;

use crate::api::{
    self, BetaApplication, BetaContact, BetaDocumentState, BetaState, BetaStatus, OpenAnswer,
    QuizAnswer, QuizPaper, QuizScore,
};
use crate::auth::use_auth;
use crate::components::{EYEBROW, HoverRule, SectionLabel};
use crate::nav;

const MIN_WHY_YOU: usize = 40;

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
    let open_answers = RwSignal::new(BTreeMap::<String, String>::new());
    let submit_state = RwSignal::new(Submit::Idle);

    let display_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let discord = RwSignal::new(String::new());
    let timezone = RwSignal::new(local_timezone());
    let why_you = RwSignal::new(String::new());

    // Prefill from the Auth0 profile, but only where the applicant hasn't
    // already typed something — this effect re-runs when the profile resolves.
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

    let details_problem = move || -> Option<String> {
        if display_name.get().trim().is_empty() {
            return Some("Enter your name, or the name you go by.".into());
        }
        let address = email.get();
        if address.trim().is_empty() || !address.contains('@') {
            return Some("Enter a valid email address.".into());
        }
        if discord.get().trim().is_empty() {
            return Some("Enter your Discord username — that's where we coordinate.".into());
        }
        if timezone.get().trim().is_empty() {
            return Some("Enter your timezone.".into());
        }
        let why = why_you.get();
        if why.trim().chars().count() < MIN_WHY_YOU {
            return Some(format!(
                "Tell us why we should pick you — at least {MIN_WHY_YOU} characters."
            ));
        }
        None
    };
    let details_ready = Signal::derive(move || details_problem().is_none());

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
                contact: BetaContact {
                    display_name: display_name.get_untracked().trim().to_string(),
                    email: email.get_untracked().trim().to_string(),
                    discord: discord.get_untracked().trim().to_string(),
                    timezone: timezone.get_untracked().trim().to_string(),
                    country: None,
                },
                why_you: why_you.get_untracked().trim().to_string(),
                answers: answers
                    .get_untracked()
                    .into_iter()
                    .map(|(question_id, choice_id)| QuizAnswer {
                        question_id,
                        choice_id,
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
                                            display_name=display_name
                                            email=email
                                            discord=discord
                                            timezone=timezone
                                            why_you=why_you
                                            problem=Signal::derive(details_problem)
                                            ready=details_ready
                                            on_back=Callback::new(move |()| step.set(1))
                                            on_next=Callback::new(move |()| step.set(3))
                                        />
                                    </Show>

                                    <Show when=move || step.get() == 3>
                                        <ReviewStep
                                            display_name=display_name
                                            email=email
                                            discord=discord
                                            timezone=timezone
                                            why_you=why_you
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

#[component]
fn QuizStep(
    #[prop(into)] questions: Signal<Vec<api::QuizQuestion>>,
    answers: RwSignal<BTreeMap<String, String>>,
    #[prop(into)] answered: Signal<usize>,
    #[prop(into)] total: Signal<usize>,
    #[prop(into)] complete: Signal<bool>,
    #[prop(into)] on_next: Callback<()>,
) -> impl IntoView {
    view! {
        <section>
            <SectionLabel num="01" label="Systems check" />
            <p class="mb-10 max-w-2xl text-sm leading-relaxed font-light text-white/65">
                "Ten questions on how the DC-10 actually works. We're not looking for a perfect score — we're looking for someone who can tell a hydraulic problem from an electrical one when they file a report. Your score goes to us with the application; it doesn't block you from submitting."
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
                    {move || format!("{:02} / {:02} answered", answered.get(), total.get())}
                </div>
            </div>

            <Show when=move || !questions.get().is_empty() fallback=|| view! { <Loading /> }>
                <div class="border-b border-white/10">
                    {move || {
                        questions
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(index, question)| {
                                let id = question.id.clone();
                                let selected_id = question.id.clone();
                                let selected = Signal::derive(move || {
                                    answers.with(|a| a.get(&selected_id).cloned().unwrap_or_default())
                                });
                                let on_change = Callback::new(move |choice: String| {
                                    answers.update(|a| {
                                        a.insert(id.clone(), choice);
                                    });
                                });
                                view! {
                                    <div class="grid gap-4 border-t border-white/10 py-6 md:grid-cols-[14rem_1fr] md:gap-8">
                                        <div class="font-mono text-[11px] tracking-[0.2em] uppercase">
                                            // The number lights up once the question is answered.
                                            <span class=move || {
                                                if selected.get().is_empty() {
                                                    "text-white/30"
                                                } else {
                                                    "text-accent-400/80"
                                                }
                                            }>{format!("{:02}", index + 1)}</span>
                                            <span class="ml-3 text-white/45">
                                                {question.topic.clone()}
                                            </span>
                                        </div>
                                        <div>
                                            <p class="mb-4 text-base font-medium text-white">
                                                {question.prompt.clone()}
                                            </p>
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
                            })
                            .collect_view()
                    }}
                </div>
            </Show>

            <div class="mt-8 flex flex-col gap-3 sm:flex-row sm:justify-end">
                <Button
                    variant=Variant::Solid
                    icon=icons::ARROW_RIGHT
                    disabled=Signal::derive(move || !complete.get())
                    on_click=on_next
                >
                    {move || {
                        if complete.get() {
                            "Continue".to_string()
                        } else {
                            format!("{} left to answer", total.get().saturating_sub(answered.get()))
                        }
                    }}
                </Button>
            </div>
        </section>
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
    display_name: RwSignal<String>,
    email: RwSignal<String>,
    discord: RwSignal<String>,
    timezone: RwSignal<String>,
    why_you: RwSignal<String>,
    #[prop(into)] problem: Signal<Option<String>>,
    #[prop(into)] ready: Signal<bool>,
    #[prop(into)] on_back: Callback<()>,
    #[prop(into)] on_next: Callback<()>,
) -> impl IntoView {
    view! {
        <section>
            <SectionLabel num="03" label="Your details" />
            <h2 class="mb-8 text-3xl font-extralight tracking-tight md:text-4xl">
                "How we "
                <span class="font-semibold">"reach you"</span>
            </h2>

            <div class="grid gap-4 md:grid-cols-2">
                <Field label="Name" required=true>
                    <TextInput
                        value=display_name
                        on_input=Callback::new(move |v| display_name.set(v))
                        placeholder="Your name"
                    />
                </Field>
                <Field label="Email" required=true>
                    <TextInput
                        value=email
                        on_input=Callback::new(move |v| email.set(v))
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
                        value=discord
                        on_input=Callback::new(move |v| discord.set(v))
                        placeholder="username"
                    />
                </Field>
                <Field label="Timezone" required=true hint="So we know when you're reachable.">
                    <TextInput
                        value=timezone
                        on_input=Callback::new(move |v| timezone.set(v))
                        placeholder="America/Denver"
                    />
                </Field>
            </div>

            <Field
                label="Why should we choose you?"
                required=true
                hint="Be specific. What you fly, how you report a bug, what you'd stress-test on a DC-10."
                class="mt-6"
            >
                <TextArea
                    value=why_you
                    on_input=Callback::new(move |v| why_you.set(v))
                    rows=7
                    placeholder="I've flown the DC-10 in every sim that's had one and I know the systems well enough to tell you which one is misbehaving. I file reports with steps, expected vs actual, and a log…"
                />
            </Field>
            <div class="mt-1 text-right font-mono text-[11px] tabular-nums text-white/35">
                {move || format!("{} characters", why_you.get().trim().chars().count())}
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
    display_name: RwSignal<String>,
    email: RwSignal<String>,
    discord: RwSignal<String>,
    timezone: RwSignal<String>,
    why_you: RwSignal<String>,
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
                <ReviewRow label="Name" value=Signal::derive(move || display_name.get()) />
                <ReviewRow label="Email" value=Signal::derive(move || email.get()) />
                <ReviewRow label="Discord" value=Signal::derive(move || discord.get()) />
                <ReviewRow label="Timezone" value=Signal::derive(move || timezone.get()) />
                <ReviewRow
                    label="Questionnaire"
                    value=Signal::derive(move || {
                        format!("{} of {} answered", answered.get(), total.get())
                    })
                />
                <div class=SPEC_ROW>
                    <dt class=SPEC_LABEL>"Why you"</dt>
                    <dd class="max-h-56 overflow-y-auto text-sm leading-6 font-light whitespace-pre-wrap text-white/75">
                        {move || why_you.get()}
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

    view! {
        <section class="mb-20">
            <SectionLabel num="01" label="Your application" />
            <p class=cn!(STATUS, if complete { "text-amber-300" } else { "text-accent-300" })>
                {if complete { "Final check" } else { "Accepted" }}
            </p>
            <h2 class="max-w-3xl text-3xl font-extralight tracking-tight md:text-4xl">
                {if complete {
                    "Paperwork received — final check"
                } else {
                    "Your application was accepted. Two documents to sign."
                }}
            </h2>
            <p class="mt-6 max-w-xl border-l border-accent-400/40 pl-5 text-sm leading-relaxed font-light text-white/65">
                {if complete {
                    "Both documents are signed and with us. We check them over and then release your build — nothing else for you to do."
                } else {
                    "Read and e-sign both below. Once they're in we do a final check and then you're a tester."
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
                "Read them properly — the NDA is what stops the aircraft leaking before release, and it binds you. If anything in either document doesn't work for you, tell us before signing rather than after."
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
                <SectionLabel num="02" label="Your signed documents" />
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
