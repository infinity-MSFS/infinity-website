use infinity_ui::prelude::*;
use leptos::task::spawn_local;

use super::legal::LINK;
use crate::api::{self, AircraftProduct, MIN_PASSWORD_LENGTH, SupportTicket};
use crate::auth::{User, use_auth};
use crate::components::{DownloadButton, EYEBROW, HoverRule, SectionLabel};
use crate::nav;
use crate::state::{use_catalog, use_manager};

/// Row layout shared by the support table.
const ROW: &str = "grid gap-2 border-t border-white/10 py-5 md:grid-cols-[14rem_1fr_auto] md:items-center md:gap-8";

#[component]
pub fn UserDashboard() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Show
            when=move || !auth.loading.get()
            fallback=|| {
                view! {
                    <div class="flex min-h-screen items-center justify-center">
                        <div class=cn!("flex items-center gap-3", EYEBROW)>
                            <Spinner size=Size::Sm />
                            "Loading your account…"
                        </div>
                    </div>
                }
            }
        >
            <Show when=move || auth.is_authenticated.get() fallback=|| view! { <LoginRequired /> }>
                <Dashboard />
            </Show>
        </Show>
    }
}

#[component]
fn LoginRequired() -> impl IntoView {
    let auth = use_auth();
    view! {
        <AnimatedPage class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
            <p class=cn!("mb-6", EYEBROW)>"Infinity / Account"</p>
            <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                "Sign in "
                <span class="font-semibold text-accent-300">"required."</span>
            </h1>
            <p class="mb-8 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                "Sign in to view your account dashboard."
            </p>
            <button
                type="button"
                on:click=move |_| auth.login()
                class="group inline-flex items-center gap-2 self-start text-lg font-medium transition-colors hover:text-accent-300"
            >
                "Sign in"
                <Icon
                    icon=icons::ARROW_RIGHT
                    class="h-4 w-4 transition-transform group-hover:translate-x-1"
                />
            </button>
        </AnimatedPage>
    }
}

