use infinity_ui::prelude::*;

use crate::components::{DownloadButton, EYEBROW, HoverRule, SectionLabel};
use crate::nav;
use crate::state::use_manager;

const FEATURES: &[(&str, &str)] = &[
    (
        "Built for Safety",
        "We build the Manager using Rust and ReScript, two modern programming languages known for catching mistakes before they ever reach you. The result is fewer crashes, fewer surprises, and a tool you can trust.",
    ),
    (
        "Addon Configuration",
        "Fine-tune every aspect of your aircraft addons with an intuitive configuration interface.",
    ),
    (
        "Seamless Setup",
        "Sign in to the Manager once and every Infinity aircraft you own is activated automatically in the sim. No in-cockpit login, no license keys, no copy-paste.",
    ),
    (
        "Self Updater",
        "Stay up-to-date effortlessly with seamless in-app updates for all your Infinity distributed aircraft and the manager.",
    ),
    (
        "Fragmented Downloads",
        "Intelligent download management with fragmentation for faster, more reliable addon installations.",
    ),
    (
        "Cross Platform",
        "Available on Windows and Linux, ensuring all pilots can access the full Infinity experience.",
    ),
];

const REQUIREMENTS: &[(&str, &str)] = &[
    (
        "Operating System",
        "Windows 10 / 11 (64-bit). Linux builds available for advanced users; macOS is not currently supported.",
    ),
    (
        "Runtime",
        "Microsoft Flight Simulator 2024 installed. A live internet connection is required for sign-in, updates, and P2P multiplayer.",
    ),
    (
        "Footprint",
        "Under 100 MB installed. Per-aircraft download size is set by the product, and fragmented downloads keep update sizes small.",
    ),
];

