use infinity_ui::prelude::*;

use crate::auth::use_auth;
use crate::nav;

const REPO_URL: &str = "https://github.com/infinity-MSFS/infinity-rs";

#[component]
pub fn DocsLanding() -> impl IntoView {
    let go = nav::use_go();
    let auth = use_auth();
    let staff = Signal::derive(move || auth.entitlements.get().is_developer());

    view! {
        <AnimatedPage class="relative z-10 min-h-screen text-white">
            <div class="px-6 pt-12 pb-16 md:px-8 md:pt-20 md:pb-24">
                <div class="mx-auto flex max-w-5xl flex-col gap-8">
                    <header class="flex flex-col gap-5">
                        <div class="flex flex-wrap items-center gap-2 text-xs tracking-[0.18em] text-white/42 uppercase">
                            <span class="rounded-full border border-accent-400/30 bg-accent-500/14 px-3 py-1 text-accent-200">
                                <Icon
                                    icon=icons::BOOK_OPEN
                                    class="-mt-0.5 mr-1.5 inline h-3.5 w-3.5"
                                />
                                "Documentation"
                            </span>
                            <span>"Pick a destination"</span>
                        </div>
                        <h1 class="text-4xl font-semibold tracking-tight text-white md:text-5xl">
                            "Infinity Documentation"
                        </h1>
                        <p class="max-w-3xl text-base leading-7 text-white/65 md:text-lg">
                            "Two parallel surfaces — aircraft technical references for pilots, and developer documentation for the people building gauges, systems, and tools against our Rust toolkit."
                        </p>
                    </header>

                    <div class="grid gap-5 lg:grid-cols-2">
                        // The ATA surface is unreleased, so it shows as
                        // "coming soon" until the viewer holds the developer
                        // role, at which point the same tile opens it.
                        {move || {
                            let bullets = vec![
                                "Browse by ATA chapter (24 Electrical, 27 Flight Controls, 29 Hydraulics, …)"
                                    .to_string(),
                                "Detailed system descriptions with diagrams and schematics"
                                    .to_string(),
                                "Normal and abnormal procedures for pilots".to_string(),
                            ];
                            let tagline = "ATA-organised technical manuals for our aircraft — system descriptions, schematics, normal/abnormal procedures, and reference tables.";
                            if staff.get() {
                                view! {
                                    <Tile
                                        title="Aircraft Documents"
                                        eyebrow="Developer preview"
                                        tagline=tagline
                                        icon=icons::PLANE
                                        flag=("Dev".to_string(), Tone::Dev)
                                        bullets=bullets
                                        cta="Browse ATA chapters"
                                        on_click=Callback::new(move |()| {
                                            go.run(nav::ATA.to_string())
                                        })
                                    />
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <Tile
                                        title="Aircraft Documents"
                                        eyebrow="Coming soon"
                                        tagline=tagline
                                        icon=icons::PLANE
                                        disabled=true
                                        flag=("Soon".to_string(), Tone::Warning)
                                        bullets=bullets
                                        cta="Available soon"
                                    />
                                }
                                    .into_any()
                            }
                        }}
                        <Tile
                            title="Developer Docs"
                            eyebrow="Available now"
                            tagline="infinity-rs — our Rust toolkit for building MSFS 2024 WASM gauges, systems, and SimConnect clients. Comprehensive getting-started guide plus per-module reference pages."
                            icon=icons::WRENCH
                            bullets=vec![
                                "Set up the toolchain and ship your first gauge in 5 minutes".to_string(),
                                "Per-module deep dives: vars, comm bus, flow, IO, NanoVG, SimConnect, more"
                                    .to_string(),
                                "Native + WASM build pipelines covered end-to-end".to_string(),
                            ]
                            cta="Read the docs"
                            on_click=Callback::new(move |()| {
                                go.run(format!("{}/getting-started", nav::DEVELOPER_DOCS))
                            })
                        />
                    </div>

                    <footer class="mt-2 text-xs tracking-[0.18em] text-white/35 uppercase">
                        "Looking for something else? See the "
                        <a
                            href=nav::HOME
                            class="text-accent-300/80 underline-offset-4 hover:underline"
                        >
                            "home page"
                        </a>
                        " or "
                        <a
                            href=REPO_URL
                            target="_blank"
                            rel="noreferrer"
                            class="text-accent-300/80 underline-offset-4 hover:underline"
                        >
                            "browse the source on GitHub ↗"
                        </a>
                        "."
                    </footer>
                </div>
            </div>
        </AnimatedPage>
    }
}
