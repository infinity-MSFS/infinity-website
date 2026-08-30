use infinity_ui::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, AircraftProduct, MIN_PASSWORD_LENGTH, SupportTicket};
use crate::auth::{User, use_auth};
use crate::components::DownloadButton;
use crate::nav;
use crate::state::{use_catalog, use_manager};

const DISCORD_URL: &str = "https://discord.gg/HZt4MCEYZk";

#[component]
pub fn UserDashboard() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Show
            when=move || !auth.loading.get()
            fallback=|| {
                view! {
                    <div class="flex min-h-screen items-center justify-center">
                        <div class="flex flex-col items-center gap-3 text-white/60">
                            <Spinner size=Size::Lg />
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
        <AnimatedPage class="flex min-h-screen items-center justify-center px-4">
            <Panel class="w-full max-w-md p-8 text-center">
                <Icon icon=icons::CIRCLE_ALERT class="mx-auto mb-4 h-16 w-16 text-amber-400" />
                <h2 class="mb-2 text-2xl font-bold text-white">"Login required"</h2>
                <p class="mb-6 text-white/60">"Sign in to view your account dashboard."</p>
                <Button
                    variant=Variant::Solid
                    icon=icons::LOG_IN
                    on_click=Callback::new(move |()| auth.login())
                >
                    "Sign in"
                </Button>
            </Panel>
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
        <AnimatedPage class="min-h-screen px-4 py-12 text-white">
            <div class="mx-auto max-w-5xl space-y-8">
                <ProfileCard />
                <SecurityCard />
                <ManagerCard />

                <section>
                    <div class="mb-4 flex items-center justify-between">
                        <h2 class="flex items-center gap-2 text-2xl font-bold">
                            <Icon icon=icons::PACKAGE class="h-6 w-6 text-accent-400" />
                            "Your products"
                        </h2>
                        <span class="text-sm text-white/40">
                            {move || format!("{} owned", total.get())}
                        </span>
                    </div>

                    <Show
                        when=move || has_products.get()
                        fallback=move || {
                            view! {
                                <EmptyState
                                    icon=icons::PACKAGE
                                    title="No products yet"
                                    description="Browse our aircraft to get started — purchases land on your account automatically."
                                >
                                    <Button
                                        variant=Variant::Solid
                                        trailing_icon=icons::ARROW_RIGHT
                                        on_click=Callback::new(move |()| {
                                            go.run(nav::AIRCRAFT.to_string())
                                        })
                                    >
                                        "Browse aircraft"
                                    </Button>
                                </EmptyState>
                            }
                        }
                    >
                        <div class="grid gap-4 sm:grid-cols-2">
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

                    <p class="mt-4 text-xs text-white/40">
                        "Install and update your aircraft from inside the Infinity Manager above."
                    </p>
                </section>

                <Panel class="p-6">
                    <h3 class="mb-2 text-lg font-semibold">"Need help?"</h3>
                    <p class="mb-4 text-sm text-white/60">
                        "Join the Discord community for the fastest response, or open a ticket if you would rather email."
                    </p>
                    <ButtonGroup class="flex-wrap">
                        <Button
                            variant=Variant::Solid
                            icon=icons::MESSAGE_CIRCLE
                            href=DISCORD_URL
                            target="_blank"
                        >
                            "Join Discord"
                        </Button>
                        <Button
                            variant=Variant::Glass
                            icon=icons::MAIL
                            on_click=Callback::new(move |()| support_open.set(true))
                        >
                            "Email support"
                        </Button>
                    </ButtonGroup>
                </Panel>
            </div>

            <SupportModal open=support_open />
        </AnimatedPage>
    }
}

