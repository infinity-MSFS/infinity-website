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
use crate::nav;

const MIN_WHY_YOU: usize = 40;

const STEPS: [(&str, IconData); 4] = [
    ("Systems check", icons::GAUGE),
    ("In your words", icons::MESSAGE_CIRCLE),
    ("Your details", icons::USER),
    ("Review", icons::CIRCLE_CHECK),
];

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
        <AnimatedPage class="relative z-10 min-h-screen px-4 py-12 text-white md:px-8">
            <div class="mx-auto max-w-4xl">
                <Header />

                {move || {
                    state_error
                        .get()
                        .map(|message| {
                            view! {
                                <Alert
                                    tone=Tone::Danger
                                    title="Couldn't load your beta status"
                                    description=message
                                    class="mb-8"
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
                            return view! { <LoadingPanel /> }.into_any();
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
                                                    <Alert
                                                        tone=Tone::Danger
                                                        title="Couldn't load the questionnaire"
                                                        description=message
                                                        class="mb-6"
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
        <div class="mb-8">
            <div class="flex flex-wrap items-center justify-between gap-3">
                <Badge tone=Tone::Info icon=icons::PLANE size=Size::Lg>
                    "Closed beta · Aerodynamics DC-10"
                </Badge>
                // Staff shortcut into the review queue. The route gates itself
                // and the worker gates the data; this only saves typing a URL.
                <Show when=move || auth.entitlements.get().is_developer()>
                    <Button
                        variant=Variant::Glass
                        size=Size::Sm
                        icon=icons::USERS
                        on_click=nav::go_to(go, nav::BETA_ADMIN)
                    >
                        "Review applications"
                    </Button>
                </Show>
            </div>
            <h1 class="mt-4 mb-3 text-4xl font-bold">"Fly the DC-10 before anyone else"</h1>
            <p class="max-w-2xl leading-relaxed text-white/70">
                "We're picking a small group who know the aircraft and can report a bug properly. Start with a short systems questionnaire, then tell us who you are and why you."
            </p>
        </div>
    }
}

#[component]
fn LoadingPanel() -> impl IntoView {
    view! {
        <Panel class="space-y-3 p-6">
            <Skeleton class="h-5 w-48" />
            <Skeleton class="h-4 w-full" />
            <Skeleton class="h-4 w-2/3" />
        </Panel>
    }
}

#[component]
fn SignedOut() -> impl IntoView {
    let auth = use_auth();
    view! {
        <Panel class="p-8 text-center">
            <div class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl border border-white/10 bg-white/5">
                <Icon icon=icons::LOG_IN class="h-6 w-6 text-accent-400" />
            </div>
            <h2 class="mb-2 text-xl font-bold">"Sign in to apply"</h2>
            <p class="mx-auto mb-6 max-w-md text-sm leading-relaxed text-white/60">
                "Applications are tied to your account so we can track your status and, if you're selected, hand you the paperwork on this same page."
            </p>
            <Button
                variant=Variant::Solid
                icon=icons::LOG_IN
                on_click=Callback::new(move |()| auth.login())
            >
                "Sign in"
            </Button>
        </Panel>
    }
}

#[component]
fn Steps(#[prop(into)] current: Signal<usize>) -> impl IntoView {
    view! {
        <div class="mb-8 flex flex-col gap-3 sm:flex-row sm:items-center">
            {STEPS
                .iter()
                .enumerate()
                .map(|(index, (label, icon))| {
                    let done = Signal::derive(move || current.get() > index);
                    let active = Signal::derive(move || current.get() == index);
                    view! {
                        <div class="flex flex-1 items-center gap-3">
                            <div class=move || {
                                cn!(
                                    "flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border transition-colors",
                                    if active.get() {
                                        "border-accent-400/40 bg-accent-500/15 text-accent-300"
                                    } else if done.get() {
                                        "border-emerald-400/30 bg-emerald-500/10 text-emerald-300"
                                    } else {
                                        "border-white/10 bg-white/5 text-white/40"
                                    },
                                )
                            }>
                                {move || {
                                    let glyph = if done.get() { icons::CHECK } else { *icon };
                                    view! { <Icon icon=glyph class="h-4 w-4" /> }
                                }}
                            </div>
                            <div class="min-w-0">
                                <div class="text-[11px] uppercase tracking-[0.16em] text-white/40">
                                    {format!("Step {}", index + 1)}
                                </div>
                                <div class=move || {
                                    cn!(
                                        "truncate text-sm font-semibold",
                                        if active.get() { "text-white" } else { "text-white/50" },
                                    )
                                }>{*label}</div>
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
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
        <Callout tone=Tone::Info icon=icons::GAUGE title="Systems check" class="mb-6">
            "Ten questions on how the DC-10 actually works. We're not looking for a perfect score — we're looking for someone who can tell a hydraulic problem from an electrical one when they file a report. Your score goes to us with the application; it doesn't block you from submitting."
        </Callout>

        <Panel class="mb-6 flex items-center gap-4 p-4">
            <Progress
                value=Signal::derive(move || {
                    let total = total.get();
                    if total == 0 { 0.0 } else { answered.get() as f64 / total as f64 }
                })
                tone=Tone::Accent
                class="flex-1"
            />
            <div class="shrink-0 text-sm tabular-nums text-white/60">
                {move || format!("{} / {} answered", answered.get(), total.get())}
            </div>
        </Panel>

        <Show
            when=move || !questions.get().is_empty()
            fallback=|| view! { <LoadingPanel /> }
        >
            <div class="space-y-4">
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
                                <Panel
                                    highlighted=Signal::derive(move || !selected.get().is_empty())
                                    class="p-5"
                                >
                                    <div class="mb-1 flex items-center gap-2 text-[11px] uppercase tracking-[0.16em] text-accent-300/80">
                                        <span>{format!("{:02}", index + 1)}</span>
                                        <span class="text-white/30">"·"</span>
                                        <span>{question.topic.clone()}</span>
                                    </div>
                                    <div class="mb-4 text-base font-semibold text-white">
                                        {question.prompt.clone()}
                                    </div>
                                    <div class="space-y-2">
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
                                                        class="w-full rounded-xl border border-white/[0.06] bg-white/[0.02] px-3 py-2.5 leading-6 hover:border-white/15"
                                                    />
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </Panel>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </Show>

        <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:justify-end">
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
        <Callout
            tone=Tone::Info
            icon=icons::MESSAGE_CIRCLE
            title="In your own words"
            class="mb-6"
        >
            "This is the part we actually read. Short and specific beats long and general — we are trying to work out whether you would be useful in a bug thread at 1am, not whether you can write."
        </Callout>

        <div class="space-y-4">
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
                            <Panel class="p-5">
                                <div class="mb-1 flex items-center gap-2 text-[11px] uppercase tracking-[0.16em] text-accent-300/80">
                                    <span>{format!("{:02}", index + 1)}</span>
                                    {(!required)
                                        .then(|| {
                                            view! {
                                                <span class="text-white/30">"· optional"</span>
                                            }
                                        })}
                                </div>
                                <div class="mb-3 text-base font-semibold text-white">
                                    {question.prompt.clone()}
                                </div>
                                <TextArea
                                    value=value
                                    on_input=on_input
                                    rows=5
                                    placeholder=question.hint.clone().unwrap_or_default()
                                />
                                <div class="mt-1.5 flex items-center justify-between gap-3 text-xs">
                                    <span class="text-white/40">
                                        {question.hint.clone().unwrap_or_default()}
                                    </span>
                                    <span class=move || {
                                        let length = value.get().trim().chars().count();
                                        cn!(
                                            "shrink-0 tabular-nums",
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
                            </Panel>
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
                        <Alert
                            tone=Tone::Warning
                            title="Not quite there"
                            description=message
                            class="mt-4"
                        />
                    }
                })
        }}

        <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:justify-between">
            <Button variant=Variant::Glass icon=icons::ARROW_LEFT on_click=on_back>
                "Back to the questions"
            </Button>
            <Button
                variant=Variant::Solid
                icon=icons::ARROW_RIGHT
                disabled=Signal::derive(move || !ready.get())
                on_click=on_next
            >
                "Continue to your details"
            </Button>
        </div>
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
        <Panel class="p-6">
            <div class="mb-5 flex items-center gap-3">
                <Icon icon=icons::USER class="h-5 w-5 text-accent-400" />
                <h2 class="text-xl font-bold">"How we reach you"</h2>
            </div>

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
            <div class="mt-1 text-right text-xs tabular-nums text-white/35">
                {move || format!("{} characters", why_you.get().trim().chars().count())}
            </div>
        </Panel>

        {move || {
            problem
                .get()
                .map(|message| {
                    view! {
                        <Alert
                            tone=Tone::Warning
                            title="Not quite there"
                            description=message
                            class="mt-4"
                        />
                    }
                })
        }}

        <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:justify-between">
            <Button variant=Variant::Glass icon=icons::ARROW_LEFT on_click=on_back>
                "Back to the questions"
            </Button>
            <Button
                variant=Variant::Solid
                icon=icons::ARROW_RIGHT
                disabled=Signal::derive(move || !ready.get())
                on_click=on_next
            >
                "Review"
            </Button>
        </div>
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
        <Panel class="p-6">
            <div class="mb-5 flex items-center gap-3">
                <Icon icon=icons::CIRCLE_CHECK class="h-5 w-5 text-accent-400" />
                <h2 class="text-xl font-bold">"Check it over"</h2>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
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
            </div>

            <div class="mt-5">
                <div class="mb-1.5 text-xs font-medium uppercase tracking-[0.14em] text-white/50">
                    "Why you"
                </div>
                <Well class="max-h-56 overflow-y-auto p-4 text-sm leading-6 whitespace-pre-wrap text-white/75">
                    {move || why_you.get()}
                </Well>
            </div>

            <Callout tone=Tone::Neutral icon=icons::SHIELD title="What happens next" class="mt-5">
                "We grade the questionnaire and read every application. If you're selected you'll hear from us on Discord or email, and this page will switch over to the NDA and beta agreement for you to sign."
            </Callout>
        </Panel>

        {move || {
            failure
                .get()
                .map(|message| {
                    view! {
                        <Alert
                            tone=Tone::Danger
                            title="Not submitted"
                            description=message
                            class="mt-4"
                        />
                    }
                })
        }}

        <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:justify-between">
            <Button
                variant=Variant::Glass
                icon=icons::ARROW_LEFT
                disabled=sending
                on_click=on_back
            >
                "Back"
            </Button>
            <Button
                variant=Variant::Solid
                icon=icons::CIRCLE_CHECK
                loading=sending
                on_click=on_submit
            >
                "Submit application"
            </Button>
        </div>

        <p class="mt-4 text-xs leading-relaxed text-white/40">
            "Applying doesn't guarantee a place. We're keeping the group small so the feedback loop stays tight."
        </p>
    }
}

#[component]
fn ReviewRow(#[prop(into)] label: String, #[prop(into)] value: Signal<String>) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-white/[0.06] bg-white/[0.02] px-4 py-3">
            <div class="text-[11px] uppercase tracking-[0.14em] text-white/40">{label}</div>
            <div class="mt-0.5 truncate text-sm text-white/85">{move || value.get()}</div>
        </div>
    }
}

#[component]
fn UnderReview(state: BetaState) -> impl IntoView {
    let reference = state.application_id.clone();
    let score = state.score;
    view! {
        <Panel class="p-8">
            <div class="mb-4 flex items-center gap-3">
                <div class="flex h-11 w-11 items-center justify-center rounded-2xl border border-sky-400/30 bg-sky-500/10">
                    <Icon icon=icons::CLOCK class="h-5 w-5 text-sky-300" />
                </div>
                <div>
                    <h2 class="text-xl font-bold">"Application received"</h2>
                    <div class="text-sm text-white/50">"We're reading it."</div>
                </div>
            </div>

            <p class="mb-5 max-w-xl text-sm leading-relaxed text-white/70">
                "If you're selected we'll reach out on Discord or by email, and this page will switch over to the NDA and beta agreement for you to sign. Nothing else to do for now."
            </p>

            <div class="grid gap-3 sm:grid-cols-2">
                {reference
                    .map(|id| {
                        view! { <ReviewRow label="Reference" value=Signal::derive(move || id.clone()) /> }
                    })}
                {score
                    .map(|score| {
                        view! { <ScoreRow score=score /> }
                    })}
            </div>
        </Panel>
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
        <Panel class="p-8">
            <div class="mb-4 flex items-center gap-3">
                <div class="flex h-11 w-11 items-center justify-center rounded-2xl border border-white/10 bg-white/5">
                    <Icon icon=icons::CIRCLE_X class="h-5 w-5 text-white/50" />
                </div>
                <h2 class="text-xl font-bold">"Not selected this time"</h2>
            </div>
            <p class="max-w-xl text-sm leading-relaxed text-white/70">
                "We had more good applications than places. This isn't a judgement on you, and it doesn't rule you out of a later test round."
            </p>
            {note
                .map(|note| {
                    view! {
                        <Well class="mt-5 p-4 text-sm leading-6 whitespace-pre-wrap text-white/70">
                            {note}
                        </Well>
                    }
                })}
        </Panel>
    }
}

/// Stage two: the tester was accepted and now has documents waiting.
#[component]
fn Paperwork(state: BetaState, go: Callback<String>) -> impl IntoView {
    let complete = state.documents_complete;
    let documents = state.documents.clone();

    view! {
        <div class="mb-6">
            <Alert
                tone=Tone::Info
                title=if complete {
                    "Paperwork received — final check"
                } else {
                    "Your application was accepted. Two documents to sign."
                }
                description=if complete {
                    "Both documents are signed and with us. We check them over and then release your build — nothing else for you to do."
                        .to_string()
                } else {
                    "Read and e-sign both below. Once they're in we do a final check and then you're a tester."
                        .to_string()
                }
            />
        </div>

        <div class="space-y-4">
            {documents
                .into_iter()
                .map(|doc| {
                    view! { <DocumentCard doc=doc go=go /> }
                })
                .collect_view()}
        </div>

        <Callout tone=Tone::Neutral icon=icons::SHIELD title="Before you sign" class="mt-6">
            "Read them properly — the NDA is what stops the aircraft leaking before release, and it binds you. If anything in either document doesn't work for you, tell us before signing rather than after."
        </Callout>
    }
}

/// Past the second gate: paperwork checked, role granted, build released.
#[component]
fn Approved(state: BetaState, go: Callback<String>) -> impl IntoView {
    // Read twice: once to decide whether the section renders, once to fill it.
    let documents = StoredValue::new(state.documents.clone());

    view! {
        <Panel class="mb-6 p-8">
            <div class="mb-4 flex items-center gap-3">
                <div class="flex h-12 w-12 items-center justify-center rounded-2xl border border-emerald-400/30 bg-emerald-500/10">
                    <Icon icon=icons::CIRCLE_CHECK class="h-6 w-6 text-emerald-300" />
                </div>
                <div>
                    <h2 class="text-2xl font-bold">"You're a DC-10 beta tester"</h2>
                    <div class="text-sm text-white/50">
                        "Paperwork checked and your account is entitled."
                    </div>
                </div>
            </div>

            <p class="max-w-xl text-sm leading-relaxed text-white/70">
                "The build shows up in the Infinity Manager under the beta channel. If it is not there yet, sign out and back in once — your entitlement is baked into the token."
            </p>

            <Callout tone=Tone::Warning icon=icons::SHIELD title="The NDA is live" class="mt-5">
                "No screenshots, no video, no streaming, nothing outside the tester channels. That holds until we say otherwise in writing."
            </Callout>
        </Panel>

        <Show when=move || !documents.with_value(Vec::is_empty)>
            <div class="mb-3 text-xs font-medium uppercase tracking-[0.14em] text-white/40">
                "Your signed documents"
            </div>
            <div class="space-y-4">
                {documents
                    .get_value()
                    .into_iter()
                    .map(|doc| view! { <DocumentCard doc=doc go=go /> })
                    .collect_view()}
            </div>
        </Show>
    }
}

#[component]
fn DocumentCard(doc: BetaDocumentState, go: Callback<String>) -> impl IntoView {
    let signed = doc.signed;
    let target = nav::beta_sign(&doc.id);
    let superseded = doc.superseded_version.clone();

    view! {
        <Panel class="flex flex-col gap-4 p-5 sm:flex-row sm:items-center">
            <div class=cn!(
                "flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border",
                if signed {
                    "border-emerald-400/30 bg-emerald-500/10 text-emerald-300"
                } else {
                    "border-white/10 bg-white/5 text-accent-400"
                },
            )>
                <Icon
                    icon=if signed { icons::CIRCLE_CHECK } else { icons::FILE_TEXT }
                    class="h-5 w-5"
                />
            </div>

            <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                    <span class="font-semibold text-white">{doc.title.clone()}</span>
                    <Badge tone=Tone::Neutral size=Size::Xs>{format!("v{}", doc.version)}</Badge>
                    {signed
                        .then(|| {
                            view! {
                                <Badge tone=Tone::Accent size=Size::Xs icon=icons::CHECK>
                                    "Signed"
                                </Badge>
                            }
                        })}
                </div>
                <p class="mt-1 text-sm leading-relaxed text-white/60">{doc.summary.clone()}</p>
                {doc
                    .signed_at
                    .clone()
                    .map(|at| {
                        view! {
                            <div class="mt-1 text-xs text-white/40">{format!("Signed {at}")}</div>
                        }
                    })}
                {superseded
                    .map(|old| {
                        view! {
                            <div class="mt-1 text-xs text-amber-300/80">
                                {format!(
                                    "You signed v{old}. This document has been updated — please sign the current version.",
                                )}
                            </div>
                        }
                    })}
            </div>

            <Button
                variant=if signed { Variant::Glass } else { Variant::Solid }
                icon=if signed { icons::EYE } else { icons::PENCIL }
                on_click=nav::go_to(go, target)
                class="shrink-0"
            >
                {if signed { "View" } else { "Read and sign" }}
            </Button>
        </Panel>
    }
}
