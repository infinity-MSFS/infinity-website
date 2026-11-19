//! This site's glue for the manual reader: maps the route onto
//! `ManualReader` and decides who may read each manual. Everything about the
//! manuals themselves lives in the rest of the module.

use infinity_ui::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map};

use super::links::{MANUAL_BASE_PATH, SiteManual, find_site_manual, is_released, manual_path};
use super::load::load_manual;
use super::reader::ManualReader;
use super::types::Manual;
use crate::auth::use_auth;
use crate::components::EYEBROW;
use crate::state::use_catalog;

#[derive(Clone, PartialEq)]
enum Gate {
    Missing,
    Loading,
    Unavailable(String),
    Open(&'static str),
}

#[component]
pub fn ManualPage() -> impl IntoView {
    let params = use_params_map();
    let auth = use_auth();
    let catalog = use_catalog();
    let catalog_loading = catalog.0.loading();
    let error = catalog.0.error;
    let navigate = use_navigate();

    let manual_id = Signal::derive(move || params.read().get("manual").unwrap_or_default());
    let chapter_slug = Signal::derive(move || params.read().get("chapter"));
    let entry = move || manual_id.with(|id| find_site_manual(id));

    // A manual goes public with its aircraft, unless it is still a draft.
    // Until then only developers can read it, the same rule as the aircraft
    // detail page. Its content is fetched only once the viewer may read it.
    let released =
        Signal::derive(move || entry().is_some_and(|entry| is_released(entry, &catalog)));

    let gate = Memo::new(move |_| {
        let Some(entry) = entry() else {
            return Gate::Missing;
        };
        if auth.loading.get() {
            return Gate::Loading;
        }
        if !auth.entitlements.get().is_developer() {
            if catalog_loading.get() {
                return Gate::Loading;
            }
            if let Some(message) = error.get() {
                return Gate::Unavailable(message);
            }
            if !released.get() {
                return Gate::Missing;
            }
        }
        Gate::Open(entry.id)
    });

    // The reader moves the page to the right section itself, so the router
    // mustn't scroll.
    let on_navigate = Callback::new(move |to: String| {
        navigate(
            &to,
            NavigateOptions {
                scroll: false,
                ..Default::default()
            },
        )
    });

    view! {
        {move || match gate.get() {
            Gate::Missing => view! { <NotFound /> }.into_any(),
            Gate::Loading => view! { <Loading /> }.into_any(),
            Gate::Unavailable(message) => {
                view! {
                    <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
                        <p class=cn!("mb-6", EYEBROW)>"Infinity / Manuals"</p>
                        <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                            "Catalog "
                            <span class="font-semibold text-accent-300">"unavailable"</span>
                        </h1>
                        <p class="max-w-md border-l border-red-400/60 pl-4 font-light text-red-200">
                            {message}
                        </p>
                    </div>
                }
                    .into_any()
            }
            Gate::Open(id) => {
                let Some(entry) = find_site_manual(id) else {
                    return view! { <NotFound /> }.into_any();
                };
                let preview = Signal::derive(move || !released.get() && !catalog_loading.get());
                view! {
                    <LoadedManual
                        entry=entry
                        chapter_slug=chapter_slug
                        on_navigate=on_navigate
                        preview=preview
                    />
                }
                    .into_any()
            }
        }}
    }
}

/// Fetches the manual's content, then shows the reader.
#[component]
fn LoadedManual(
    entry: &'static SiteManual,
    chapter_slug: Signal<Option<String>>,
    on_navigate: Callback<String>,
    /// A developer is reading a manual the public can't see yet.
    preview: Signal<bool>,
) -> impl IntoView {
    let loaded = RwSignal::new(None::<Result<&'static Manual, String>>);
    spawn_local(async move {
        let result = load_manual(entry.id).await;
        if let Err(problem) = &result {
            web_sys::console::error_1(&format!("[manual {}] {problem}", entry.id).into());
        }
        // The page may have moved on while the file loaded.
        loaded.try_set(Some(result));
    });

    let names: Vec<&str> = entry.products.iter().map(|p| p.aircraft).collect();
    let aircraft_names = match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        Some((last, _)) => (*last).to_string(),
        None => String::new(),
    };
    let banner = if entry.draft {
        "This manual is a draft and isn't visible to public users yet.".to_string()
    } else {
        format!("Not visible to public users until the {aircraft_names} is on the storefront.")
    };

    move || match loaded.get() {
        None => view! { <Loading /> }.into_any(),
        Some(Err(_)) => view! { <LoadFailed /> }.into_any(),
        Some(Ok(manual)) => {
            let banner = banner.clone();
            view! {
                // The navbar is a 4rem sticky bar inside the app's scroll container.
                <div class="relative z-10 min-h-screen text-white" style="--manual-top-offset: 4rem">
                    <Show when=move || preview.get()>
                        <p class="mx-4 mt-6 flex flex-wrap items-baseline gap-x-3 gap-y-1 border-l border-amber-400/60 pl-4 text-sm font-light text-amber-200/90 md:mx-8">
                            <span class="font-mono text-[11px] tracking-[0.25em] text-amber-300 uppercase">
                                "Developer preview"
                            </span>
                            {banner.clone()}
                        </p>
                    </Show>
                    <ManualReader
                        manual=manual
                        base_path=manual_path(manual.id)
                        chapter_slug=chapter_slug
                        on_navigate=on_navigate
                        asset_base=format!("/manual-assets/{}", manual.id)
                    />
                </div>
            }
                .into_any()
        }
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! {
        <div class=cn!("flex min-h-screen items-center justify-center", EYEBROW)>"Loading…"</div>
    }
}

#[component]
fn LoadFailed() -> impl IntoView {
    view! {
        <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
            <p class=cn!("mb-6", EYEBROW)>"Infinity / Manuals"</p>
            <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                "The manual "
                <span class="font-semibold text-accent-300">"didn't load"</span>
            </h1>
            <p class="mb-8 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                "Check your connection and try again. If the site was just updated, reloading fetches the new version."
            </p>
            <button
                type="button"
                on:click=move |_| {
                    let _ = window().location().reload();
                }
                class="group inline-flex items-center gap-2 self-start text-lg font-medium transition-colors hover:text-accent-300"
            >
                <Icon icon=icons::REFRESH_CW class="h-4 w-4" />
                "Reload"
            </button>
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
            <p class=cn!("mb-6", EYEBROW)>"Infinity / Manuals"</p>
            <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                "Manual "
                <span class="font-semibold text-accent-300">"not found"</span>
            </h1>
            <p class="mb-8 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                "This manual doesn't exist or isn't available."
            </p>
            <a
                href=MANUAL_BASE_PATH
                class="group inline-flex items-center gap-2 self-start text-lg font-medium transition-colors hover:text-accent-300"
            >
                <Icon
                    icon=icons::ARROW_LEFT
                    class="h-4 w-4 transition-transform group-hover:-translate-x-1"
                />
                "Back to Flight Manuals"
            </a>
        </div>
    }
}
