use infinity_ui::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::EYEBROW;
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
                <span class=cn!("hidden shrink-0 lg:inline", EYEBROW)>
                    "infinity-rs "
                    <span class="text-white/25">"/"</span>
                    " Docs"
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
                    class="ml-auto inline-flex shrink-0 items-center gap-1.5 font-mono text-[11px] text-white/45 transition-colors hover:text-accent-300"
                >
                    <Icon icon=icons::GITHUB class="h-3.5 w-3.5" />
                    <span class="hidden sm:inline">"infinity-rs"</span>
                    <Icon icon=icons::ARROW_UP_RIGHT class="hidden h-3 w-3 sm:inline" />
                </a>
            </StickyBar>

            <div class="px-6 pt-10 pb-16 md:px-10 md:pt-14">
                {move || {
                    let current = slug.get();
                    library
                        .with_value(|docs| {
                            docs.iter().find(|doc| doc.slug == current).cloned()
                        })
                        .map(|doc| {
                            let base = format!("{}/{}", nav::DEVELOPER_DOCS, doc.slug);
                            view! { <docs::DocArticle doc=doc base_path=base /> }
                        })
                }}
            </div>
        </div>
    }
}
