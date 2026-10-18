use infinity_ui::prelude::*;
use infinity_ui::{MediaGallery, MediaItem, MediaKind};
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use super::legal::{EulaDocument, LINK};
use crate::api::{self, AircraftProduct};
use crate::auth::use_auth;
use crate::brand;
use crate::components::{EYEBROW, HoverRule, SectionLabel};
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
                    <div class=cn!("flex min-h-screen items-center justify-center", EYEBROW)>
                        "Loading…"
                    </div>
                }
                    .into_any();
            }
            if let Some(message) = error.get() {
                return view! {
                    <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
                        <p class=cn!("mb-6", EYEBROW)>"Infinity / Aircraft"</p>
                        <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                            "Catalog "
                            <span class="font-semibold text-accent-300">"unavailable."</span>
                        </h1>
                        <p class="max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                            {message}
                        </p>
                    </div>
                }
                    .into_any();
            }
            match product.get() {
                Some(product) => view! { <ProductDetail product=product /> }.into_any(),
                None => {
                    view! {
                        <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
                            <p class=cn!("mb-6", EYEBROW)>"Infinity / Aircraft"</p>
                            <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                                "Aircraft "
                                <span class="font-semibold text-accent-300">"not found."</span>
                            </h1>
                            <p class="mb-8 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                                "This product doesn't exist or isn't available."
                            </p>
                            <button
                                type="button"
                                on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                                class="group inline-flex items-center gap-2 self-start text-lg font-medium transition-colors hover:text-accent-300"
                            >
                                <Icon
                                    icon=icons::ARROW_LEFT
                                    class="h-4 w-4 transition-transform group-hover:-translate-x-1"
                                />
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
    // `tutorialUrl` still drops the tutorials row.
    let web_manual = manual::manual_link_for_product(&product.slug);
    let manual_location = product.manual_location().map(str::to_string);
    let show_manual = web_manual.is_some() || manual_location.is_some();
    let go = nav::use_go();

    view! {
        <div class="relative z-10 min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <nav class=cn!("mb-10 flex flex-wrap items-center gap-x-3 gap-y-1", EYEBROW)>
                    <span>"Infinity"</span>
                    <span class="text-white/25">"/"</span>
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                        class="uppercase transition-colors hover:text-accent-300"
                    >
                        "Aircraft"
                    </button>
                    <span class="text-white/25">"/"</span>
                    <span class="text-white/80">{product.name.clone()}</span>
                </nav>

                {product
                    .dev_only
                    .then(|| {
                        view! {
                            <Alert
                                tone=Tone::Dev
                                icon=icons::FLASK_CONICAL
                                title="Developer-only test product"
                                description="Not visible to public users."
                                class="mb-8"
                            />
                        }
                    })}

                <div class="grid items-start gap-12 lg:grid-cols-2 lg:gap-14">
                    <MediaGallery
                        items=gallery
                        label=product.name.clone()
                        class="relative z-10"
                    />

                    <div class="relative z-10">
                        <h1 class="text-4xl leading-[1.02] font-extralight tracking-tight md:text-6xl">
                            {product.name.clone()}
                        </h1>
                        <p class="mt-3 text-base font-light text-white/55">
                            "by "
                            <span class="font-medium text-accent-300">{product.developer.clone()}</span>
                        </p>
                        {product
                            .collaborators
                            .clone()
                            .map(|c| {
                                view! { <p class="mt-1 font-mono text-[11px] text-white/40">{c}</p> }
                            })}
                        {product
                            .platform_note
                            .clone()
                            .filter(|note| !note.is_empty())
                            .map(|note| {
                                view! {
                                    <p class="mt-5 border-l border-amber-400/60 pl-4 text-sm font-light text-amber-200/90">
                                        {note}
                                    </p>
                                }
                            })}

                        <div class="mt-8 space-y-4 border-l border-accent-400/40 pl-5">
                            {product
                                .long_description
                                .clone()
                                .into_iter()
                                .map(|para| {
                                    view! {
                                        <p class="leading-relaxed font-light text-white/70">{para}</p>
                                    }
                                })
                                .collect_view()}
                        </div>

                        <div class="mt-10 mb-6 border-t border-white/15 pt-5">
                            <p class=cn!("mb-2", EYEBROW)>"Price"</p>
                            <div class="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                                {
                                    let list_price = product
                                        .price_display
                                        .clone()
                                        .unwrap_or_default();
                                    move || match discount_price.get() {
                                        Some(price) => {
                                            view! {
                                                <span class="text-4xl font-light tracking-tight">{price}</span>
                                                <span class="text-lg font-light text-white/35 line-through">
                                                    {list_price.clone()}
                                                </span>
                                                <span class="font-mono text-[11px] tracking-[0.2em] text-accent-300 uppercase">
                                                    "Owner discount"
                                                </span>
                                            }
                                                .into_any()
                                        }
                                        None => {
                                            view! {
                                                <span class="text-4xl font-light tracking-tight">
                                                    {list_price.clone()}
                                                </span>
                                            }
                                                .into_any()
                                        }
                                    }
                                }
                            </div>
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

                        <dl class=cn!(
                            "grid gap-4 border-t border-white/10 pt-5 font-mono text-[11px]",
                            if show_manual { "grid-cols-3" } else { "grid-cols-2" },
                        )>
                            <Perk label="Delivery" value="Instant download" />
                            <Perk label="Support" value="Discord" />
                            {show_manual
                                .then(|| {
                                    view! {
                                        <Perk
                                            label="Manual"
                                            value=if web_manual.is_some() { "Online" } else { "In-sim EFB" }
                                        />
                                    }
                                })}
                        </dl>
                    </div>
                </div>

                {(!product.features.is_empty())
                    .then(|| {
                        view! {
                            <section class="mt-24">
                                <SectionLabel num="01" label="Features" />
                                <ul class="grid gap-x-10 md:grid-cols-2 lg:grid-cols-3">
                                    {product
                                        .features
                                        .clone()
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, feature)| {
                                            view! {
                                                <li class="flex gap-4 border-t border-white/10 py-4">
                                                    <span class="pt-0.5 font-mono text-[11px] text-accent-400/80">
                                                        {format!("{:02}", i + 1)}
                                                    </span>
                                                    <span class="text-sm leading-relaxed font-light text-white/80">
                                                        {feature}
                                                    </span>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            </section>
                        }
                    })}

                {(!product.specs.is_empty())
                    .then(|| {
                        view! {
                            <section class="mt-20">
                                <SectionLabel num="02" label="Specifications" />
                                <dl class="grid grid-cols-2 gap-x-10 lg:grid-cols-4">
                                    {product
                                        .specs
                                        .clone()
                                        .into_iter()
                                        .map(|spec| {
                                            view! {
                                                <div class="flex flex-col-reverse border-t border-white/10 pt-4 pb-8">
                                                    <dt class="mt-1 font-mono text-[11px] tracking-[0.2em] text-white/45 uppercase">
                                                        {spec.label}
                                                    </dt>
                                                    <dd class="text-2xl font-light tracking-tight md:text-3xl">
                                                        {spec.value}
                                                    </dd>
                                                </div>
                                            }
                                        })
                                        .collect_view()}
                                </dl>
                            </section>
                        }
                    })}

                <section class="mt-20">
                    <SectionLabel num="03" label="Support" />
                    <div class="border-b border-white/10">
                        <SupportRow
                            title="Discord Server"
                            copy="Join our community for support, updates, and discussions."
                            href=product.support_discord_url.clone()
                            action="Join Discord"
                            external=true
                        />
                        {product
                            .tutorial_url
                            .clone()
                            .map(|url| {
                                view! {
                                    <SupportRow
                                        title="Video Tutorials"
                                        copy="Step-by-step tutorials and flight demonstrations."
                                        href=Some(url)
                                        action="Watch tutorials"
                                        external=true
                                    />
                                }
                            })}
                        {show_manual
                            .then(|| match (web_manual.clone(), manual_location.clone()) {
                                (Some(href), _) => {
                                    view! {
                                        <SupportRow
                                            title="Flight Manual"
                                            copy="Systems, procedures and limitations, all in the online manual."
                                            href=Some(href.to_string())
                                            action="Read the manual"
                                            external=false
                                        />
                                    }
                                        .into_any()
                                }
                                (None, location) => {
                                    view! {
                                        <div class=SUPPORT_ROW>
                                            <h3 class="text-base font-medium">"Flight Manual"</h3>
                                            <p class="text-sm font-light text-white/55">
                                                "The full documentation and procedures guide ships inside the aircraft — there is nothing separate to download."
                                            </p>
                                            <span class="inline-flex items-center gap-2 font-mono text-[11px] text-accent-300">
                                                <Icon icon=brand::TABLET class="h-3.5 w-3.5 shrink-0" />
                                                {location}
                                            </span>
                                        </div>
                                    }
                                        .into_any()
                                }
                            })}
                    </div>
                </section>

                <button
                    type="button"
                    on:click=move |_| eula_open.set(true)
                    class="group relative mt-20 flex w-full flex-col items-start gap-1 border-t border-white/15 py-6 text-left transition-colors hover:bg-white/[0.03] md:flex-row md:items-center md:justify-between md:gap-8"
                >
                    <HoverRule />
                    <span>
                        <span class="block font-mono text-[11px] text-accent-400/80">"Licensing"</span>
                        <span class="mt-1 block max-w-2xl text-sm font-light text-white/55">
                            "The terms you agree to when you buy and install this aircraft."
                        </span>
                    </span>
                    <span class="mt-3 flex shrink-0 items-center gap-2 text-lg font-medium md:mt-0">
                        "End-User License Agreement"
                        <Icon
                            icon=icons::ARROW_RIGHT
                            class="h-4 w-4 transition-transform group-hover:translate-x-1"
                        />
                    </span>
                </button>
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
fn Perk(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div>
            <dt class="mb-1 tracking-[0.2em] text-white/40 uppercase">{label}</dt>
            <dd class="text-white/80">{value}</dd>
        </div>
    }
}

const SUPPORT_ROW: &str = "grid gap-2 border-t border-white/10 py-5 md:grid-cols-[14rem_1fr_auto] md:items-center md:gap-8";

/// One line of the support table. Without an `href` the row drops its action,
/// which is how a product with no Discord link shows up.
#[component]
fn SupportRow(
    title: &'static str,
    copy: &'static str,
    href: Option<String>,
    action: &'static str,
    external: bool,
) -> impl IntoView {
    match href {
        Some(href) => view! {
            <a
                href=href
                target=external.then_some("_blank")
                rel=external.then_some("noopener noreferrer")
                class=cn!("group relative transition-colors hover:bg-white/[0.03]", SUPPORT_ROW)
            >
                <HoverRule />
                <h3 class="text-base font-medium">{title}</h3>
                <p class="text-sm font-light text-white/55">{copy}</p>
                <span class="inline-flex items-center gap-2 text-sm font-medium">
                    {action}
                    <Icon
                        icon=if external { icons::ARROW_UP_RIGHT } else { icons::ARROW_RIGHT }
                        class="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5"
                    />
                </span>
            </a>
        }
        .into_any(),
        None => view! {
            <div class=SUPPORT_ROW>
                <h3 class="text-base font-medium">{title}</h3>
                <p class="text-sm font-light text-white/55">{copy}</p>
            </div>
        }
        .into_any(),
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