#[component]
fn ProfileCard() -> impl IntoView {
    let auth = use_auth();
    let user = Signal::derive(move || auth.user.get().unwrap_or_default());

    view! {
        <Panel class="p-6 md:p-8">
            <div class="flex flex-col gap-6 md:flex-row md:items-center">
                {move || {
                    let user = user.get();
                    view! {
                        <Avatar
                            name=user.display_name()
                            src=user.picture.clone()
                            size=Size::Lg
                            class="h-20 w-20 border-2 border-white/15 text-2xl"
                        />
                    }
                }}
                <div class="min-w-0 flex-1">
                    <h1 class="truncate text-2xl font-bold md:text-3xl">
                        {move || user.get().display_name()}
                    </h1>
                    <p class="truncate text-white/60">
                        {move || user.get().email.unwrap_or_default()}
                    </p>
                    <div class="mt-3 flex flex-wrap gap-2">
                        <Show when=move || auth.entitlements.get().is_developer()>
                            <Badge tone=Tone::Dev icon=icons::FLASK_CONICAL squared=true>
                                "Developer"
                            </Badge>
                        </Show>
                        {move || {
                            auth.entitlements
                                .get()
                                .channels
                                .into_iter()
                                .filter(|channel| channel != "stable")
                                .map(|channel| {
                                    view! {
                                        <Badge tone=Tone::Accent squared=true>
                                            {channel}
                                        </Badge>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
                <Button
                    variant=Variant::Glass
                    icon=icons::LOG_OUT
                    on_click=Callback::new(move |()| auth.logout())
                    class="self-start md:self-center"
                >
                    "Sign out"
                </Button>
            </div>
        </Panel>
    }
}

/// Password change - handled remotely, we are just passing it to worker
#[component]
fn SecurityCard() -> impl IntoView {
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
        <Panel class="p-6">
            <div class="flex gap-5">
                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-accent-400/30 bg-accent-500/15">
                    <Icon icon=icons::KEY_ROUND class="h-6 w-6 text-accent-300" />
                </div>
                <div class="min-w-0 flex-1">
                    <Show
                        when=move || database_account.get()
                        fallback=move || {
                            view! {
                                <h2 class="text-lg font-semibold">"Password"</h2>
                                <p class="mt-1 text-sm text-white/60">
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
                            <div>
                                <h2 class="text-lg font-semibold">"Password"</h2>
                                <p class="mt-1 text-sm text-white/60">
                                    "Enter your current password, then choose a new one."
                                </p>
                            </div>
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

                        <div class="mt-4 grid gap-3 sm:grid-cols-3">
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
                </div>
            </div>
        </Panel>
    }
}

#[component]
fn ManagerCard() -> impl IntoView {
    let manager = use_manager();
    let version = manager.version();
    let platforms = manager.platforms();
    let loading = manager.0.loading();
    let error = manager.0.error;

    view! {
        <Show
            when=move || error.with(Option::is_none)
            fallback=move || {
                view! {
                    <Alert
                        tone=Tone::Danger
                        title="Couldn't load the latest Infinity Manager release"
                        description=error.get().unwrap_or_default()
                    />
                }
            }
        >
            <Panel class="p-6 md:p-7">
                <div class="flex flex-col gap-5 md:flex-row md:items-center">
                    <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-xl border border-accent-400/30 bg-accent-500/15">
                        <Icon icon=icons::CPU class="h-7 w-7 text-accent-300" />
                    </div>
                    <div class="min-w-0 flex-1">
                        <div class="flex flex-wrap items-center gap-2">
                            <h2 class="text-xl font-bold">"Infinity Manager"</h2>
                            {move || {
                                version
                                    .get()
                                    .map(|v| {
                                        view! {
                                            <Badge tone=Tone::Accent squared=true>
                                                {format!("v{v}")}
                                            </Badge>
                                        }
                                    })
                            }}
                            <Badge tone=Tone::Accent squared=true>"Auto-updating"</Badge>
                        </div>
                        <p class="mt-1 text-sm text-white/60">
                            "Install and update your aircraft with the latest manager. Already installed? Updates download automatically."
                        </p>
                    </div>
                    <DownloadButton platforms=platforms loading=loading class="shrink-0" />
                </div>
            </Panel>
        </Show>
    }
}

#[component]
fn OwnedProduct(product: AircraftProduct) -> impl IntoView {
    let go = nav::use_go();
    let href = format!("{}/{}", nav::AIRCRAFT, product.slug);
    let open_href = href.clone();
    let openable = product.storefront;

    view! {
        <Panel class="group flex flex-col overflow-hidden">
            <button
                type="button"
                disabled=!openable
                on:click=move |_| {
                    if openable {
                        go.run(href.clone());
                    }
                }
                class="relative aspect-video overflow-hidden bg-white/5 disabled:cursor-default"
            >
                <img
                    src=product.card_image.clone()
                    alt=product.name.clone()
                    class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-[1.02]"
                />
                <span class="absolute top-3 left-3">
                    <Badge tone=Tone::Accent squared=true>"Owned"</Badge>
                </span>
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
            </button>
            <div class="flex flex-1 items-start justify-between gap-3 p-5">
                <div class="min-w-0">
                    <h3 class="truncate text-lg leading-tight font-bold">{product.name.clone()}</h3>
                    <p class="truncate text-sm text-accent-300">{product.developer.clone()}</p>
                </div>
                {openable
                    .then(|| {
                        view! {
                            <IconButton
                                icon=icons::EXTERNAL_LINK
                                label="View product page"
                                variant=Variant::Glass
                                size=Size::Sm
                                on_click=Callback::new(move |()| go.run(open_href.clone()))
                            />
                        }
                    })}
            </div>
        </Panel>
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
        <Panel class="flex flex-col overflow-hidden">
            <div class="relative flex aspect-video items-center justify-center bg-gradient-to-br from-accent-900/40 via-black/40 to-black/20">
                <Icon icon=icons::PACKAGE class="h-12 w-12 text-accent-300/60" />
                <span class="absolute top-3 left-3">
                    <Badge tone=Tone::Accent squared=true>"Owned"</Badge>
                </span>
            </div>
            <div class="flex flex-1 flex-col gap-1 p-5">
                <h3 class="text-lg leading-tight font-bold">{title}</h3>
                <p class="font-mono text-xs text-white/50">{id}</p>
            </div>
        </Panel>
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
