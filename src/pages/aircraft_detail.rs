use infinity_ui::prelude::*;
use infinity_ui::{MediaGallery, MediaItem, MediaKind};
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use super::legal::{EulaDocument, LINK};
use crate::api::{self, AircraftProduct};
use crate::auth::use_auth;
use crate::brand;
use crate::manual;
use crate::nav;
use crate::state::use_catalog;

#[component]
pub fn AircraftDetail() -> impl IntoView {
    let go = nav::use_go();
    let params = use_params_map();
    let catalog = use_catalog();
    let loading = catalog.0.loading();
    let error = catalog.0.error;
    let slug = Signal::derive(move || params.read().get("slug").unwrap_or_default());
    let staff = use_auth();
    let product = Signal::derive(move || {
        let developer = staff.entitlements.get().is_developer();
        catalog
            .by_slug(&slug.get())
            .filter(|p| developer || p.storefront)
    });

    view! {
        {move || {
            if loading.get() {
                return view! {
                    <div class="flex min-h-screen items-center justify-center text-white/60">
                        "Loading…"
                    </div>
                }
                    .into_any();
            }
            if let Some(message) = error.get() {
                return view! {
                    <div class="flex min-h-screen flex-col items-center justify-center px-4 text-white">
                        <h1 class="mb-2 text-2xl font-bold">"Catalog unavailable"</h1>
                        <p class="mb-6 max-w-md text-center text-white/60">{message}</p>
                    </div>
                }
                    .into_any();
            }
            match product.get() {
                Some(product) => view! { <ProductDetail product=product /> }.into_any(),
                None => {
                    view! {
                        <div class="flex min-h-screen flex-col items-center justify-center px-4 text-white">
                            <h1 class="mb-2 text-3xl font-bold">"Aircraft not found"</h1>
                            <p class="mb-6 text-white/60">
                                "This product doesn't exist or isn't available."
                            </p>
                            <button
                                type="button"
                                on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                                class="rounded-lg bg-accent-600 px-6 py-2 text-white hover:bg-accent-700"
                            >
                                "Back to Aircraft"
                            </button>
                        </div>
                    }
                        .into_any()
                }
            }
        }}
    }
}

