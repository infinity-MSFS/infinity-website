use infinity_ui::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::docs;
use crate::nav;

const REPO_URL: &str = "https://github.com/infinity-MSFS/infinity-rs";

#[component]
pub fn DeveloperDocs() -> impl IntoView {
    let go = nav::use_go();
    let params = use_params_map();
    let library = StoredValue::new_local(docs::library());

    let slug = Signal::derive(move || {
        let requested = params.read().get("slug").unwrap_or_default();
        library.with_value(|docs| {
            docs.iter()
                .find(|doc| doc.slug == requested)
                .or_else(|| docs.first())
                .map(|doc| doc.slug.clone())
                .unwrap_or_default()
        })
    });

    let groups = Signal::derive(move || library.with_value(|docs| docs::groups(docs)));

    let on_change = Callback::new(move |next: String| {
        go.run(format!("{}/{next}", nav::DEVELOPER_DOCS));
        if let Some(window) = web_sys::window() {
            window.scroll_to_with_x_and_y(0.0, 0.0);
        }
    });

    view! {
        <div class="relative z-10 min-h-screen text-white">
            <StickyBar top="top-16">
                <span class="hidden shrink-0 text-xs font-medium tracking-[0.18em] text-white/35 uppercase lg:inline">
                    "infinity-rs / Docs"
                </span>
                <Select
                    value=slug
                    on_change=on_change
                    groups=groups
                    class="min-w-0 flex-1 lg:w-96 lg:flex-none"
                />
                <a
                    href=REPO_URL
                    target="_blank"
                    rel="noreferrer"
                    aria-label="View infinity-rs on GitHub"
                    class="ml-auto shrink-0 rounded-2xl border border-white/10 bg-black/20 px-3 py-2 text-xs tracking-[0.16em] text-white/60 uppercase transition hover:border-accent-400/25 hover:bg-accent-500/10 hover:text-white"
                >
                    <span class="hidden sm:inline">"GitHub ↗"</span>
                    <span class="sm:hidden">
                        <Icon icon=icons::GITHUB class="h-4 w-4" />
                    </span>
                </a>
            </StickyBar>

            <AnimatedPage class="px-6 pt-6 pb-12 md:px-8 md:pb-12">
                {move || {
                    let current = slug.get();
                    library
                        .with_value(|docs| {
                            docs.iter().find(|doc| doc.slug == current).cloned()
                        })
                        .map(|doc| {
                            let base = format!("{}/{}", nav::DEVELOPER_DOCS, doc.slug);
                            view! { <DocPage doc=doc base_path=base /> }
                        })
                }}
            </AnimatedPage>
        </div>
    }
}
