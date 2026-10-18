use infinity_ui::prelude::*;
use leptos_router::hooks::use_query_map;

use super::legal::LINK;
use crate::auth::use_auth;
use crate::brand;
use crate::components::{EYEBROW, HoverRule, SectionLabel};
use crate::nav;
use crate::platform::platform_label;
use crate::state::use_manager;

#[component]
pub fn PurchaseSuccess() -> impl IntoView {
    let go = nav::use_go();
    let auth = use_auth();
    let query = use_query_map();
    let synced = RwSignal::new(false);
    let expected = Signal::derive(move || query.read().get("product"));

    Effect::new(move |_| {
        if auth.loading.get() || !auth.is_authenticated.get() || synced.get_untracked() {
            return;
        }
        synced.set(true);
        auth.sync_until_granted(expected.get_untracked());
    });

    let granted = Signal::derive(move || match expected.get() {
        Some(id) => auth.entitlements.get().products.contains(&id),
        None => false,
    });
    let manager = use_manager();
    let version = manager.version();
    let platforms = manager.platforms();
    let loading = manager.0.loading();
    let load_error = manager.0.error;
    let primary = Signal::derive(move || {
        platforms.with(|p| {
            p.get_key_value("windows-x86_64")
                .or_else(|| p.first())
                .map(|(k, v)| (k.clone(), v.url.clone()))
        })
    });
    let others = Signal::derive(move || {
        let primary_key = primary.get().map(|(k, _)| k);
        platforms.with(|p| {
            p.iter()
                .filter(|(k, _)| Some(*k) != primary_key.as_ref())
                .map(|(k, v)| (k.clone(), v.url.clone()))
                .collect::<Vec<_>>()
        })
    });

    let steps: Vec<AnyView> = vec![
        view! {
            <><strong class="font-medium text-white">"Download the Infinity Manager"</strong>
            " — our installer + launcher for all Infinity aircraft."</>
        }
        .into_any(),
        view! {
            <><strong class="font-medium text-white">"Sign in"</strong>
            " with the same account you used to check out."</>
        }
        .into_any(),
        view! {
            <>"Open the "<strong class="font-medium text-white">"Library"</strong>
            " tab and install your new aircraft."</>
        }
        .into_any(),
        view! {
            <>"Restart Microsoft Flight Simulator — your aircraft will appear in the selection menu."</>
        }
        .into_any(),
    ];

    view! {
        <AnimatedPage class="min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <header class="mb-16 max-w-3xl">
                    <p class=cn!("mb-10", EYEBROW)>"Infinity / Purchase"</p>
                    <h1 class="text-4xl leading-[1.05] font-extralight tracking-tight md:text-6xl">
                        "Thanks for your "
                        <span class="font-semibold text-accent-300">"purchase."</span>
                    </h1>
                    <p class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg">
                        {move || {
                            if granted.get() {
                                "Your purchase is now linked to your account."
                            } else if auth.syncing.get() {
                                "Linking your purchase to your account…"
                            } else if auth.is_authenticated.get() {
                                "Your purchase is being processed."
                            } else {
                                "Sign in with the account you checked out with to link this purchase."
                            }
                        }}
                    </p>
                    <Show when=move || auth.syncing.get()>
                        <p class=cn!("mt-6 flex items-center gap-3", EYEBROW)>
                            <Spinner size=Size::Xs />
                            "Syncing your new product…"
                        </p>
                    </Show>
                    <Show when=move || {
                        !auth.syncing.get() && !granted.get() && auth.is_authenticated.get()
                            && synced.get()
                    }>
                        <p class="mt-6 max-w-xl border-l border-amber-300/50 pl-5 text-sm leading-relaxed font-light text-amber-200/85">
                            "Payment confirmed, but your account has not synced yet — it can take a few minutes. Your product will appear on your dashboard shortly; if it does not, sign out and back in or contact support."
                        </p>
                    </Show>
                </header>

                <section class="mb-20">
                    <SectionLabel num="01" label="Install your aircraft" />
                    <ol class="max-w-3xl border-b border-white/10">
                        {steps
                            .into_iter()
                            .enumerate()
                            .map(|(index, body)| {
                                view! {
                                    <li class="grid grid-cols-[3rem_1fr] items-baseline border-t border-white/10 py-4 font-light text-white/70">
                                        <span class="font-mono text-[11px] text-accent-400/80">
                                            {format!("{:02}", index + 1)}
                                        </span>
                                        <span>{body}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ol>
                </section>

                <section class="mb-20">
                    <SectionLabel num="02" label="Download" />
                    <div class="flex flex-col gap-8 md:flex-row md:items-end md:justify-between">
                        <div class="max-w-xl">
                            <h2 class="text-3xl font-extralight tracking-tight md:text-4xl">
                                "Infinity "
                                <span class="font-semibold">"Manager"</span>
                            </h2>
                            <p class="mt-3 text-sm font-light text-white/55">
                                "Updates automatically once installed."
                            </p>
                            {move || {
                                version
                                    .get()
                                    .map(|v| {
                                        view! {
                                            <p class="mt-4 font-mono text-[11px] text-accent-300">
                                                {format!("v{v}")}
                                            </p>
                                        }
                                    })
                            }}
                        </div>
                        {move || {
                            if load_error.with(Option::is_some) {
                                return view! {
                                    <div class="border-l border-red-400/60 pl-4 text-sm text-red-200">
                                        "Couldn't load latest release."
                                    </div>
                                }
                                    .into_any();
                            }
                            if loading.get() {
                                return view! {
                                    <div class=cn!("flex items-center gap-3", EYEBROW)>
                                        <Icon
                                            icon=icons::LOADER_CIRCLE
                                            class="h-3.5 w-3.5 animate-spin"
                                        />
                                        "Loading…"
                                    </div>
                                }
                                    .into_any();
                            }
                            match primary.get() {
                                Some((key, url)) => {
                                    view! {
                                        <a
                                            href=url
                                            class="group inline-flex items-center gap-2 self-start text-lg font-medium transition-colors hover:text-accent-300 md:self-auto"
                                        >
                                            <Icon
                                                icon=icons::DOWNLOAD
                                                class="h-4 w-4 transition-transform group-hover:translate-y-0.5"
                                            />
                                            {format!("Download for {}", platform_label(&key))}
                                        </a>
                                    }
                                        .into_any()
                                }
                                None => ().into_any(),
                            }
                        }}
                    </div>

                    <Show when=move || others.with(|o| !o.is_empty())>
                        <div class=cn!(
                            "mt-8 flex flex-wrap items-center gap-x-6 gap-y-2 border-t border-white/10 pt-5", EYEBROW
                        )>
                            <span class="text-white/35">"Other platforms"</span>
                            {move || {
                                others
                                    .get()
                                    .into_iter()
                                    .map(|(key, url)| {
                                        view! {
                                            <a
                                                href=url
                                                class="inline-flex items-center gap-1.5 transition-colors hover:text-accent-300"
                                            >
                                                <Icon icon=icons::DOWNLOAD class="h-3 w-3" />
                                                {platform_label(&key)}
                                            </a>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </Show>
                </section>

                <section class="mb-16">
                    <SectionLabel num="03" label="Next" />
                    <div class="border-b border-white/10">
                        <NextRow label="Go to Dashboard" on_go=move || go.run(nav::USER.to_string()) />
                        <NextRow
                            label="Browse More Aircraft"
                            on_go=move || go.run(nav::AIRCRAFT.to_string())
                        />
                    </div>
                </section>

                // The refund window starts here and this is the one moment the
                // buyer is provably reading. Also heads off the "I don't
                // recognise this charge" dispute, since the descriptor isn't
                // our name.
                <div class="max-w-2xl space-y-2 text-xs leading-relaxed font-light text-white/50">
                    <p>
                        <span class="font-normal text-white/75">"Changed your mind?"</span>
                        " Full refund within 14 days of this purchase, as long as you haven't installed the aircraft yet. If the aircraft turns out to be faulty or not as described, there's no time limit and no install condition on that. "
                        <a href=nav::LEGAL_REFUNDS class=LINK>
                            "Refund policy"
                        </a>
                    </p>
                    <p>
                        "Your receipt comes from Lemon Squeezy, our merchant of record; your statement will show "
                        <code class="font-mono text-white/75">"LEMSQZY*"</code>
                        "."
                    </p>
                </div>
            </div>
        </AnimatedPage>
    }
}

/// A next-step row: hairline above, accent rule on hover, trailing arrow.
#[component]
fn NextRow(label: &'static str, on_go: impl Fn() + 'static) -> impl IntoView {
    view! {
        <button
            type="button"
            on:click=move |_| on_go()
            class="group relative flex w-full items-center justify-between gap-6 border-t border-white/10 py-5 text-left transition-colors hover:bg-white/[0.03]"
        >
            <HoverRule />
            <span class="text-base font-medium">{label}</span>
            <Icon
                icon=icons::ARROW_RIGHT
                class="h-4 w-4 text-white/50 transition-transform group-hover:translate-x-0.5 group-hover:text-accent-300"
            />
        </button>
    }
}

// There is nothing to show about a checkout that never completed beyond how to
// get help. This page used to render an "Attempted Product Details" card that
// always read "Product key not available" — on a site whose own copy promises
// no license keys.
#[component]
pub fn PurchaseCancelled() -> impl IntoView {
    let go = nav::use_go();

    view! {
        <AnimatedPage class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 py-10 text-white md:px-10">
            <p class=cn!("mb-6", EYEBROW)>"Infinity / Purchase"</p>
            <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                "Purchase "
                <span class="font-semibold text-accent-300">"cancelled."</span>
            </h1>
            <p class="mb-12 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                "Your purchase was not completed. No charges have been made to your account."
            </p>

            <div class="mb-12 max-w-xl border-t border-white/10 pt-5">
                <h2 class=cn!("mb-3 flex items-center gap-2", EYEBROW)>
                    <Icon icon=brand::LIFE_BUOY class="h-3.5 w-3.5 text-accent-400/80" />
                    "Something go wrong?"
                </h2>
                <p class="text-sm leading-relaxed font-light text-white/60">
                    "If you meant to buy and the checkout failed, or you think you have been charged when you shouldn't have been, get in touch from your "
                    <a href=nav::USER class=LINK>
                        "dashboard"
                    </a>
                    " and we'll sort it out. Our "
                    <a href=nav::LEGAL_REFUNDS class=LINK>
                        "refund policy"
                    </a>
                    " explains what happens next."
                </p>
            </div>

            <div class="max-w-xl border-b border-white/10">
                <NextRow label="Go to Dashboard" on_go=move || go.run(nav::USER.to_string()) />
                <NextRow
                    label="Browse Products Again"
                    on_go=move || go.run(nav::AIRCRAFT.to_string())
                />
            </div>
        </AnimatedPage>
    }
}