#[component]
fn Dashboard() -> impl IntoView {
    let go = nav::use_go();
    let auth = use_auth();
    let catalog = use_catalog();
    let support_open = RwSignal::new(false);

    let owned: Signal<Vec<AircraftProduct>> = Signal::derive(move || {
        let granted = auth.entitlements.get();
        catalog.0.value.with(|products| {
            products
                .as_deref()
                .unwrap_or_default()
                .iter()
                .filter(|product| granted.owns_any(&product.entitlements()))
                .cloned()
                .collect()
        })
    });

    let unmatched: Signal<Vec<String>> = Signal::derive(move || {
        let claimed: Vec<String> = owned
            .get()
            .iter()
            .flat_map(AircraftProduct::entitlements)
            .collect();
        auth.entitlements
            .get()
            .products
            .into_iter()
            .filter(|id| !claimed.contains(id))
            .collect()
    });

    let total = Signal::derive(move || owned.get().len() + unmatched.get().len());
    let has_products = Signal::derive(move || total.get() > 0);

    view! {
        <AnimatedPage class="min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <ProfileHeader />

                <section class="mb-20">
                    <SectionLabel num="01" label="Your products" />
                    <p class="-mt-4 mb-8 font-mono text-[11px] text-white/40">
                        {move || format!("{:02} owned", total.get())}
                    </p>

                    <Show
                        when=move || has_products.get()
                        fallback=move || {
                            view! {
                                <div class="border-t border-white/10 py-8">
                                    <p class="mb-5 max-w-xl font-light text-white/60">
                                        "No products yet. Browse our aircraft to get started — purchases land on your account automatically."
                                    </p>
                                    <button
                                        type="button"
                                        on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                                        class="group inline-flex items-center gap-2 text-lg font-medium transition-colors hover:text-accent-300"
                                    >
                                        "Browse aircraft"
                                        <Icon
                                            icon=icons::ARROW_RIGHT
                                            class="h-4 w-4 transition-transform group-hover:translate-x-1"
                                        />
                                    </button>
                                </div>
                            }
                        }
                    >
                        <div class="grid gap-x-10 gap-y-12 sm:grid-cols-2">
                            {move || {
                                owned
                                    .get()
                                    .into_iter()
                                    .map(|product| view! { <OwnedProduct product=product /> })
                                    .collect_view()
                            }}
                            {move || {
                                unmatched
                                    .get()
                                    .into_iter()
                                    .map(|id| view! { <UnmatchedEntitlement id=id /> })
                                    .collect_view()
                            }}
                        </div>
                    </Show>

                    <p class="mt-8 text-xs text-white/40">
                        "Install and update your aircraft from inside the Infinity Manager below. Need a refund, or something not working? See the "
                        <a href=nav::LEGAL_REFUNDS class=LINK>
                            "refund policy"
                        </a>
                        "."
                    </p>
                </section>

                <ManagerSection />
                <SecuritySection />

                <section>
                    <SectionLabel num="04" label="Support" />
                    <div class="border-b border-white/10">
                        <a
                            href=nav::DISCORD_URL
                            target="_blank"
                            rel="noopener noreferrer"
                            class=cn!("group relative transition-colors hover:bg-white/[0.03]", ROW)
                        >
                            <HoverRule />
                            <h3 class="text-base font-medium">"Discord"</h3>
                            <p class="text-sm font-light text-white/55">
                                "The fastest way to get help from the team and community."
                            </p>
                            <RowAction label="Join Discord" icon=icons::ARROW_UP_RIGHT />
                        </a>
                        <button
                            type="button"
                            on:click=move |_| support_open.set(true)
                            class=cn!("group relative text-left transition-colors hover:bg-white/[0.03]", ROW)
                        >
                            <HoverRule />
                            <h3 class="text-base font-medium">"Email support"</h3>
                            <p class="text-sm font-light text-white/55">
                                "Open a ticket if you would rather talk over email."
                            </p>
                            <RowAction label="Open a ticket" icon=icons::ARROW_RIGHT />
                        </button>
                        <a
                            href=nav::LEGAL_REFUNDS
                            class=cn!("group relative transition-colors hover:bg-white/[0.03]", ROW)
                        >
                            <HoverRule />
                            <h3 class="text-base font-medium">"Refund policy"</h3>
                            <p class="text-sm font-light text-white/55">
                                "When and how you can get your money back."
                            </p>
                            <RowAction label="Read the policy" icon=icons::ARROW_RIGHT />
                        </a>
                    </div>
                </section>
            </div>

            <SupportModal open=support_open />
        </AnimatedPage>
    }
}

#[component]
fn RowAction(label: &'static str, icon: IconData) -> impl IntoView {
    view! {
        <span class="inline-flex items-center gap-2 text-sm font-medium">
            {label}
            <Icon icon=icon class="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" />
        </span>
    }
}

