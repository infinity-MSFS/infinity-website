use infinity_ui::prelude::*;

use crate::api::AircraftProduct;
use crate::components::{EYEBROW, HoverRule, SectionLabel};
use crate::nav;
use crate::state::use_catalog;

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
                    <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
                        <p class=cn!("mb-6", EYEBROW)>"Infinity / Aircraft"</p>
                        <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                            "Catalog "
                            <span class="font-semibold text-accent-300">"unavailable."</span>
                        </h1>
                        <p class="max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                            {move || error.get().unwrap_or_default()}
                        </p>
                    </div>
                }
            }
        >
            <div class="relative z-10 min-h-screen text-white">
                <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                    <header class="mb-16 flex flex-col gap-10 md:flex-row md:items-end md:justify-between">
                        <div class="max-w-3xl">
                            <p class=cn!("mb-10", EYEBROW)>"Infinity / Aircraft"</p>
                            <h1 class="text-5xl leading-[1.02] font-extralight tracking-tight md:text-7xl">
                                "Hand-built aircraft."
                                <br />
                                <span class="font-semibold text-accent-300">"Free updates included."</span>
                            </h1>
                            <p class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg">
                                "High-fidelity aircraft for Microsoft Flight Simulator, built by hand and supported with free updates through the Infinity Manager."
                            </p>
                        </div>
                        <dl class="flex shrink-0 gap-10 font-mono text-[11px] md:flex-col md:gap-4 md:text-right">
                            <HeaderStat
                                label="Available"
                                value=Signal::derive(move || {
                                    if loading.get() {
                                        "—".to_string()
                                    } else {
                                        products.with(|p| format!("{:02}", p.len()))
                                    }
                                })
                            />
                            <HeaderStat
                                label="Platform"
                                value=Signal::derive(|| "MSFS 2024".to_string())
                            />
                        </dl>
                    </header>

                    <SectionLabel num="01" label="Catalog" />
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <SkeletonBody /> }
                    >
                        <Show
                            when=move || products.with(|p| !p.is_empty())
                            fallback=|| {
                                view! {
                                    <p class="border-t border-white/10 py-10 font-light text-white/55">
                                        "No aircraft available right now. Check back soon."
                                    </p>
                                }
                            }
                        >
                            <AnimatedPage>
                                <div class="grid gap-x-10 gap-y-14 md:grid-cols-2">
                                    {move || {
                                        products
                                            .get()
                                            .into_iter()
                                            .enumerate()
                                            .map(|(i, product)| {
                                                view! { <ProductCard index=i + 1 product=product go=go /> }
                                            })
                                            .collect_view()
                                    }}
                                    <ComingSoonCard />
                                </div>
                                <FreewareRow />
                            </AnimatedPage>
                        </Show>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ProductCard(index: usize, product: AircraftProduct, go: Callback<String>) -> impl IntoView {
    let href = format!("{}/{}", nav::AIRCRAFT, product.slug);
    view! {
        <button
            type="button"
            on:click=move |_| go.run(href.clone())
            class="group flex flex-col text-left"
        >
            <div class="relative aspect-[16/9] overflow-hidden rounded-sm border border-white/10 bg-white/5">
                <img
                    src=product.card_image.clone()
                    alt=product.name.clone()
                    class="h-full w-full object-cover transition-transform duration-700 group-hover:scale-[1.03]"
                />
                {(product.dev_only || !product.storefront)
                    .then(|| {
                        let label = if product.dev_only { "Dev" } else { "Hidden" };
                        view! {
                            <span class="absolute top-3 right-3">
                                <Badge tone=Tone::Dev icon=icons::FLASK_CONICAL squared=true>
                                    {label}
                                </Badge>
                            </span>
                        }
                    })}
            </div>
            <div class="relative mt-5 flex flex-1 flex-col border-t border-white/10 pt-4">
                <HoverRule />
                <div class="mb-3 flex items-center justify-between font-mono text-[11px]">
                    <span class="text-accent-400/80">{format!("{index:02}")}</span>
                    {product
                        .price_display
                        .clone()
                        .map(|price| view! { <span class="text-white/70">{price}</span> })}
                </div>
                <h2 class="text-2xl font-medium tracking-tight">{product.name.clone()}</h2>
                <p class="mt-1 mb-4 text-sm font-light text-white/45">
                    {product.developer.clone()}
                </p>
                <p class="mb-5 line-clamp-2 flex-1 text-sm leading-relaxed font-light text-white/65">
                    {product.short_description.clone().unwrap_or_default()}
                </p>
                <span class="flex items-center gap-2 text-sm font-medium text-white">
                    "View details"
                    <Icon
                        icon=icons::ARROW_RIGHT
                        class="h-4 w-4 transition-transform group-hover:translate-x-1"
                    />
                </span>
            </div>
        </button>
    }
}

