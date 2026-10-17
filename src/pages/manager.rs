use infinity_ui::prelude::*;

use crate::components::DownloadButton;
use crate::nav;
use crate::state::use_manager;

const DISCORD_URL: &str = "https://discord.gg/HZt4MCEYZk";

const FEATURES: &[(IconData, &str, &str, &str)] = &[
    (
        icons::SHIELD_CHECK,
        "bg-orange-500/20",
        "text-orange-400",
        "Built for Safety",
    ),
    (
        icons::COG,
        "bg-purple-500/20",
        "text-purple-400",
        "Addon Configuration",
    ),
    (
        icons::KEY_ROUND,
        "bg-blue-500/20",
        "text-blue-400",
        "Seamless Setup",
    ),
    (
        icons::REFRESH_CW,
        "bg-cyan-500/20",
        "text-cyan-400",
        "Self Updater",
    ),
    (
        icons::DOWNLOAD,
        "bg-yellow-500/20",
        "text-yellow-400",
        "Fragmented Downloads",
    ),
    (
        icons::MONITOR,
        "bg-amber-500/20",
        "text-amber-400",
        "Cross Platform",
    ),
];

const FEATURE_COPY: &[&str] = &[
    "We build the Manager using Rust and ReScript, two modern programming languages known for catching mistakes before they ever reach you. The result is fewer crashes, fewer surprises, and a tool you can trust.",
    "Fine-tune every aspect of your aircraft addons with an intuitive configuration interface.",
    "Sign in to the Manager once and every Infinity aircraft you own is activated automatically in the sim. No in-cockpit login, no license keys, no copy-paste.",
    "Stay up-to-date effortlessly with seamless in-app updates for all your Infinity distributed aircraft and the manager.",
    "Intelligent download management with fragmentation for faster, more reliable addon installations.",
    "Available on Windows and Linux, ensuring all pilots can access the full Infinity experience.",
];