#[component]
fn ProfileHeader() -> impl IntoView {
    let auth = use_auth();
    let user = Signal::derive(move || auth.user.get().unwrap_or_default());

    view! {
        <header class="mb-16">
            <p class=cn!("mb-10", EYEBROW)>"Infinity / Account"</p>
            <div class="flex flex-col gap-6 md:flex-row md:items-end">
                {move || {
                    let user = user.get();
                    view! {
                        <Avatar
                            name=user.display_name()
                            src=user.picture.clone()
                            size=Size::Lg
                            class="h-20 w-20 border border-white/15 text-2xl"
                        />
                    }
                }}
                <div class="min-w-0 flex-1">
                    <h1 class="truncate text-4xl leading-[1.05] font-extralight tracking-tight md:text-6xl">
                        {move || user.get().display_name()}
                    </h1>
                    <div class="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[11px]">
                        <span class="truncate text-white/50">
                            {move || user.get().email.unwrap_or_default()}
                        </span>
                        <Show when=move || auth.entitlements.get().is_developer()>
                            <span class="tracking-[0.2em] text-amber-300 uppercase">"Developer"</span>
                        </Show>
                        {move || {
                            auth.entitlements
                                .get()
                                .channels
                                .into_iter()
                                .filter(|channel| channel != "stable")
                                .map(|channel| {
                                    view! {
                                        <span class="tracking-[0.2em] text-accent-300 uppercase">
                                            {channel}
                                        </span>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
                <button
                    type="button"
                    on:click=move |_| auth.logout()
                    class="inline-flex items-center gap-2 self-start text-sm text-white/55 transition-colors hover:text-white md:self-end"
                >
                    <Icon icon=icons::LOG_OUT class="h-4 w-4" />
                    "Sign out"
                </button>
            </div>
        </header>
    }
}

/// Password change - handled remotely, we are just passing it to worker
#[component]
fn SecuritySection() -> impl IntoView {
    let auth = use_auth();
    let current = RwSignal::new(String::new());
    let next = RwSignal::new(String::new());
    let confirm = RwSignal::new(String::new());
    let reveal = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let done = RwSignal::new(false);

    let user = Signal::derive(move || auth.user.get().unwrap_or_default());
    let database_account = Signal::derive(move || user.get().is_database_user());
    let provider = Signal::derive(move || {
        api::provider_label(user.get().sub.as_deref()).unwrap_or("an external provider")
    });
    let field_type =
        Signal::derive(move || if reveal.get() { "text" } else { "password" }.to_string());
    let incomplete = Signal::derive(move || {
        current.get().is_empty() || next.get().is_empty() || confirm.get().is_empty()
    });

    let submit = move |()| {
        error.set(None);
        done.set(false);

        // Cheap checks first: the worker meters attempts per day, so an
        // obvious typo should never cost one.
        if next.get_untracked() != confirm.get_untracked() {
            error.set(Some("New passwords do not match.".into()));
            return;
        }
        if next.get_untracked().chars().count() < MIN_PASSWORD_LENGTH {
            error.set(Some(format!(
                "New password must be at least {MIN_PASSWORD_LENGTH} characters."
            )));
            return;
        }
        if next.get_untracked() == current.get_untracked() {
            error.set(Some(
                "New password must be different from your current one.".into(),
            ));
            return;
        }

        saving.set(true);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                saving.set(false);
                error.set(Some("Your session expired. Sign in again.".into()));
                return;
            };
            let result =
                api::change_password(&token, &current.get_untracked(), &next.get_untracked()).await;
            saving.set(false);
            match result {
                Ok(()) => {
                    done.set(true);
                    current.set(String::new());
                    next.set(String::new());
                    confirm.set(String::new());
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="mb-20">
            <SectionLabel num="03" label="Password" />
            <Show
                when=move || database_account.get()
                fallback=move || {
                    view! {
                        <p class="max-w-2xl border-l border-accent-400/40 pl-5 text-sm leading-relaxed font-light text-white/60">
                            {move || {
                                format!(
                                    "You sign in with {}, so your password is managed there. Change it from your {} account settings.",
                                    provider.get(),
                                    provider.get(),
                                )
                            }}
                        </p>
                    }
                }
            >
                <div class="flex items-start justify-between gap-4">
                    <p class="text-sm font-light text-white/60">
                        "Enter your current password, then choose a new one."
                    </p>
                    {move || {
                        let (glyph, label) = if reveal.get() {
                            (icons::EYE_OFF, "Hide")
                        } else {
                            (icons::EYE, "Show")
                        };
                        view! {
                            <Button
                                variant=Variant::Ghost
                                size=Size::Xs
                                icon=glyph
                                on_click=Callback::new(move |()| {
                                    reveal.update(|v| *v = !*v)
                                })
                            >
                                {label}
                            </Button>
                        }
                    }}
                </div>

                <Show when=move || done.get()>
                    <Alert
                        tone=Tone::Accent
                        title="Password changed"
                        description="You are still signed in here. Use the new password next time you sign in, including in the Infinity Manager."
                        class="mt-4"
                    />
                </Show>
                {move || {
                    error
                        .get()
                        .map(|message| {
                            view! {
                                <Alert
                                    tone=Tone::Danger
                                    title="Not changed"
                                    description=message
                                    class="mt-4"
                                />
                            }
                        })
                }}

                <div class="mt-6 grid gap-4 sm:grid-cols-3">
                    <Field label="Current password">
                        <TextInput
                            value=current
                            on_input=Callback::new(move |v| current.set(v))
                            input_type=field_type
                            disabled=saving
                        />
                    </Field>
                    <Field label="New password">
                        <TextInput
                            value=next
                            on_input=Callback::new(move |v| next.set(v))
                            input_type=field_type
                            disabled=saving
                        />
                    </Field>
                    <Field label="Confirm new password">
                        <TextInput
                            value=confirm
                            on_input=Callback::new(move |v| confirm.set(v))
                            input_type=field_type
                            disabled=saving
                        />
                    </Field>
                    <div class="flex justify-end sm:col-span-3">
                        <Button
                            variant=Variant::Solid
                            loading=saving
                            disabled=incomplete
                            on_click=Callback::new(submit)
                        >
                            "Change password"
                        </Button>
                    </div>
                </div>
            </Show>
        </section>
    }
}

#[component]
fn ManagerSection() -> impl IntoView {
    let manager = use_manager();
    let version = manager.version();
    let platforms = manager.platforms();
    let loading = manager.0.loading();
    let error = manager.0.error;

    view! {
        <section class="mb-20">
            <SectionLabel num="02" label="Infinity Manager" />
            <Show
                when=move || error.with(Option::is_none)
                fallback=move || {
                    view! {
                        <p class="border-l border-red-400/60 pl-4 text-sm text-red-200">
                            "Couldn't load the latest Infinity Manager release. "
                            <span class="text-red-200/70">{move || error.get().unwrap_or_default()}</span>
                        </p>
                    }
                }
            >
                <div class="flex flex-col gap-6 md:flex-row md:items-end md:justify-between">
                    <div class="max-w-xl">
                        <p class="text-sm leading-relaxed font-light text-white/60">
                            "Install and update your aircraft with the latest manager. Already installed? Updates download automatically."
                        </p>
                        <p class="mt-3 font-mono text-[11px] text-white/40">
                            {move || {
                                version
                                    .get()
                                    .map(|v| {
                                        view! {
                                            "Latest release  /  "
                                            <span class="text-accent-300">{format!("v{v}")}</span>
                                            "  /  "
                                        }
                                    })
                            }}
                            "Auto-updating"
                        </p>
                    </div>
                    <DownloadButton platforms=platforms loading=loading class="shrink-0" />
                </div>
            </Show>
        </section>
    }
}

#[component]
fn OwnedProduct(product: AircraftProduct) -> impl IntoView {
    let go = nav::use_go();
    let href = format!("{}/{}", nav::AIRCRAFT, product.slug);
    let openable = product.storefront;

    view! {
        <button
            type="button"
            disabled=!openable
            on:click=move |_| {
                if openable {
                    go.run(href.clone());
                }
            }
            class="group flex flex-col text-left disabled:cursor-default"
        >
            <div class="relative aspect-video overflow-hidden rounded-sm border border-white/10 bg-white/5">
                <img
                    src=product.card_image.clone()
                    alt=product.name.clone()
                    class="h-full w-full object-cover transition-transform duration-700 group-hover:scale-[1.03]"
                />
                {product
                    .dev_only
                    .then(|| {
                        view! {
                            <span class="absolute top-3 right-3">
                                <Badge tone=Tone::Dev icon=icons::FLASK_CONICAL squared=true>
                                    "Dev"
                                </Badge>
                            </span>
                        }
                    })}
            </div>
            <div class="relative mt-4 flex flex-1 items-end justify-between gap-3 border-t border-white/10 pt-4">
                {openable.then(|| view! { <HoverRule /> })}
                <div class="min-w-0">
                    <p class="mb-2 font-mono text-[11px] tracking-[0.2em] text-accent-400/80 uppercase">
                        "Owned"
                    </p>
                    <h3 class="truncate text-xl font-medium tracking-tight">{product.name.clone()}</h3>
                    <p class="truncate text-sm font-light text-white/45">{product.developer.clone()}</p>
                </div>
                {openable
                    .then(|| {
                        view! {
                            <Icon
                                icon=icons::ARROW_RIGHT
                                class="h-4 w-4 shrink-0 transition-transform group-hover:translate-x-1"
                            />
                        }
                    })}
            </div>
        </button>
    }
}

#[component]
fn UnmatchedEntitlement(id: String) -> impl IntoView {
    let title = id
        .split(['-', '_', ':'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            if part.chars().any(|c| c.is_ascii_digit()) {
                part.to_uppercase()
            } else {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <div class="flex flex-col">
            <div class="flex aspect-video items-center justify-center rounded-sm border border-dashed border-white/15">
                <span class=EYEBROW>"No preview"</span>
            </div>
            <div class="mt-4 border-t border-white/10 pt-4">
                <p class="mb-2 font-mono text-[11px] tracking-[0.2em] text-accent-400/80 uppercase">
                    "Owned"
                </p>
                <h3 class="text-xl font-medium tracking-tight">{title}</h3>
                <p class="font-mono text-[11px] text-white/40">{id}</p>
            </div>
        </div>
    }
}

#[component]
fn SupportModal(open: RwSignal<bool>) -> impl IntoView {
    let auth = use_auth();
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let discord = RwSignal::new(String::new());
    let subject = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let sending = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let sent = RwSignal::new(false);

    Effect::new(move |_| {
        let Some(user) = auth.user.get() else {
            return;
        };
        if email.get_untracked().is_empty()
            && let Some(address) = user.email.clone()
        {
            email.set(address);
        }
        if name.get_untracked().is_empty()
            && let Some(display) = user.name.clone()
        {
            name.set(display);
        }
    });

    let incomplete =
        Signal::derive(move || subject.get().trim().is_empty() || message.get().trim().is_empty());

    let send = move |()| {
        error.set(None);
        sending.set(true);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                sending.set(false);
                error.set(Some("Your session expired. Sign in again.".into()));
                return;
            };
            let account_email = auth
                .user
                .get_untracked()
                .and_then(|user: User| user.email)
                .unwrap_or_default();
            let ticket = SupportTicket {
                name: name.get_untracked(),
                email: email.get_untracked(),
                discord_name: discord.get_untracked(),
                subject: subject.get_untracked(),
                message: message.get_untracked(),
                user_email: account_email,
            };
            let result = api::submit_support_ticket(&token, &ticket).await;
            sending.set(false);
            match result {
                Ok(()) => {
                    sent.set(true);
                    subject.set(String::new());
                    message.set(String::new());
                    set_timeout(
                        move || {
                            open.set(false);
                            sent.set(false);
                        },
                        std::time::Duration::from_millis(1500),
                    );
                }
                Err(err) => error.set(Some(err)),
            }
        });
    };

    view! {
        <Modal
            open=open
            on_close=Callback::new(move |()| open.set(false))
            title="Email support"
            description="We reply to the address on your account."
            icon=icons::MAIL
            width="max-w-md"
            footer=ViewFn::from(move || {
                view! {
                    <ButtonGroup>
                        <Button
                            variant=Variant::Ghost
                            on_click=Callback::new(move |()| open.set(false))
                        >
                            "Cancel"
                        </Button>
                        <Button
                            variant=Variant::Solid
                            icon=icons::MAIL
                            loading=sending
                            disabled=incomplete
                            on_click=Callback::new(send)
                        >
                            "Send message"
                        </Button>
                    </ButtonGroup>
                }
            })
        >
            <div class="space-y-3">
                <Show when=move || sent.get()>
                    <Alert
                        tone=Tone::Accent
                        title="Message sent"
                        description="We'll get back to you by email."
                    />
                </Show>
                {move || {
                    error
                        .get()
                        .map(|message| {
                            view! {
                                <Alert tone=Tone::Danger title="Not sent" description=message />
                            }
                        })
                }}

                <div class="grid gap-3 sm:grid-cols-2">
                    <Field label="Name">
                        <TextInput
                            value=name
                            on_input=Callback::new(move |v| name.set(v))
                            placeholder="Your name"
                            disabled=sending
                        />
                    </Field>
                    <Field label="Discord (optional)">
                        <TextInput
                            value=discord
                            on_input=Callback::new(move |v| discord.set(v))
                            placeholder="username"
                            disabled=sending
                        />
                    </Field>
                </div>
                <Field label="Reply-to email">
                    <TextInput
                        value=email
                        on_input=Callback::new(move |v| email.set(v))
                        input_type="email"
                        disabled=sending
                    />
                </Field>
                <Field label="Subject" required=true>
                    <TextInput
                        value=subject
                        on_input=Callback::new(move |v| subject.set(v))
                        placeholder="What do you need help with?"
                        disabled=sending
                    />
                </Field>
                <Field label="Message" required=true>
                    <TextArea
                        value=message
                        on_input=Callback::new(move |v| message.set(v))
                        rows=5
                        placeholder="Tell us what happened, and what you expected."
                        disabled=sending
                    />
                </Field>
            </div>
        </Modal>
    }
}