#[component]
fn HeaderStat(#[prop(into)] label: String, #[prop(into)] value: Signal<String>) -> impl IntoView {
    view! {
        <div>
            <dt class="mb-1 tracking-[0.25em] text-white/40 uppercase">{label}</dt>
            <dd class="text-sm whitespace-nowrap text-accent-300">{move || value.get()}</dd>
        </div>
    }
}

#[component]
fn FreewareRow() -> impl IntoView {
    let go = nav::use_go();
    view! {
        <button
            type="button"
            on:click=move |_| go.run(nav::MANAGER.to_string())
            class="group relative mt-20 flex w-full flex-col items-start gap-1 border-t border-white/15 py-6 text-left transition-colors hover:bg-white/[0.03] md:flex-row md:items-center md:justify-between md:gap-8"
        >
            <HoverRule />
            <span>
                <span class="block font-mono text-[11px] text-accent-400/80">
                    "Looking for freeware?"
                </span>
                <span class="mt-1 block max-w-2xl text-sm font-light text-white/55">
                    "Free aircraft from the community live inside the Infinity Manager. Browse, install, and keep them updated alongside your paid purchases, all in one place."
                </span>
            </span>
            <span class="mt-3 flex shrink-0 items-center gap-2 text-lg font-medium md:mt-0">
                "Get the Manager"
                <Icon
                    icon=icons::ARROW_RIGHT
                    class="h-4 w-4 transition-transform group-hover:translate-x-1"
                />
            </span>
        </button>
    }
}

#[component]
fn ComingSoonCard() -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <div class="flex aspect-[16/9] items-center justify-center rounded-sm border border-dashed border-white/15">
                <span class=EYEBROW>"In development"</span>
            </div>
            <div class="mt-5 flex flex-1 flex-col border-t border-white/10 pt-4">
                <p class="mb-3 font-mono text-[11px] text-white/30">"—"</p>
                <h2 class="text-2xl font-medium tracking-tight text-white/80">"More on the way"</h2>
                <p class="mt-1 mb-4 text-sm font-light text-white/45">"Coming soon"</p>
                <p class="mb-5 flex-1 text-sm leading-relaxed font-light text-white/55">
                    "We're a small team building each aircraft by hand. Join our Discord to follow development progress and be first to know when new airframes launch."
                </p>
                <a
                    href=nav::DISCORD_URL
                    target="_blank"
                    rel="noopener noreferrer"
                    class="group inline-flex items-center gap-2 self-start text-sm font-medium text-white/80 transition-colors hover:text-white"
                >
                    "Join the Discord"
                    <Icon
                        icon=icons::ARROW_UP_RIGHT
                        class="h-3.5 w-3.5 opacity-60 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
                    />
                </a>
            </div>
        </div>
    }
}

#[component]
fn SkeletonBody() -> impl IntoView {
    view! {
        <div aria-busy="true" aria-label="Loading catalog" class="grid gap-x-10 gap-y-14 md:grid-cols-2">
            {(0..4)
                .map(|_| {
                    view! {
                        <div class="flex flex-col">
                            <div class="aspect-[16/9] animate-pulse rounded-sm bg-white/5"></div>
                            <div class="mt-5 border-t border-white/10 pt-4">
                                <div class="mb-4 h-3 w-8 animate-pulse rounded-sm bg-white/10"></div>
                                <div class="mb-3 h-6 w-2/3 animate-pulse rounded-sm bg-white/10"></div>
                                <div class="mb-5 h-3 w-1/3 animate-pulse rounded-sm bg-white/5"></div>
                                <div class="space-y-2">
                                    <div class="h-3 w-full animate-pulse rounded-sm bg-white/5"></div>
                                    <div class="h-3 w-4/5 animate-pulse rounded-sm bg-white/5"></div>
                                </div>
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