const REQUIREMENTS: &[(IconData, &str, &str)] = &[
    (
        icons::MONITOR,
        "Operating System",
        "Windows 10 / 11 (64-bit). Linux builds available for advanced users; macOS is not currently supported.",
    ),
    (
        icons::CPU,
        "Runtime",
        "Microsoft Flight Simulator 2024 installed. A live internet connection is required for sign-in, updates, and P2P multiplayer.",
    ),
    (
        icons::HARD_DRIVE,
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
        <AnimatedPage class="min-h-screen px-4 py-12 text-white">
            <div class="mx-auto max-w-7xl">
                <div class="mb-12 text-center">
                    <h1 style="text-shadow: white 1px 0 70px;" class="mb-4 text-5xl font-bold">
                        "The Infinity Manager"
                    </h1>
                </div>

                <div class="mb-16 grid gap-12 lg:grid-cols-2">
                    <div class="space-y-4">
                        <div class="relative overflow-hidden rounded-xl border border-white/10 bg-white/[0.04] shadow-2xl backdrop-blur-sm">
                            <img
                                src="/manager.png"
                                alt="Infinity Manager Interface"
                                class="h-auto w-full"
                            />
                        </div>
                    </div>

                    <div class="relative z-10 space-y-6">
                        <div>
                            <h2 class="mb-4 text-3xl font-bold">"About the Manager"</h2>
                            <p class="mb-6 leading-relaxed text-white/90">
                                "The Infinity Manager is the official companion app for every Infinity aircraft. It handles addon configuration, seamless updates, one-click product activation, and a growing set of utilities that make flying our aircraft easier."
                            </p>
                            <p class="mb-6 leading-relaxed text-white/90">
                                "The Manager is built with Rust and ReScript, two modern programming languages picked specifically because they help us catch problems before they ever reach your PC. The result is software that stays fast, stable, and out of your way."
                            </p>
                            <div class="mb-6 rounded-lg border border-accent-500/30 bg-accent-500/10 p-4 backdrop-blur-sm">
                                <p class="text-sm text-accent-300">
                                    "The Manager is required to install and run any payware aircraft distributed through the Infinity ecosystem. It is free for everyone and updates itself automatically."
                                </p>
                            </div>
                            <div class="flex gap-4">
                                <a
                                    href=DISCORD_URL
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="flex items-center gap-2 rounded-lg border border-[#5865F2]/20 bg-[#5865F2]/10 px-6 py-3 transition-colors hover:border-[#5865F2]/40 hover:bg-[#5865F2]/20"
                                >
                                    <Icon icon=crate::brand::DISCORD class="h-5 w-5" label="Discord" />
                                    <span class="text-sm font-medium">"Join Discord"</span>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="mb-16 rounded-xl border border-white/10 bg-white/[0.04] p-6 backdrop-blur-sm md:p-8">
                    <div class="flex flex-col gap-6 md:flex-row md:items-center">
                        <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-xl border border-accent-400/30 bg-accent-500/15">
                            <Icon icon=icons::DOWNLOAD class="h-7 w-7 text-accent-300" />
                        </div>
                        <div class="min-w-0 flex-1">
                            <div class="mb-1 flex flex-wrap items-center gap-2">
                                <h2 class="text-2xl font-bold">"Get the Manager"</h2>
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
                            <p class="text-sm text-white/70">
                                "Free for everyone. Sign in to your account to manage installs and updates, or grab the latest build directly."
                            </p>
                        </div>
                        <div class="flex shrink-0 flex-col gap-3 sm:flex-row">
                            <Show
                                when=move || load_error.with(Option::is_none)
                                fallback=|| {
                                    view! {
                                        <div class="flex items-center rounded-lg border border-red-500/30 bg-red-500/10 px-5 py-3 text-sm text-red-200">
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
                                class="flex items-center justify-center gap-2 rounded-lg border border-white/10 bg-white/[0.06] px-5 py-3 font-semibold text-white transition-colors hover:border-white/20 hover:bg-white/[0.10]"
                            >
                                <Icon icon=icons::LOG_IN class="h-4 w-4" />
                                "Open Dashboard"
                            </button>
                        </div>
                    </div>
                </div>

                <div class="mt-16">
                    <h2 class="mb-8 text-center text-3xl font-bold">"Key Features"</h2>
                    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                        {FEATURES
                            .iter()
                            .zip(FEATURE_COPY)
                            .map(|((icon, tint, glyph, title), copy)| {
                                view! {
                                    <div class="rounded-xl border border-white/10 bg-white/[0.04] p-6 backdrop-blur-sm transition-all hover:border-accent-400/40 hover:bg-white/[0.07]">
                                        <div class="mb-4 flex items-center gap-4">
                                            <div class=cn!("rounded-lg p-3", *tint)>
                                                <Icon icon=*icon class=cn!("h-8 w-8", *glyph) />
                                            </div>
                                            <h3 class="text-xl font-semibold">{*title}</h3>
                                        </div>
                                        <p class="text-white/70">{*copy}</p>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>

                <div class="mt-16">
                    <h2 class="mb-8 text-center text-3xl font-bold">"System Requirements"</h2>
                    <div class="grid gap-6 md:grid-cols-3">
                        {REQUIREMENTS
                            .iter()
                            .map(|(icon, title, copy)| {
                                view! {
                                    <div class="rounded-xl border border-white/10 bg-white/[0.04] p-6 backdrop-blur-sm">
                                        <div class="mb-3 flex items-center gap-3">
                                            <Icon icon=*icon class="h-5 w-5 text-accent-300" />
                                            <h3 class="text-lg font-semibold">{*title}</h3>
                                        </div>
                                        <p class="text-sm text-white/70">{*copy}</p>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>

                <div class="mt-20 border-t border-white/10 pt-8">
                    <div class="flex flex-col items-center justify-between gap-6 md:flex-row">
                        <div class="text-center md:text-left">
                            <h3 class="mb-2 text-xl font-semibold">"Building Your Own Aircraft?"</h3>
                            <p class="text-sm text-white/70">
                                "Read the infinity-rs developer docs, our Rust toolkit for MSFS 2024 WASM gauges, systems, and SimConnect clients."
                            </p>
                        </div>
                        <button
                            type="button"
                            on:click=move |_| go.run(nav::DOCS.to_string())
                            class="group flex items-center gap-3 rounded-lg bg-accent-600 px-8 py-4 text-white shadow-lg shadow-accent-600/20 transition-all hover:bg-accent-700"
                        >
                            <Icon
                                icon=icons::BOOK_OPEN
                                class="h-5 w-5 transition-transform group-hover:scale-110"
                            />
                            <span class="font-semibold">"Open Developer Docs"</span>
                            <Icon
                                icon=icons::ARROW_RIGHT
                                class="h-4 w-4 transition-transform group-hover:translate-x-1"
                            />
                        </button>
                    </div>
                </div>
            </div>
        </AnimatedPage>
    }
}
