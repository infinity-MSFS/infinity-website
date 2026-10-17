use infinity_ui::prelude::*;
use leptos_router::hooks::use_query_map;

use super::legal::LINK;
use crate::auth::use_auth;
use crate::brand;
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
            <><strong class="text-white">"Download the Infinity Manager"</strong>
            " — our installer + launcher for all Infinity aircraft."</>
        }
        .into_any(),
        view! {
            <><strong class="text-white">"Sign in"</strong>
            " with the same account you used to check out."</>
        }
        .into_any(),
        view! {
            <>"Open the "<strong class="text-white">"Library"</strong>
            " tab and install your new aircraft."</>
        }
        .into_any(),
        view! {
            <>"Restart Microsoft Flight Simulator — your aircraft will appear in the selection menu."</>
        }
        .into_any(),
    ];

    view! {
        <div class="flex min-h-screen items-center justify-center px-4 py-12 text-white">
            <div class="w-full max-w-2xl space-y-6">
                <div class="rounded-2xl border border-white/10 bg-white/[0.04] p-8 text-center backdrop-blur-sm">
                    <div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full border border-accent-400/30 bg-accent-500/15">
                        <Icon icon=icons::CIRCLE_CHECK class="h-9 w-9 text-accent-300" />
                    </div>
                    <h1 class="mb-2 text-3xl font-bold md:text-4xl">
                        "Thanks for your purchase!"
                    </h1>
                    <p class="text-white/60">
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
                        <p class="mt-3 inline-flex items-center gap-2 text-xs text-white/50">
                            <Spinner size=Size::Xs />
                            "Syncing your new product…"
                        </p>
                    </Show>
                    <Show when=move || {
                        !auth.syncing.get() && !granted.get() && auth.is_authenticated.get()
                            && synced.get()
                    }>
                        <p class="mx-auto mt-3 max-w-md text-xs text-amber-300/90">
                            "Payment confirmed, but your account has not synced yet — it can take a few minutes. Your product will appear on your dashboard shortly; if it does not, sign out and back in or contact support."
                        </p>
                    </Show>
                </div>

                <div class="rounded-2xl border border-white/10 bg-white/[0.04] p-6 backdrop-blur-sm md:p-7">
                    <h2 class="mb-4 flex items-center gap-2 text-lg font-semibold">
                        <Icon icon=icons::PLANE class="h-5 w-5 text-accent-400" />
                        "Install your aircraft"
                    </h2>
                    <ol class="space-y-3 text-sm">
                        {steps
                            .into_iter()
                            .enumerate()
                            .map(|(index, body)| {
                                view! {
                                    <li class="flex items-start gap-3 text-white/75">
                                        <span class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-accent-400/30 bg-accent-500/15 text-xs font-bold text-accent-300">
                                            {index + 1}
                                        </span>
                                        <span>{body}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ol>
                </div>

                <div class="rounded-2xl border border-white/10 bg-white/[0.04] p-6 backdrop-blur-sm md:p-7">
                    <div class="flex flex-col gap-4 md:flex-row md:items-center">
                        <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-accent-400/30 bg-accent-500/15">
                            <Icon icon=icons::CPU class="h-6 w-6 text-accent-300" />
                        </div>
                        <div class="min-w-0 flex-1">
                            <div class="flex flex-wrap items-center gap-2">
                                <h3 class="font-semibold">"Infinity Manager"</h3>
                                {move || {
                                    version
                                        .get()
                                        .map(|v| {
                                            view! {
                                                <span class="inline-flex items-center rounded-md border border-accent-400/30 bg-accent-500/15 px-2 py-0.5 text-[10px] font-semibold tracking-wider text-accent-300 uppercase">
                                                    {format!("v{v}")}
                                                </span>
                                            }
                                        })
                                }}
                            </div>
                            <p class="mt-0.5 text-xs text-white/50">
                                "Updates automatically once installed."
                            </p>
                        </div>
                        {move || {
                            if load_error.with(Option::is_some) {
                                return view! {
                                    <div class="text-xs text-red-300">
                                        "Couldn't load latest release."
                                    </div>
                                }
                                    .into_any();
                            }
                            if loading.get() {
                                return view! {
                                    <div class="flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2.5 text-sm text-white/50">
                                        <Icon
                                            icon=icons::LOADER_CIRCLE
                                            class="h-4 w-4 animate-spin"
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
                                            class="flex items-center justify-center gap-2 rounded-lg bg-accent-600 px-5 py-2.5 font-semibold text-white shadow-lg shadow-accent-600/25 transition-colors hover:bg-accent-700"
                                        >
                                            <Icon icon=icons::DOWNLOAD class="h-4 w-4" />
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
                        <div class="mt-4 flex flex-wrap gap-2 border-t border-white/5 pt-4 text-xs">
                            <span class="mr-1 text-white/40">"Other platforms:"</span>
                            {move || {
                                others
                                    .get()
                                    .into_iter()
                                    .map(|(key, url)| {
                                        view! {
                                            <a
                                                href=url
                                                class="inline-flex items-center gap-1 rounded-md border border-white/10 bg-white/[0.04] px-2 py-1 text-white/70 transition-colors hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
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
                </div>

                <div class="flex flex-col gap-3 sm:flex-row">
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::USER.to_string())
                        class="flex flex-1 items-center justify-center gap-2 rounded-lg border border-white/15 bg-white/[0.06] px-5 py-3 font-medium text-white backdrop-blur-sm transition-colors hover:border-white/25 hover:bg-white/[0.10]"
                    >
                        <Icon icon=icons::LOG_IN class="h-4 w-4" />
                        "Go to Dashboard"
                    </button>
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                        class="flex flex-1 items-center justify-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-5 py-3 font-medium text-white/80 transition-colors hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                    >
                        "Browse More Aircraft"
                        <Icon icon=icons::ARROW_RIGHT class="h-4 w-4" />
                    </button>
                </div>

                // The refund window starts here and this is the one moment the
                // buyer is provably reading. Also heads off the "I don't
                // recognise this charge" dispute, since the descriptor isn't
                // our name.
                <div class="space-y-2 px-1 text-xs text-white/50">
                    <p>
                        <span class="text-white/70">"Changed your mind?"</span>
                        " Full refund within 14 days of this purchase, as long as you haven't installed the aircraft yet. If the aircraft turns out to be faulty or not as described, there's no time limit and no install condition on that. "
                        <a href=nav::LEGAL_REFUNDS class=LINK>
                            "Refund policy"
                        </a>
                    </p>
                    <p>
                        "Your receipt comes from Lemon Squeezy, our merchant of record; your statement will show "
                        <code>"LEMSQZY*"</code>
                        "."
                    </p>
                </div>
            </div>
        </div>
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
        <div class="flex min-h-screen items-center justify-center px-4 py-12 text-white">
            <div class="w-full max-w-md rounded-2xl border border-white/10 bg-white/5 p-8 text-center shadow-xl backdrop-blur-md">
                <Icon icon=icons::CIRCLE_X class="mx-auto mb-4 h-16 w-16 text-red-400" />
                <h1 class="mb-3 text-3xl font-bold">"Purchase Cancelled"</h1>
                <p class="mb-6 text-white/60">
                    "Your purchase was not completed. No charges have been made to your account."
                </p>

                <div class="mb-6 rounded-lg border border-white/10 bg-white/5 p-5 text-left">
                    <h2 class="mb-2 flex items-center gap-2 text-base font-semibold">
                        <Icon icon=brand::LIFE_BUOY class="h-4 w-4 text-accent-400" />
                        "Something go wrong?"
                    </h2>
                    <p class="text-sm text-white/60">
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

                <div class="space-y-3">
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::USER.to_string())
                        class="flex w-full items-center justify-center gap-2 rounded-lg bg-accent-600 px-6 py-3 font-semibold text-white transition-colors hover:bg-accent-700"
                    >
                        <Icon icon=icons::HOUSE class="h-5 w-5" />
                        "Go to Dashboard"
                    </button>
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                        class="flex w-full items-center justify-center gap-2 rounded-lg bg-white/10 px-6 py-3 font-semibold text-white transition-colors hover:bg-white/20"
                    >
                        "Browse Products Again"
                    </button>
                </div>
            </div>
        </div>
    }
}
