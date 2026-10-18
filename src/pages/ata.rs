//! TODO: Implement ATA page; we will come to this when the 10 drops

use infinity_ui::prelude::*;

use crate::auth::use_auth;
use crate::components::EYEBROW;
use crate::nav;

#[component]
pub fn AtaDocuments() -> impl IntoView {
    let auth = use_auth();

    view! {
        <Show
            when=move || !auth.loading.get()
            fallback=|| {
                view! {
                    <div class=cn!("flex min-h-screen items-center justify-center", EYEBROW)>
                        "Loading…"
                    </div>
                }
            }
        >
            <Show
                when=move || auth.entitlements.get().is_developer()
                fallback=|| view! { <NotAvailable /> }
            >
                <AnimatedPage class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
                    <p class=cn!("mb-6 flex gap-3", EYEBROW)>
                        "Infinity / Docs"
                        <span class="text-amber-300/90">"Developer preview"</span>
                    </p>
                    <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                        "Aircraft "
                        <span class="font-semibold text-accent-300">"documents."</span>
                    </h1>
                    <p class="max-w-xl border-l border-accent-400/40 pl-5 font-light text-white/60">
                        "The ATA chapters are still being imported. This surface is unlocked for staff so the renderer can be checked against real content as it lands."
                    </p>
                </AnimatedPage>
            </Show>
        </Show>
    }
}

/// Editorial "nothing here" screen with a single way back.
#[component]
fn Missing(
    eyebrow: &'static str,
    message: &'static str,
    back_label: &'static str,
    back_to: &'static str,
) -> impl IntoView {
    let go = nav::use_go();

    view! {
        <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
            <p class=cn!("mb-6", EYEBROW)>{eyebrow}</p>
            <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                "Page "
                <span class="font-semibold text-accent-300">"not found."</span>
            </h1>
            <p class="mb-8 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                {message}
            </p>
            <button
                type="button"
                on:click=move |_| go.run(back_to.to_string())
                class="group inline-flex items-center gap-2 self-start text-lg font-medium transition-colors hover:text-accent-300"
            >
                <Icon
                    icon=icons::ARROW_LEFT
                    class="h-4 w-4 transition-transform group-hover:-translate-x-1"
                />
                {back_label}
            </button>
        </div>
    }
}

#[component]
fn NotAvailable() -> impl IntoView {
    view! {
        <Missing
            eyebrow="Infinity / Docs"
            message="This page isn't available yet."
            back_label="Back to Docs"
            back_to=nav::DOCS
        />
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <Missing
            eyebrow="Infinity / 404"
            message="That page doesn't exist. It may have moved, or the link may be out of date."
            back_label="Back to Home"
            back_to=nav::HOME
        />
    }
}