#[component]
pub fn Manager() -> impl IntoView {
    let go = nav::use_go();
    let manager = use_manager();
    let version = manager.version();
    let platforms = manager.platforms();
    let load_error = manager.0.error;
    let loading = manager.0.loading();

    view! {
        <AnimatedPage class="min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <header class="mb-16 max-w-3xl">
                    <p class=cn!("mb-10", EYEBROW)>"Infinity / The Manager"</p>
                    <h1 class="text-5xl leading-[1.02] font-extralight tracking-tight md:text-7xl">
                        "One app."
                        <br />
                        <span class="font-semibold text-accent-300">"Every aircraft."</span>
                    </h1>
                    <p class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg">
                        "The Infinity Manager is the official companion app for every Infinity aircraft. It handles addon configuration, seamless updates, one-click product activation, and a growing set of utilities that make flying our aircraft easier."
                    </p>
                </header>

                <div class="mb-20 grid gap-10 lg:grid-cols-[3fr_2fr] lg:gap-14">
                    <div class="overflow-hidden rounded-sm border border-white/10">
                        <img
                            src="/manager.png"
                            alt="Infinity Manager Interface"
                            class="h-auto w-full"
                        />
                    </div>

                    <div class="flex flex-col justify-between gap-8">
                        <p class="leading-relaxed font-light text-white/70">
                            "The Manager is built with Rust and ReScript, two modern programming languages picked specifically because they help us catch problems before they ever reach your PC. The result is software that stays fast, stable, and out of your way."
                        </p>
                        <div class="border-t border-white/10 pt-5">
                            <p class="mb-2 font-mono text-[11px] tracking-[0.25em] text-accent-400/80 uppercase">
                                "Note"
                            </p>
                            <p class="text-sm leading-relaxed font-light text-white/70">
                                "The Manager is required to install and run any payware aircraft distributed through the Infinity ecosystem. It is free for everyone and updates itself automatically."
                            </p>
                        </div>
                        <a
                            href=nav::DISCORD_URL
                            target="_blank"
                            rel="noopener noreferrer"
                            class="group inline-flex items-center gap-2 self-start text-sm text-white/60 transition-colors hover:text-white"
                        >
                            <Icon icon=crate::brand::DISCORD class="h-4 w-4" label="Discord" />
                            "Questions? Join the Discord"
                            <Icon
                                icon=icons::ARROW_UP_RIGHT
                                class="h-3.5 w-3.5 opacity-60 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
                            />
                        </a>
                    </div>
                </div>

                <section class="mb-20">
                    <SectionLabel num="01" label="Download" />
                    <div class="flex flex-col gap-8 md:flex-row md:items-end md:justify-between">
                        <div class="max-w-xl">
                            <h2 class="text-3xl font-extralight tracking-tight md:text-4xl">
                                "Get the "
                                <span class="font-semibold">"Manager"</span>
                            </h2>
                            <p class="mt-3 text-sm font-light text-white/55">
                                "Free for everyone. Sign in to your account to manage installs and updates, or grab the latest build directly."
                            </p>
                            {move || {
                                version
                                    .get()
                                    .map(|v| {
                                        view! {
                                            <p class="mt-4 font-mono text-[11px] text-white/40">
                                                "Latest release  /  "
                                                <span class="text-accent-300">{format!("v{v}")}</span>
                                            </p>
                                        }
                                    })
                            }}
                        </div>
                        <div class="flex shrink-0 flex-col gap-3 sm:flex-row">
                            <Show
                                when=move || load_error.with(Option::is_none)
                                fallback=|| {
                                    view! {
                                        <div class="flex items-center border-l border-red-400/60 pl-4 text-sm text-red-200">
                                            "Couldn't load the latest release. Try again shortly."
                                        </div>
                                    }
                                }
                            >
                                <DownloadButton platforms=platforms loading=loading />
                            </Show>
                            <button
                                type="button"
                                on:click=move |_| go.run(nav::USER.to_string())
                                class="flex items-center justify-center gap-2 rounded-sm border border-white/15 px-5 py-3 font-medium text-white transition-colors hover:border-white/30 hover:bg-white/[0.04]"
                            >
                                <Icon icon=icons::LOG_IN class="h-4 w-4" />
                                "Open Dashboard"
                            </button>
                        </div>
                    </div>
                </section>

                <section class="mb-20">
                    <SectionLabel num="02" label="Features" />
                    <div class="grid gap-x-10 md:grid-cols-2 lg:grid-cols-3">
                        {FEATURES
                            .iter()
                            .enumerate()
                            .map(|(i, (title, copy))| {
                                view! {
                                    <div class="group relative border-t border-white/10 pt-5 pb-8">
                                        <HoverRule />
                                        <p class="mb-3 font-mono text-[11px] text-accent-400/80">
                                            {format!("{:02}", i + 1)}
                                        </p>
                                        <h3 class="mb-2 text-lg font-medium">{*title}</h3>
                                        <p class="text-sm leading-relaxed font-light text-white/55">
                                            {*copy}
                                        </p>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </section>

                <section class="mb-20">
                    <SectionLabel num="03" label="System Requirements" />
                    <dl class="border-t border-white/10">
                        {REQUIREMENTS
                            .iter()
                            .map(|(title, copy)| {
                                view! {
                                    <div class="grid gap-2 border-b border-white/10 py-5 md:grid-cols-[14rem_1fr] md:gap-8">
                                        <dt class="font-mono text-xs tracking-wider text-white/45 uppercase">
                                            {*title}
                                        </dt>
                                        <dd class="text-sm leading-relaxed font-light text-white/75">
                                            {*copy}
                                        </dd>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </dl>
                </section>

                <button
                    type="button"
                    on:click=move |_| go.run(nav::DOCS.to_string())
                    class="group relative flex w-full flex-col items-start gap-1 border-t border-white/15 py-6 text-left transition-colors hover:bg-white/[0.03] md:flex-row md:items-center md:justify-between md:gap-8"
                >
                    <HoverRule />
                    <span>
                        <span class="block font-mono text-[11px] text-accent-400/80">
                            "Building your own aircraft?"
                        </span>
                        <span class="mt-1 block max-w-2xl text-sm font-light text-white/55">
                            "Read the infinity-rs developer docs, our Rust toolkit for MSFS 2024 WASM gauges, systems, and SimConnect clients."
                        </span>
                    </span>
                    <span class="mt-3 flex shrink-0 items-center gap-2 text-lg font-medium md:mt-0">
                        "Developer Docs"
                        <Icon
                            icon=icons::ARROW_RIGHT
                            class="h-4 w-4 transition-transform group-hover:translate-x-1"
                        />
                    </span>
                </button>
            </div>
        </AnimatedPage>
    }
}
