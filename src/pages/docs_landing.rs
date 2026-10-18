use infinity_ui::prelude::*;

use crate::auth::use_auth;
use crate::components::{EYEBROW, HoverRule, SectionLabel};
use crate::manual::links::{SITE_MANUALS, manual_path};
use crate::nav;

const REPO_URL: &str = "https://github.com/infinity-MSFS/infinity-rs";

#[component]
pub fn DocsLanding() -> impl IntoView {
    let go = nav::use_go();
    let auth = use_auth();
    let staff = Signal::derive(move || auth.entitlements.get().is_developer());

    view! {
        <AnimatedPage class="relative z-10 min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <header class="mb-16 max-w-3xl">
                    <p class=cn!("mb-10", EYEBROW)>"Infinity / Docs"</p>
                    <h1 class="text-5xl leading-[1.02] font-extralight tracking-tight md:text-7xl">
                        "Infinity"
                        <br />
                        <span class="font-semibold text-accent-300">"Documentation."</span>
                    </h1>
                    <p class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg">
                        "Two parallel surfaces — aircraft technical references for pilots, and developer documentation for the people building gauges, systems, and tools against our Rust toolkit."
                    </p>
                </header>

                <SectionLabel num="01" label="Pick a destination" />
                <div class="border-b border-white/10">
                    <DocRow
                        status="Available now"
                        title="Developer Docs"
                        tagline="infinity-rs — our Rust toolkit for building MSFS 2024 WASM gauges, systems, and SimConnect clients. Comprehensive getting-started guide plus per-module reference pages."
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
                    // The ATA surface is unreleased, so it shows as "coming
                    // soon" until the viewer holds the developer role, at
                    // which point the same row opens it.
                    {move || {
                        let bullets = vec![
                            "Browse by ATA chapter (24 Electrical, 27 Flight Controls, 29 Hydraulics, …)"
                                .to_string(),
                            "Detailed system descriptions with diagrams and schematics".to_string(),
                            "Normal and abnormal procedures for pilots".to_string(),
                        ];
                        let tagline = "ATA-organised technical manuals for our aircraft — system descriptions, schematics, normal/abnormal procedures, and reference tables.";
                        if staff.get() {
                            view! {
                                <DocRow
                                    status="Developer preview"
                                    dev=true
                                    title="Aircraft Documents"
                                    tagline=tagline
                                    bullets=bullets
                                    cta="Browse ATA chapters"
                                    on_click=Callback::new(move |()| go.run(nav::ATA.to_string()))
                                />
                            }
                                .into_any()
                        } else {
                            view! {
                                <DocRow
                                    status="Coming soon"
                                    title="Aircraft Documents"
                                    tagline=tagline
                                    bullets=bullets
                                    cta="Available soon"
                                />
                            }
                                .into_any()
                        }
                    }}
                    // Web flight manuals are drafts, so unlike the ATA row
                    // there is no "coming soon" teaser: the row doesn't exist
                    // for anyone without the developer role.
                    <Show when=move || staff.get()>
                        {SITE_MANUALS
                            .iter()
                            .map(|manual| {
                                let aircraft = manual
                                    .products
                                    .iter()
                                    .map(|p| p.aircraft)
                                    .collect::<Vec<_>>()
                                    .join(" / ");
                                let path = manual_path(manual.id);
                                view! {
                                    <DocRow
                                        status="Developer preview"
                                        dev=true
                                        title="Flight Manual"
                                        tagline=format!(
                                            "The {aircraft} flight manual, readable in the browser — systems, limitations and procedures for the whole family.",
                                        )
                                        bullets=vec![
                                            "Limitations, normal, abnormal and emergency procedures"
                                                .to_string(),
                                            "Full-text search and deep links to any section".to_string(),
                                            "Draft — chapters are still being written".to_string(),
                                        ]
                                        cta="Open the manual"
                                        on_click=Callback::new(move |()| go.run(path.clone()))
                                    />
                                }
                            })
                            .collect_view()}
                    </Show>
                </div>

                <footer class="mt-10 flex flex-wrap items-center justify-between gap-x-6 gap-y-2 font-mono text-[11px] text-white/35">
                    <span>"Looking for something else?"</span>
                    <span class="flex gap-6">
                        <a href=nav::HOME class="transition-colors hover:text-accent-300">
                            "Home page"
                        </a>
                        <a
                            href=REPO_URL
                            target="_blank"
                            rel="noreferrer"
                            class="inline-flex items-center gap-1.5 transition-colors hover:text-accent-300"
                        >
                            <Icon icon=icons::GITHUB class="h-3.5 w-3.5" />
                            "infinity-rs source"
                            <Icon icon=icons::ARROW_UP_RIGHT class="h-3 w-3" />
                        </a>
                    </span>
                </footer>
            </div>
        </AnimatedPage>
    }
}

/// One destination. Without `on_click` the row is inert and dimmed, which is
/// how an unreleased surface shows.
#[component]
fn DocRow(
    status: &'static str,
    #[prop(optional)] dev: bool,
    title: &'static str,
    #[prop(into)] tagline: String,
    bullets: Vec<String>,
    cta: &'static str,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let enabled = on_click.is_some();
    view! {
        <button
            type="button"
            disabled=!enabled
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(())
                }
            }
            class=cn!(
                "group relative grid w-full gap-4 border-t border-white/10 py-8 text-left md:grid-cols-[16rem_1fr_auto] md:gap-10",
                if enabled { "transition-colors hover:bg-white/[0.03]" } else { "cursor-default opacity-55" },
            )
        >
            {enabled.then(|| view! { <HoverRule /> })}
            <div>
                <p class=cn!(
                    "mb-2 font-mono text-[11px] tracking-[0.2em] uppercase",
                    if dev { "text-amber-300/90" } else if enabled { "text-accent-400/80" } else { "text-white/40" },
                )>{status}</p>
                <h2 class="text-2xl font-medium tracking-tight">{title}</h2>
            </div>
            <div>
                <p class="mb-4 text-sm leading-relaxed font-light text-white/65">{tagline}</p>
                <ul class="space-y-1.5">
                    {bullets
                        .into_iter()
                        .map(|b| {
                            view! {
                                <li class="flex gap-3 text-sm font-light text-white/50">
                                    <span class="font-mono text-white/25">"—"</span>
                                    {b}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
            <span class="flex items-center gap-2 self-start text-sm font-medium whitespace-nowrap md:self-center">
                {cta}
                {enabled
                    .then(|| {
                        view! {
                            <Icon
                                icon=icons::ARROW_RIGHT
                                class="h-4 w-4 transition-transform group-hover:translate-x-1"
                            />
                        }
                    })}
            </span>
        </button>
    }
}
