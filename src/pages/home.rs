use infinity_ui::prelude::*;

use crate::brand::InfinityMark;
use crate::nav;

const HEADLINE: &str = "High-fidelity aircraft.";
const HEADLINE_ACCENT: &str = "Effortless management.";
const DESCRIPTION: &str = "Infinity is the distribution platform and aircraft manager from Aero Dynamics — shipping our premium add-ons alongside hand-picked freeware from the wider community. Engineered in Rust with a strict push for safe, reliable code across everything we build.";

const DESTINATIONS: &[(&str, &str, &str, &str)] = &[
    (
        "01",
        "Aircraft",
        "Premium add-ons for MSFS 2024",
        nav::AIRCRAFT,
    ),
    (
        "02",
        "The Manager",
        "Install, update, and keep it all in sync",
        nav::MANAGER,
    ),
    (
        "03",
        "Developer Docs",
        "Our open-source developer tooling for MSFS 2024",
        nav::DOCS,
    ),
];

const SPECS: &[&str] = &["Built in pure Rust"];

#[component]
pub fn Home() -> impl IntoView {
    let go = nav::use_go();

    view! {
        <div class="relative h-[calc(100dvh-4rem)] w-full overflow-x-hidden overflow-y-auto">
            <div
                aria-hidden="true"
                class="pointer-events-none absolute top-1/2 -right-[10%] hidden w-[68vw] max-w-[1000px] -translate-y-1/2 opacity-[0.06] lg:block"
            >
                <InfinityMark class="h-auto w-full" glow="none" />
            </div>

            <div class="relative z-10 mx-auto flex min-h-full max-w-6xl flex-col justify-between px-6 py-10 md:px-10 md:py-14">
                <Reveal
                    y=8.0
                    duration=0.6
                    class="flex items-center gap-4 font-mono text-[11px] tracking-[0.25em] text-white/50 uppercase"
                >
                    <span>"By Aero Dynamics"</span>
                </Reveal>

                <div class="my-14 max-w-3xl">
                    <Reveal
                        tag="h1"
                        y=16.0
                        duration=0.7
                        delay=0.1
                        class="text-5xl leading-[1.02] font-extralight tracking-tight text-white md:text-7xl"
                    >
                        {HEADLINE}
                        <br />
                        <span class="font-semibold text-accent-300">{HEADLINE_ACCENT}</span>
                    </Reveal>
                    <Reveal
                        y=12.0
                        duration=0.7
                        delay=0.25
                        class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg"
                    >
                        {DESCRIPTION}
                    </Reveal>
                </div>

                <Reveal y=12.0 duration=0.7 delay=0.4>
                    <nav class="grid border-t border-white/15 sm:grid-cols-3">
                        {DESTINATIONS
                            .iter()
                            .map(|(num, label, summary, route)| {
                                view! {
                                    <button
                                        type="button"
                                        on:click=move |_| go.run(route.to_string())
                                        class="group relative flex flex-col items-start gap-1 border-b border-white/10 py-5 text-left transition-colors hover:bg-white/[0.03] sm:border-r sm:border-b-0 sm:px-6 sm:first:pl-0 sm:last:border-r-0"
                                    >
                                        <span class="absolute -top-px left-0 h-px w-0 bg-accent-400 transition-all duration-300 group-hover:w-full"></span>
                                        <span class="font-mono text-[11px] text-accent-400/80">{*num}</span>
                                        <span class="flex items-center gap-2 text-lg font-medium text-white">
                                            {*label}
                                            <Icon
                                                icon=icons::ARROW_RIGHT
                                                class="h-4 w-4 -translate-x-1 opacity-0 transition-all group-hover:translate-x-0 group-hover:opacity-100"
                                            />
                                        </span>
                                        <span class="text-sm font-light text-white/45">{*summary}</span>
                                    </button>
                                }
                            })
                            .collect_view()}
                    </nav>

                    <div class="mt-6 flex flex-wrap items-center justify-between gap-x-6 gap-y-2 font-mono text-[11px] text-white/35">
                        <span>{SPECS.join("  /  ")}</span>
                        <a
                            href=nav::GITHUB_ORG
                            target="_blank"
                            rel="noreferrer noopener"
                            class="inline-flex items-center gap-1.5 transition-colors hover:text-accent-300"
                        >
                            <Icon icon=icons::GITHUB class="h-3.5 w-3.5" />
                            "github.com/infinity-MSFS"
                            <Icon icon=icons::ARROW_UP_RIGHT class="h-3 w-3" />
                        </a>
                    </div>
                </Reveal>
            </div>
        </div>
    }
}