#[component]
fn ProductDetail(product: AircraftProduct) -> impl IntoView {
    let eula_open = RwSignal::new(false);

    let gallery: Vec<MediaItem> = product
        .media
        .iter()
        .map(|item| {
            let media = MediaItem {
                kind: if item.kind == "video" {
                    MediaKind::Video
                } else {
                    MediaKind::Image
                },
                url: item.url.clone(),
                thumbnail: item.thumbnail.clone(),
                alt: None,
            };
            media
        })
        .collect();
    let gallery = Signal::derive(move || gallery.clone());

    let auth = use_auth();
    let checkout_error = RwSignal::new(None::<String>);
    let checkout_pending = RwSignal::new(false);
    // Gates the Purchase button. Its evidentiary value needs the worker to
    // store the acknowledgement against the order — until it does, this is a
    // disclosure control only.
    let ack_policy = RwSignal::new(false);

    let entitlements = StoredValue::new_local(product.entitlements());
    let owned = Signal::derive(move || {
        auth.is_authenticated.get()
            && entitlements.with_value(|ids| auth.entitlements.get().owns_any(ids))
    });

    let discount = StoredValue::new_local(product.owner_discount.clone());
    let discount_price = Signal::derive(move || {
        if owned.get() || !auth.is_authenticated.get() {
            return None;
        }
        let offer = discount.get_value()?;
        auth.entitlements
            .get()
            .products
            .contains(&offer.requires_product)
            .then(|| offer.display())
    });

    let slug = StoredValue::new_local(product.slug.clone());
    let purchase = move |()| {
        checkout_error.set(None);
        let tab =
            web_sys::window().and_then(|w| w.open_with_url_and_target("", "_blank").ok().flatten());
        checkout_pending.set(true);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                checkout_pending.set(false);
                checkout_error.set(Some(
                    "Your session expired. Sign in again to continue.".into(),
                ));
                if let Some(tab) = tab {
                    let _ = tab.close();
                }
                return;
            };
            let user = auth.user.get_untracked();
            let result = api::create_checkout(
                &slug.get_value(),
                &token,
                user.as_ref().and_then(|u| u.email.clone()),
                user.as_ref().and_then(|u| u.name.clone()),
            )
            .await;
            checkout_pending.set(false);
            match result {
                Ok(checkout) => match tab {
                    Some(tab) => {
                        let _ = tab.location().set_href(&checkout.url);
                    }
                    None => {
                        let _ = web_sys::window()
                            .and_then(|w| w.open_with_url_and_target(&checkout.url, "_blank").ok());
                    }
                },
                Err(err) => {
                    if let Some(tab) = tab {
                        let _ = tab.close();
                    }
                    let message = match err.code.as_str() {
                        "already_owned" => {
                            // The cached token is stale; refresh so the button
                            // flips to "Already owned" instead of lying.
                            spawn_local(async move { auth.refresh_entitlements(true).await });
                            "You already own this product. Check your dashboard.".to_string()
                        }
                        "rate_limited" => {
                            "Too many checkout attempts. Try again tomorrow, or contact support."
                                .to_string()
                        }
                        _ => err.message,
                    };
                    checkout_error.set(Some(message));
                }
            }
        });
    };

    // A product with a web manual (crate::manual::links) links to it. Every
    // other Infinity aircraft carries its manual in the in-sim EFB, so the
    // manual surfaces show by default; `manual_location()` is `None` only for
    // a product that opts out with `manualLocation: ""`. A missing
    // `tutorialUrl` still drops the tutorials card.
    let web_manual = manual::manual_link_for_product(&product.slug);
    let manual_location = product.manual_location().map(str::to_string);
    let show_manual = web_manual.is_some() || manual_location.is_some();
    let show_tutorials = product.tutorial_url.is_some();
    let support_cards = 1 + usize::from(show_tutorials) + usize::from(show_manual);
    let support_grid = match support_cards {
        3 => "grid gap-8 md:grid-cols-3",
        2 => "mx-auto grid max-w-2xl gap-8 md:grid-cols-2",
        _ => "mx-auto grid max-w-md gap-8",
    };

    view! {
        <div class="relative z-10 min-h-screen text-white">
            <div class="mx-auto max-w-7xl px-8 py-12">
                {product
                    .dev_only
                    .then(|| {
                        view! {
                            <Alert
                                tone=Tone::Dev
                                icon=icons::FLASK_CONICAL
                                title="Developer-only test product"
                                description="Not visible to public users."
                                class="mb-6"
                            />
                        }
                    })}

                <div class="grid items-start gap-12 lg:grid-cols-2">
                    <MediaGallery
                        items=gallery
                        label=product.name.clone()
                        class="relative z-10"
                    />

                    <div class="relative z-10 space-y-8">
                        <div>
                            <h1 class="mb-2 text-4xl font-bold">{product.name.clone()}</h1>
                            <p class="mb-0 text-xl text-accent-300">
                                {format!("by {}", product.developer)}
                            </p>
                            {product
                                .collaborators
                                .clone()
                                .map(|c| {
                                    view! { <p class="mb-4 text-xs text-accent-300/80">{c}</p> }
                                })}
                            <p class="mb-6 text-sm text-amber-300">
                                {product.platform_note.clone().unwrap_or_default()}
                            </p>

                            {product
                                .long_description
                                .clone()
                                .into_iter()
                                .map(|para| {
                                    view! {
                                        <p class="mb-8 leading-relaxed text-white/90">{para}</p>
                                    }
                                })
                                .collect_view()}

                            <div class="mb-8 flex flex-wrap items-center gap-4">
                                {
                                    let list_price = product
                                        .price_display
                                        .clone()
                                        .unwrap_or_default();
                                    move || match discount_price.get() {
                                        Some(price) => {
                                            view! {
                                                <div class="text-3xl font-bold text-accent-400">{price}</div>
                                                <div class="text-xl text-white/40 line-through">
                                                    {list_price.clone()}
                                                </div>
                                                <Badge tone=Tone::Accent icon=icons::CHECK>
                                                    "Owner discount"
                                                </Badge>
                                            }
                                                .into_any()
                                        }
                                        None => {
                                            view! {
                                                <div class="text-3xl font-bold text-accent-400">
                                                    {list_price.clone()}
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }
                                }
                            </div>

                            {if product.is_server_down {
                                view! {
                                    <Alert
                                        tone=Tone::Warning
                                        title="Purchasing is temporarily unavailable"
                                        description="We are working through temporary server issues. Please check back soon."
                                        class="mb-8"
                                    />
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="mb-8">
                                        {move || {
                                            if !auth.is_authenticated.get() {
                                                return view! {
                                                    <Button
                                                        variant=Variant::Solid
                                                        size=Size::Lg
                                                        icon=icons::LOG_IN
                                                        full_width=true
                                                        on_click=Callback::new(move |()| auth.login())
                                                    >
                                                        "Sign in to purchase"
                                                    </Button>
                                                }
                                                    .into_any();
                                            }
                                            if owned.get() {
                                                return view! {
                                                    <Button
                                                        variant=Variant::Glass
                                                        size=Size::Lg
                                                        icon=icons::CIRCLE_CHECK
                                                        full_width=true
                                                        disabled=Signal::from(true)
                                                    >
                                                        "Already owned"
                                                    </Button>
                                                }
                                                    .into_any();
                                            }
                                            view! {
                                                <label class="mb-3 flex cursor-pointer items-start gap-2.5 text-xs text-white/50 select-none">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || ack_policy.get()
                                                        on:change=move |ev| {
                                                            ack_policy.set(event_target_checked(&ev))
                                                        }
                                                        class="mt-0.5 h-4 w-4 shrink-0 cursor-pointer rounded border-white/20 bg-white/5 accent-accent-600"
                                                    />
                                                    <span>
                                                        "I've read the "
                                                        <a href=nav::LEGAL_REFUNDS class=LINK>
                                                            "refund policy"
                                                        </a>
                                                        " and understand this is a licence to use the aircraft, not a transfer of ownership."
                                                    </span>
                                                </label>
                                                <div title=move || {
                                                    (!ack_policy.get())
                                                        .then_some("Please confirm you've read the refund policy")
                                                }>
                                                    <Button
                                                        variant=Variant::Solid
                                                        size=Size::Lg
                                                        icon=icons::SHOPPING_CART
                                                        full_width=true
                                                        loading=checkout_pending
                                                        disabled=Signal::derive(move || !ack_policy.get())
                                                        on_click=Callback::new(purchase)
                                                    >
                                                        "Purchase"
                                                    </Button>
                                                </div>
                                            }
                                                .into_any()
                                        }}

                                        {move || {
                                            checkout_error
                                                .get()
                                                .map(|message| {
                                                    view! {
                                                        <p class="mt-2 text-sm text-red-400">{message}</p>
                                                    }
                                                })
                                        }}

                                        {move || {
                                            auth.user
                                                .get()
                                                .and_then(|user| user.email)
                                                .map(|email| {
                                                    view! {
                                                        <p class="mt-2 text-xs text-white/40">
                                                            "Checkout email will be pre-filled with "
                                                            <span class="text-white/70">{email}</span>
                                                        </p>
                                                    }
                                                })
                                        }}
                                    </div>
                                }
                                    .into_any()
                            }}

                            // Pre-purchase disclosure. This has to render above
                            // the fold of the buy decision, not inside the EULA
                            // modal: a refund policy only binds a buyer who could
                            // read it before paying, and most buyers never open
                            // the modal.
                            <div class="-mt-4 mb-8 space-y-2 text-xs text-white/40">
                                <p>
                                    "This is a " <span class="text-white/60">"licence to use"</span>
                                    " the aircraft, not a transfer of ownership, and the free Infinity Manager is required to install, update and run it. "
                                    <button
                                        type="button"
                                        on:click=move |_| eula_open.set(true)
                                        class=LINK
                                    >
                                        "Read the EULA"
                                    </button>
                                    " "
                                    <a
                                        href=nav::legal_eula(&product.slug)
                                        class="text-white/50 underline underline-offset-2 hover:text-white/70"
                                    >
                                        "(open as page)"
                                    </a>
                                </p>
                                <p>
                                    <span class="text-white/60">"Refunds:"</span>
                                    " full refund within 14 days of purchase, as long as you haven't installed the aircraft yet, and subject to the conditions in our refund policy. Faulty or not as described? No time limit — we'll fix it or refund it. Your statutory rights are not affected. "
                                    <a href=nav::LEGAL_REFUNDS class=LINK>
                                        "Read the full refund policy"
                                    </a>
                                    "."
                                </p>
                                <p>
                                    "Payment is taken by Lemon Squeezy, our merchant of record; your statement will show "
                                    <code>"LEMSQZY*"</code>
                                    "."
                                </p>
                            </div>

                            <div class=cn!(
                                "grid gap-4 border-y border-white/10 py-6 text-center",
                                if show_manual { "grid-cols-3" } else { "grid-cols-2" },
                            )>
                                <div>
                                    <Icon
                                        icon=icons::DOWNLOAD
                                        class="mx-auto mb-2 h-6 w-6 text-accent-400"
                                    />
                                    <div class="text-sm text-white/60">"Instant Download"</div>
                                </div>
                                <div>
                                    <Icon
                                        icon=icons::USERS
                                        class="mx-auto mb-2 h-6 w-6 text-accent-400"
                                    />
                                    <div class="text-sm text-white/60">"Discord Support"</div>
                                </div>
                                {show_manual
                                    .then(|| {
                                        view! {
                                            <div>
                                                <Icon
                                                    icon=icons::FILE_TEXT
                                                    class="mx-auto mb-2 h-6 w-6 text-accent-400"
                                                />
                                                <div class="text-sm text-white/60">"Full Manual"</div>
                                                <div class="text-sm text-white/60">
                                                    {if web_manual.is_some() { "Online" } else { "In-Sim EFB" }}
                                                </div>
                                            </div>
                                        }
                                    })}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="mt-16">
                    <h2 class="mb-8 text-center text-3xl font-bold">"Features"</h2>
                    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                        {product
                            .features
                            .clone()
                            .into_iter()
                            .map(|feature| {
                                view! {
                                    <div class="rounded-lg border border-white/10 bg-white/5 p-4 transition-colors hover:bg-white/10">
                                        <div class="flex items-start space-x-3">
                                            <div class="mt-2 h-2 w-2 shrink-0 rounded-full bg-accent-400"></div>
                                            <span class="text-white/90">{feature}</span>
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>

                <div class="mt-16">
                    <h2 class="mb-8 text-center text-3xl font-bold">
                        "Performance Specifications"
                    </h2>
                    <div class="rounded-xl border border-white/10 bg-white/5 p-8">
                        <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
                            {product
                                .specs
                                .clone()
                                .into_iter()
                                .map(|spec| {
                                    view! {
                                        <div class="text-center">
                                            <div class="mb-1 text-2xl font-bold text-accent-400">
                                                {spec.value}
                                            </div>
                                            <div class="text-sm text-white/60">{spec.label}</div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </div>

                <div class="mt-16 rounded-xl border border-white/10 bg-white/5 p-8">
                    <h2 class="mb-6 text-center text-2xl font-bold">"Support & Community"</h2>
                    <div class=support_grid>
                        <div class="text-center">
                            <Icon
                                icon=icons::MESSAGE_CIRCLE
                                class="mx-auto mb-4 h-12 w-12 text-accent-400"
                            />
                            <h3 class="mb-2 text-xl font-semibold">"Discord Server"</h3>
                            <p class="mb-4 text-white/60">
                                "Join our community for support, updates, and discussions"
                            </p>
                            {product
                                .support_discord_url
                                .clone()
                                .map(|url| {
                                    view! {
                                        <a
                                            href=url
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="inline-block rounded-lg bg-accent-600 px-6 py-2 text-white transition-colors hover:bg-accent-700"
                                        >
                                            "Join Discord"
                                        </a>
                                    }
                                })}
                        </div>
                        {product
                            .tutorial_url
                            .clone()
                            .map(|url| {
                                view! {
                                    <div class="text-center">
                                        <Icon
                                            icon=icons::YOUTUBE
                                            class="mx-auto mb-4 h-12 w-12 text-red-400"
                                        />
                                        <h3 class="mb-2 text-xl font-semibold">"Video Tutorials"</h3>
                                        <p class="mb-4 text-white/60">
                                            "Watch step-by-step tutorials and flight demonstrations"
                                        </p>
                                        <a
                                            href=url
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="inline-block rounded-lg bg-red-600 px-6 py-2 text-white transition-colors hover:bg-red-700"
                                        >
                                            "Watch Tutorials"
                                        </a>
                                    </div>
                                }
                            })}
                        {show_manual
                            .then(|| {
                                let body = match (web_manual.clone(), manual_location.clone()) {
                                    (Some(href), _) => {
                                        view! {
                                            <p class="mb-4 text-white/60">
                                                "Systems, procedures and limitations, all in the online manual."
                                            </p>
                                            <a
                                                href=href
                                                class="inline-block rounded-lg bg-accent-600 px-6 py-2 text-white transition-colors hover:bg-accent-700"
                                            >
                                                "Read the Manual"
                                            </a>
                                        }
                                            .into_any()
                                    }
                                    (None, location) => {
                                        view! {
                                            <p class="mb-4 text-white/60">
                                                "The full documentation and procedures guide ships inside the aircraft — there is nothing separate to download."
                                            </p>
                                            <div class="inline-flex items-center gap-2 rounded-lg border border-accent-400/30 bg-accent-500/10 px-4 py-2 text-sm font-semibold text-accent-200">
                                                <Icon icon=brand::TABLET class="h-4 w-4 shrink-0" />
                                                {location}
                                            </div>
                                        }
                                            .into_any()
                                    }
                                };
                                view! {
                                    <div class="text-center">
                                        <Icon
                                            icon=icons::FILE_TEXT
                                            class="mx-auto mb-4 h-12 w-12 text-accent-400"
                                        />
                                        <h3 class="mb-2 text-xl font-semibold">"Flight Manual"</h3>
                                        {body}
                                    </div>
                                }
                            })}
                    </div>
                </div>

                <div class="mt-12 text-center">
                    <button
                        type="button"
                        on:click=move |_| eula_open.set(true)
                        class="inline-flex items-center space-x-2 rounded-lg border border-white/10 bg-white/5 px-6 py-3 text-white/80 transition-colors hover:bg-white/10 hover:text-white"
                    >
                        <Icon icon=icons::SHIELD class="h-5 w-5" />
                        <span>"End-User License Agreement (EULA)"</span>
                    </button>
                </div>

                <div class="mt-12 text-center text-white/60">
                    <p class="mb-2">
                        "Take command and experience this aircraft in Microsoft Flight Simulator!"
                    </p>
                </div>
            </div>

            <EulaModal
                product=product.clone()
                open=eula_open
                on_close=Callback::new(move |()| eula_open.set(false))
            />
        </div>
    }
}

#[component]
fn EulaModal(
    product: AircraftProduct,
    open: RwSignal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Modal
            open=open
            on_close=on_close
            title="End-User License Agreement"
            icon=icons::SHIELD
            width="max-w-4xl"
            footer=ViewFn::from(move || {
                view! {
                    <Button
                        variant=Variant::Solid
                        on_click=Callback::new(move |()| on_close.run(()))
                    >
                        "Close"
                    </Button>
                }
            })
        >
            <div class="space-y-4 text-sm leading-relaxed text-white/90">
                <EulaDocument product=product.clone() />
            </div>
        </Modal>
    }
}
