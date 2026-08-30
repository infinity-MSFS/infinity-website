//! TODO: Implement ATA page; we will come to this when the 10 drops

use infinity_ui::prelude::*;

use crate::auth::use_auth;
use crate::nav;

#[component]
pub fn AtaDocuments() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Show
            when=move || !auth.loading.get()
            fallback=|| {
                view! {
                    <div class="flex min-h-screen items-center justify-center text-white/60">
                        "Loading…"
                    </div>
                }
            }
        >
            <Show
                when=move || auth.entitlements.get().is_developer()
                fallback=|| view! { <NotAvailable /> }
            >
                <AnimatedPage class="px-4 py-16 md:px-8">
                    <div class="mx-auto max-w-3xl">
                        <EmptyState
                            icon=icons::PLANE
                            title="Aircraft documents"
                            description="The ATA chapters are still being imported. This surface is unlocked for staff so the renderer can be checked against real content as it lands."
                        />
                    </div>
                </AnimatedPage>
            </Show>
        </Show>
    }
}

#[component]
fn NotAvailable() -> impl IntoView {
    let go = nav::use_go();

    view! {
        <div class="flex min-h-screen flex-col items-center justify-center px-4 text-white">
            <h1 class="mb-2 text-3xl font-bold">"Page not found"</h1>
            <p class="mb-6 max-w-md text-center text-white/60">
                "This page isn't available yet."
            </p>
            <button
                type="button"
                on:click=move |_| go.run(nav::DOCS.to_string())
                class="rounded-lg bg-accent-600 px-6 py-2 text-white hover:bg-accent-700"
            >
                "Back to Docs"
            </button>
        </div>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    let go = nav::use_go();

    view! {
        <div class="flex min-h-screen flex-col items-center justify-center px-4 text-white">
            <h1 class="mb-2 text-3xl font-bold">"Page not found"</h1>
            <p class="mb-6 max-w-md text-center text-white/60">
                "That page doesn't exist. It may have moved, or the link may be out of date."
            </p>
            <button
                type="button"
                on:click=move |_| go.run(nav::HOME.to_string())
                class="rounded-lg bg-accent-600 px-6 py-2 text-white hover:bg-accent-700"
            >
                "Back to Home"
            </button>
        </div>
    }
}
