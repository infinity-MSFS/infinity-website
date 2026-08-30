use infinity_ui::prelude::*;

use crate::brand::{GLOW_HERO, InfinityMark};
use crate::nav;

const TAGLINE: &str = "For Microsoft Flight Simulator 2024";
const HEADLINE: &str = "High-fidelity aircraft.";
const HEADLINE_ACCENT: &str = "Effortless management.";
const DESCRIPTION: &str = "Infinity is the distribution platform and aircraft manager from Aero Dynamics — shipping our premium add-ons alongside hand-picked freeware from the wider community. Engineered in Rust with a strict push for safe, reliable code across everything we build.";

const CHIPS: &[(IconData, &str)] = &[
    (icons::PLANE, "MSFS 2024"),
    (icons::CPU, "Built in Rust"),
    (icons::CODE_XML, "Open-Source SDK"),
];

#[component]
pub fn Home() -> impl IntoView {
    let go = nav::use_go();

    view! {
        <div class="relative flex h-[calc(100dvh-4rem)] w-full items-center justify-center overflow-x-hidden overflow-y-auto px-4 py-12">
            <div class="relative z-10 flex w-full max-w-4xl flex-col items-center text-center">
                <Reveal
                    y=12.0
                    duration=0.6
                    class="mb-6 inline-flex items-center gap-2 rounded-full border border-accent-400/30 bg-accent-500/10 px-3 py-1 text-xs font-semibold tracking-wider text-accent-300 uppercase backdrop-blur-sm"
                >
                    <span class="h-1.5 w-1.5 rounded-full bg-accent-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"></span>
                    {TAGLINE}
                </Reveal>

                <Reveal y=16.0 duration=0.7 delay=0.05 class="mb-5 flex flex-col items-center">
                    <div class="flex items-start justify-center gap-3">
                        <InfinityMark class="h-14 w-14 md:h-16 md:w-16" glow=GLOW_HERO />
                        <span class="bg-gradient-to-b from-white to-white/70 bg-clip-text text-5xl font-bold tracking-tight text-transparent md:text-6xl">
                            "Infinity"
                        </span>
                    </div>
                    <span class="mt-1 text-[11px] tracking-[0.2em] text-white/40 uppercase">
                        "by Aero Dynamics"
                    </span>
                </Reveal>

                <Reveal
                    tag="h1"
                    y=16.0
                    duration=0.7
                    delay=0.15
                    class="mb-2 max-w-3xl text-3xl leading-tight font-bold md:text-5xl"
                >
                    {HEADLINE}
                    " "
                    <span class="bg-gradient-to-r from-accent-300 to-accent-500 bg-clip-text text-transparent">
                        {HEADLINE_ACCENT}
                    </span>
                </Reveal>

                <div class="mb-8 max-w-2xl">
                    <TextGenerate words=DESCRIPTION class="w-full" />
                </div>

                <Reveal
                    y=12.0
                    duration=0.6
                    delay=0.35
                    class="mb-8 flex flex-col items-center gap-3 sm:flex-row"
                >
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                        class="group flex items-center gap-2 rounded-lg bg-accent-600 px-6 py-3 font-semibold text-white shadow-lg shadow-accent-600/25 transition-colors hover:bg-accent-700"
                    >
                        <Icon icon=icons::PLANE class="h-4 w-4" />
                        "Browse Aircraft"
                        <Icon
                            icon=icons::ARROW_RIGHT
                            class="h-4 w-4 transition-transform group-hover:translate-x-0.5"
                        />
                    </button>
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::MANAGER.to_string())
                        class="flex items-center gap-2 rounded-lg border border-white/15 bg-white/[0.06] px-6 py-3 font-medium text-white backdrop-blur-sm transition-colors hover:border-white/25 hover:bg-white/[0.10]"
                    >
                        <Icon icon=icons::BOOK_OPEN class="h-4 w-4" />
                        "About the Manager"
                    </button>
                    <button
                        type="button"
                        on:click=move |_| go.run(nav::DOCS.to_string())
                        class="flex items-center gap-2 rounded-lg border border-accent-400/30 bg-white/[0.04] px-6 py-3 font-medium text-accent-200 backdrop-blur-sm transition-colors hover:border-accent-400/60 hover:bg-accent-500/10 hover:text-accent-100"
                    >
                        <Icon icon=icons::CODE_XML class="h-4 w-4" />
                        "Developer Docs"
                    </button>
                </Reveal>

                <Reveal
                    y=0.0
                    duration=0.8
                    delay=0.55
                    class="flex flex-wrap items-center justify-center gap-2"
                >
                    {CHIPS
                        .iter()
                        .map(|(icon, label)| {
                            view! {
                                <span class="inline-flex items-center gap-1.5 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-xs text-white/70 backdrop-blur-sm">
                                    <span class="text-accent-300">
                                        <Icon icon=*icon class="h-3.5 w-3.5" />
                                    </span>
                                    {*label}
                                </span>
                            }
                        })
                        .collect_view()}
                </Reveal>
            </div>
        </div>
    }
}
