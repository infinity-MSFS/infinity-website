use infinity_ui::prelude::*;

use crate::api::AircraftProduct;
use crate::nav;
use crate::state::use_catalog;

const DISCORD_URL: &str = "https://discord.gg/HZt4MCEYZk";

#[component]
pub fn AircraftListing() -> impl IntoView {
    let go = nav::use_go();
    let catalog = use_catalog();
    let products = catalog.listing();
    let loading = catalog.0.loading();
    let error = catalog.0.error;

    view! {
        <Show
            when=move || error.with(Option::is_none)
            fallback=move || {
                view! {
                    <div class="flex min-h-screen flex-col items-center justify-center px-4 text-white">
                        <h1 class="mb-2 text-2xl font-bold">"Catalog unavailable"</h1>
                        <p class="mb-6 max-w-md text-center text-white/60">
                            {move || error.get().unwrap_or_default()}
                        </p>
                    </div>
                }
            }
        >
            <div class="relative z-10 min-h-screen text-white">
                <div class="mx-auto max-w-6xl px-6 py-12 md:px-8">
                    <div class="mb-10 flex flex-col gap-6 md:flex-row md:items-end md:justify-between">
                        <div>
                            <span class="mb-3 inline-flex items-center rounded-md border border-accent-400/30 bg-accent-500/15 px-2.5 py-1 text-[10px] font-semibold tracking-wider text-accent-300 uppercase">
                                "Catalog"
                            </span>
                            <h1 class="mb-3 text-4xl font-bold md:text-5xl">"Aircraft"</h1>
                            <p class="max-w-xl text-white/70">
                                "High-fidelity aircraft for Microsoft Flight Simulator, built by hand and supported with free updates through the Infinity Manager."
                            </p>
                        </div>
                        <div class="grid shrink-0 grid-cols-2 gap-3 md:gap-4">
                            <HeaderStat
                                label="Available"
                                value=Signal::derive(move || {
                                    if loading.get() {
                                        "—".to_string()
                                    } else {
                                        products.with(|p| p.len().to_string())
                                    }
                                })
                            />
                            <HeaderStat
                                label="Platform"
                                value=Signal::derive(|| "MSFS 2024".to_string())
                            />
                        </div>
                    </div>

                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <SkeletonBody /> }
                    >
                        <Show
                            when=move || products.with(|p| !p.is_empty())
                            fallback=|| {
                                view! {
                                    <div class="rounded-xl border border-white/10 bg-white/[0.04] p-12 text-center text-white/60 backdrop-blur-sm">
                                        "No aircraft available right now. Check back soon."
                                    </div>
                                }
                            }
                        >
                            <AnimatedPage>
                                <FreewareCallout />
                                <div class="grid gap-6 md:grid-cols-2">
                                    {move || {
                                        products
                                            .get()
                                            .into_iter()
                                            .map(|product| {
                                                view! { <ProductCard product=product go=go /> }
                                            })
                                            .collect_view()
                                    }}
                                    <ComingSoonCard />
                                </div>
                            </AnimatedPage>
                        </Show>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ProductCard(product: AircraftProduct, go: Callback<String>) -> impl IntoView {
    let href = format!("{}/{}", nav::AIRCRAFT, product.slug);
    view! {
        <button
            type="button"
            on:click=move |_| go.run(href.clone())
            class="group relative flex flex-col overflow-hidden rounded-xl border border-white/10 bg-white/[0.04] text-left backdrop-blur-sm transition-all hover:border-accent-400/40 hover:bg-white/[0.07]"
        >
            {(product.dev_only || !product.storefront)
                .then(|| {
                    let label = if product.dev_only { "Dev" } else { "Hidden" };
                    view! {
                        <span class="absolute top-3 right-3 z-10">
                            <Badge tone=Tone::Dev icon=icons::FLASK_CONICAL squared=true>
                                {label}
                            </Badge>
                        </span>
                    }
                })}
            {product
                .price_display
                .clone()
                .map(|price| {
                    view! {
                        <div class="absolute top-3 left-3 z-10 inline-flex items-center rounded-md border border-accent-400/40 bg-black/60 px-2.5 py-1 text-sm font-bold text-accent-300 backdrop-blur-md">
                            {price}
                        </div>
                    }
                })}
            <div class="relative aspect-[16/9] overflow-hidden bg-white/5">
                <img
                    src=product.card_image.clone()
                    alt=product.name.clone()
                    class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
                />
                <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent"></div>
            </div>
            <div class="flex flex-1 flex-col p-6">
                <h2 class="mb-1 text-2xl font-bold">{product.name.clone()}</h2>
                <p class="mb-4 text-sm text-accent-300">{product.developer.clone()}</p>
                <p class="mb-6 line-clamp-2 flex-1 text-sm text-white/70">
                    {product.short_description.clone().unwrap_or_default()}
                </p>
                <div class="flex items-center justify-between border-t border-white/10 pt-4">
                    <span class="text-xs tracking-wider text-white/50 uppercase">"MSFS 2024"</span>
                    <span class="text-sm font-semibold text-accent-300 transition-transform group-hover:translate-x-1">
                        "View details →"
                    </span>
                </div>
            </div>
        </button>
    }
}

#[component]
fn HeaderStat(#[prop(into)] label: String, #[prop(into)] value: Signal<String>) -> impl IntoView {
    view! {
        <div class="min-w-[110px] rounded-lg border border-white/10 bg-white/[0.04] px-4 py-3 backdrop-blur-sm">
            <div class="mb-1 text-xs tracking-wider text-white/50 uppercase">{label}</div>
            <div class="text-sm font-semibold whitespace-nowrap text-white">{move || value.get()}</div>
        </div>
    }
}

#[component]
fn FreewareCallout() -> impl IntoView {
    let go = nav::use_go();
    view! {
        <div class="mb-6 rounded-xl border border-white/10 bg-white/[0.04] p-6 backdrop-blur-sm md:p-7">
            <div class="flex flex-col gap-5 md:flex-row md:items-center">
                <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-xl border border-accent-400/30 bg-accent-500/15">
                    <Icon icon=icons::GIFT class="h-7 w-7 text-accent-300" />
                </div>
                <div class="min-w-0 flex-1">
                    <h2 class="mb-1 text-xl font-bold">"Looking for freeware?"</h2>
                    <p class="text-sm text-white/70">
                        "Free aircraft from the community live inside the Infinity Manager. Browse, install, and keep them updated alongside your paid purchases, all in one place."
                    </p>
                </div>
                <button
                    type="button"
                    on:click=move |_| go.run(nav::MANAGER.to_string())
                    class="flex shrink-0 items-center justify-center gap-2 rounded-lg bg-accent-600 px-5 py-3 font-semibold text-white shadow-lg shadow-accent-600/20 transition-colors hover:bg-accent-700"
                >
                    "Get the Manager"
                    <Icon icon=icons::ARROW_RIGHT class="h-4 w-4" />
                </button>
            </div>
        </div>
    }
}

#[component]
fn ComingSoonCard() -> impl IntoView {
    view! {
        <div class="flex flex-col overflow-hidden rounded-xl border border-dashed border-white/15 bg-white/[0.02] backdrop-blur-sm">
            <div class="flex aspect-[16/9] items-center justify-center bg-gradient-to-br from-accent-900/30 via-black/30 to-black/10">
                <div class="flex h-14 w-14 items-center justify-center rounded-full border border-white/10 bg-white/5">
                    <Icon icon=icons::FLASK_CONICAL class="h-6 w-6 text-white/40" />
                </div>
            </div>
            <div class="flex flex-1 flex-col p-6">
                <h2 class="mb-1 text-2xl font-bold text-white/80">"More on the way"</h2>
                <p class="mb-4 text-sm text-accent-300/70">"In development"</p>
                <p class="mb-6 flex-1 text-sm text-white/55">
                    "We're a small team building each aircraft by hand. Join our Discord to follow development progress and be first to know when new airframes launch."
                </p>
                <div class="flex items-center justify-between border-t border-white/10 pt-4">
                    <span class="text-xs tracking-wider text-white/40 uppercase">"Coming soon"</span>
                    <a
                        href=DISCORD_URL
                        target="_blank"
                        rel="noopener noreferrer"
                        class="inline-block text-sm font-semibold text-accent-300 transition-transform hover:translate-x-1"
                    >
                        "Join Discord →"
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SkeletonBody() -> impl IntoView {
    view! {
        <div aria-busy="true" aria-label="Loading catalog">
            <div class="mb-6 rounded-xl border border-white/10 bg-white/[0.03] p-6 md:p-7">
                <div class="flex flex-col gap-5 md:flex-row md:items-center">
                    <div class="h-14 w-14 shrink-0 animate-pulse rounded-xl bg-white/5"></div>
                    <div class="min-w-0 flex-1 space-y-2">
                        <div class="h-5 w-44 animate-pulse rounded bg-white/10"></div>
                        <div class="h-3 w-full max-w-md animate-pulse rounded bg-white/5"></div>
                        <div class="h-3 w-3/4 max-w-sm animate-pulse rounded bg-white/5"></div>
                    </div>
                    <div class="h-12 w-44 shrink-0 animate-pulse rounded-lg bg-white/5"></div>
                </div>
            </div>

            <div class="grid gap-6 md:grid-cols-2">
                {(0..4)
                    .map(|_| {
                        view! {
                            <div class="flex flex-col overflow-hidden rounded-xl border border-white/10 bg-white/[0.03]">
                                <div class="aspect-[16/9] animate-pulse bg-white/5"></div>
                                <div class="flex flex-1 flex-col p-6">
                                    <div class="mb-3 h-6 w-2/3 animate-pulse rounded bg-white/10"></div>
                                    <div class="mb-5 h-3 w-1/3 animate-pulse rounded bg-white/5"></div>
                                    <div class="mb-6 space-y-2">
                                        <div class="h-3 w-full animate-pulse rounded bg-white/5"></div>
                                        <div class="h-3 w-4/5 animate-pulse rounded bg-white/5"></div>
                                    </div>
                                    <div class="flex items-center justify-between border-t border-white/10 pt-4">
                                        <div class="h-3 w-20 animate-pulse rounded bg-white/5"></div>
                                        <div class="h-3 w-24 animate-pulse rounded bg-white/10"></div>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
