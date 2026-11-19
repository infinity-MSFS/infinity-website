//! `/manual`: the Flight Manual picker the docs page links to. One row per
//! web manual. A row opens its manual when the viewer may read it (public
//! once its aircraft is on the storefront, developers always) and says when
//! it's coming otherwise. Reads only the manual table, never manual content.

use infinity_ui::prelude::*;

use super::links::{
    SITE_MANUALS, SiteManual, aircraft_on_sale, aircraft_query, is_released, manual_path,
};
use super::shared::FOCUS;
use crate::auth::use_auth;
use crate::components::{EYEBROW, HoverRule, SectionLabel};
use crate::nav;
use crate::state::{Catalog, use_catalog};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Available,
    /// Readable by developers only.
    Preview,
    Soon,
    /// Still finding out whether the viewer may read it.
    Checking,
}

#[component]
pub fn ManualIndex() -> impl IntoView {
    let auth = use_auth();
    let catalog = use_catalog();
    let catalog_loading = catalog.0.loading();

    let status_of = move |manual: &SiteManual| {
        if is_released(manual, &catalog) {
            Status::Available
        } else if auth.entitlements.get().is_developer() {
            Status::Preview
        } else if auth.loading.get() || catalog_loading.get() {
            Status::Checking
        } else {
            Status::Soon
        }
    };

    view! {
        <AnimatedPage class="relative z-10 min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <header class="mb-16 max-w-3xl">
                    <p class=cn!("mb-10 flex flex-wrap gap-x-3", EYEBROW)>
                        <a href=nav::DOCS class=cn!("uppercase transition-colors hover:text-accent-300", FOCUS)>
                            "Infinity / Docs"
                        </a>
                        <span class="text-white/25">"/"</span>
                        <span class="text-white/80">"Flight Manual"</span>
                    </p>
                    <h1 class="text-5xl leading-[1.02] font-extralight tracking-tight md:text-7xl">
                        "Flight"
                        <br />
                        <span class="font-semibold text-accent-300">"Manuals."</span>
                    </h1>
                    <p class="mt-8 max-w-xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg">
                        "Choose your aircraft. Each manual covers its systems, procedures, limitations and handling, and shows only what applies to the variant you pick inside it."
                    </p>
                </header>

                <SectionLabel num="01" label="Choose a manual" />
                <ul class="list-none border-b border-white/10">
                    {SITE_MANUALS
                        .iter()
                        .map(|manual| {
                            view! {
                                {move || {
                                    view! {
                                        <ManualRow manual=manual status=status_of(manual) catalog=catalog />
                                    }
                                }}
                            }
                        })
                        .collect_view()}
                </ul>

                <p class="mt-10 font-mono text-[11px] text-white/35">
                    "Every aircraft also carries its manual in the cockpit EFB, under the Aircraft tab."
                </p>
            </div>
        </AnimatedPage>
    }
}

#[component]
fn ManualRow(manual: &'static SiteManual, status: Status, catalog: Catalog) -> impl IntoView {
    let open = matches!(status, Status::Available | Status::Preview);
    let path = manual_path(manual.id);
    let (status_text, status_class) = match status {
        Status::Available => ("Available now", "text-accent-400/80"),
        Status::Preview => ("Developer preview", "text-amber-300/90"),
        Status::Soon => ("Coming soon", "text-white/40"),
        Status::Checking => ("…", "text-white/30"),
    };
    // A manual for an aircraft people can already buy is coming to the web;
    // the full manual is in the sim meanwhile.
    let waiting = if aircraft_on_sale(manual, &catalog) {
        "In the cockpit EFB for now"
    } else {
        "Publishes with the aircraft"
    };

    view! {
        <li class=cn!(
            "group relative grid gap-4 border-t border-white/10 py-8 md:grid-cols-[16rem_1fr_auto] md:gap-10 md:px-2",
            if open { "transition-colors hover:bg-white/[0.03]" } else { "opacity-60" },
        )>
            {open.then(|| view! { <HoverRule /> })}
            <div>
                <p class=cn!("mb-2 font-mono text-[11px] tracking-[0.2em] uppercase", status_class)>
                    {status_text}
                </p>
                <h2 class="text-2xl font-medium tracking-tight">
                    {if open {
                        // The whole row is the link; the aircraft links sit above it.
                        view! {
                            <a href=path.clone() class=cn!("after:absolute after:inset-0", FOCUS)>
                                {manual.name}
                            </a>
                        }
                            .into_any()
                    } else {
                        manual.name.into_any()
                    }}
                </h2>
            </div>
            <div>
                <p class="mb-4 text-sm leading-relaxed font-light text-white/65">{manual.description}</p>
                <ul class="flex list-none flex-wrap gap-x-5 gap-y-2">
                    {manual
                        .covers
                        .iter()
                        .map(|&aircraft| {
                            let chip = "font-mono text-[11px] tracking-[0.2em] uppercase";
                            if open {
                                view! {
                                    <li>
                                        <a
                                            href=format!("{path}{}", aircraft_query(aircraft))
                                            class=cn!(
                                                chip,
                                                "relative z-10 border-b border-white/15 text-white/65 transition-colors hover:border-accent-400/60 hover:text-accent-300",
                                                FOCUS,
                                            )
                                            aria-label=format!("Open the {} manual on the {aircraft}", manual.name)
                                        >
                                            {aircraft}
                                        </a>
                                    </li>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <li class=cn!(chip, "text-white/45")>{aircraft}</li>
                                }
                                    .into_any()
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
            <span class="flex items-center gap-2 self-start text-sm font-medium whitespace-nowrap md:self-center">
                {if open {
                    view! {
                        "Open the manual"
                        <Icon
                            icon=icons::ARROW_RIGHT
                            class="h-4 w-4 transition-transform group-hover:translate-x-1"
                        />
                    }
                        .into_any()
                } else {
                    view! { <span class="font-light text-white/50">{waiting}</span> }.into_any()
                }}
            </span>
        </li>
    }
}
