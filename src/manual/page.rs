//! This site's glue for the manual reader: maps the route onto
//! `ManualReader` and decides who may read each manual. Everything about the
//! manuals themselves lives in the rest of the module.

use infinity_ui::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map};

use super::links::{SITE_MANUALS, manual_path};
use super::manuals;
use super::reader::ManualReader;
use crate::auth::use_auth;
use crate::nav;
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
    let entry = move || manual_id.with(|id| SITE_MANUALS.iter().find(|m| m.id == id.as_str()));

    // A manual goes public with its aircraft. Until one of them is on the
    // storefront, only developers can read it, the same rule as the aircraft
    // detail page.
    let released = Signal::derive(move || {
        entry().is_some_and(|entry| {
            entry.products.iter().any(|product| {
                catalog
                    .by_slug(product.slug)
                    .is_some_and(|p| p.storefront && !p.dev_only)
            })
        })
    });

    let gate = Memo::new(move |_| {
        let Some(manual) = manual_id
            .with(|id| manuals::find(id))
            .filter(|_| entry().is_some())
        else {
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
        Gate::Open(manual.id)
    });

    let aircraft_names = move || {
        let names: Vec<&str> = entry()
            .map(|e| e.products.iter().map(|p| p.aircraft).collect())
            .unwrap_or_default();
        match names.split_last() {
            Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
            Some((last, _)) => (*last).to_string(),
            None => String::new(),
        }
    };

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
            Gate::Loading => {
                view! {
                    <div class="flex min-h-screen items-center justify-center text-white/60">
                        "Loading…"
                    </div>
                }
                    .into_any()
            }
            Gate::Unavailable(message) => {
                view! {
                    <div class="flex min-h-screen flex-col items-center justify-center px-4 text-white">
                        <h1 class="mb-2 text-2xl font-bold">"Catalog unavailable"</h1>
                        <p class="mb-6 max-w-md text-center text-white/60">{message}</p>
                    </div>
                }
                    .into_any()
            }
            Gate::Open(id) => {
                let Some(manual) = manuals::find(id) else {
                    return view! { <NotFound /> }.into_any();
                };
                view! {
                    // The navbar is a 4rem sticky bar inside the app's scroll container.
                    <div class="relative z-10 min-h-screen text-white" style="--manual-top-offset: 4rem">
                        <Show when=move || !released.get() && !catalog_loading.get()>
                            <Alert
                                tone=Tone::Dev
                                icon=icons::FLASK_CONICAL
                                title=format!(
                                    "Developer preview. Not visible to public users until the {} is on the storefront.",
                                    aircraft_names(),
                                )
                                class="mx-4 mt-4 md:mx-8"
                            />
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
        }}
    }
}

#[component]
fn NotFound() -> impl IntoView {
    let go = nav::use_go();

    view! {
        <div class="flex min-h-screen flex-col items-center justify-center px-4 text-white">
            <h1 class="mb-2 text-3xl font-bold">"Manual not found"</h1>
            <p class="mb-6 text-white/60">"This manual doesn't exist or isn't available."</p>
            <button
                type="button"
                on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                class="rounded-lg bg-accent-600 px-6 py-2 text-white hover:bg-accent-700"
            >
                "Back to Aircraft"
            </button>
        </div>
    }
}
