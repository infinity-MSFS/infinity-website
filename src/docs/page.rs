//! The developer-docs article layout. Same `Doc`/`DocSection` data as
//! infinity-ui's `DocPage`, drawn in the site's editorial style: a numbered
//! mono contents list beside hairline-ruled sections instead of glass cards.

use std::time::Duration;

use infinity_ui::prelude::*;

use crate::components::EYEBROW;

/// Sections clear the nav and the docs bar when scrolled to.
const SCROLL_MARGIN: &str = "scroll-mt-[8rem]";

/// The number a section shows: its own, or its position.
fn label(section: &DocSection, idx: usize) -> String {
    section
        .number
        .clone()
        .unwrap_or_else(|| format!("{:02}", idx + 1))
}

#[component]
pub fn DocArticle(doc: Doc, #[prop(into)] base_path: String) -> impl IntoView {
    let sections = StoredValue::new(doc.sections.clone());
    let slugs = Signal::derive(move || {
        sections.with_value(|s| s.iter().map(|x| x.slug.clone()).collect::<Vec<_>>())
    });
    let active = use_active_section(slugs);

    Effect::new(move |_| {
        let hash = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_default();
        let target = hash.trim_start_matches('#').to_string();
        if target.is_empty() {
            return;
        }
        set_timeout(
            move || {
                scroll_to_id(&target, false);
                active.set(Some(target.clone()));
            },
            Duration::from_millis(80),
        );
    });

    let pick = {
        let base_path = base_path.clone();
        Callback::new(move |slug: String| {
            active.set(Some(slug.clone()));
            scroll_to_id(&slug, false);
            if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                let url = format!("{base_path}#{slug}");
                let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
            }
        })
    };

    let scope = doc.scope.clone().unwrap_or_else(|| doc.category.clone());

    view! {
        <div class="mx-auto flex max-w-6xl flex-col gap-12 text-white xl:flex-row xl:gap-16">
            <Contents
                sections=sections.get_value()
                active=Signal::derive(move || active.get())
                on_pick=pick
            />

            // Only the article replays the page-in animation; the contents
            // list is navigation and stays put across a navigation.
            <AnimatedPage class="min-w-0 flex-1">
                <header class="mb-16 max-w-3xl">
                    <p class=cn!("mb-8 flex flex-wrap items-center gap-x-3", EYEBROW)>
                        <span>{scope}</span>
                        <span class="text-white/25">"/"</span>
                        <span class="text-accent-400/80">{doc.eyebrow.clone()}</span>
                    </p>
                    <h1 class="text-4xl leading-[1.05] font-extralight tracking-tight md:text-6xl">
                        {doc.title.clone()}
                    </h1>
                    {doc
                        .tagline
                        .clone()
                        .map(|t| {
                            view! {
                                <p class="mt-8 max-w-2xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65">
                                    {t}
                                </p>
                            }
                        })}
                    {(!doc.tags.is_empty())
                        .then(|| {
                            view! {
                                <p class="mt-6 font-mono text-[11px] text-white/40">
                                    {doc.tags.join("  /  ")}
                                </p>
                            }
                        })}
                </header>

                <div class="space-y-20">
                    {doc
                        .sections
                        .iter()
                        .cloned()
                        .enumerate()
                        .map(|(idx, section)| {
                            let slug = section.slug.clone();
                            let is_active = Signal::derive(move || {
                                active.get().as_deref() == Some(slug.as_str())
                            });
                            view! {
                                <Section
                                    number=label(&section, idx)
                                    section=section
                                    base_path=base_path.clone()
                                    active=is_active
                                />
                            }
                        })
                        .collect_view()}
                </div>
            </AnimatedPage>
        </div>
    }
}

#[component]
fn Contents(
    sections: Vec<DocSection>,
    #[prop(into)] active: Signal<Option<String>>,
    on_pick: Callback<String>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let entries = sections
        .into_iter()
        .enumerate()
        .map(|(idx, s)| {
            let haystack = [
                Some(s.title.as_str()),
                s.summary.as_deref(),
                Some(s.slug.as_str()),
                s.number.as_deref(),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
            (label(&s, idx), s.slug, s.title, haystack)
        })
        .collect::<Vec<_>>();
    let entries = StoredValue::new(entries);
    let filtered = Signal::derive(move || {
        let needle = query.get().trim().to_lowercase();
        entries.with_value(|all| {
            all.iter()
                .filter(|(.., hay)| needle.is_empty() || hay.contains(&needle))
                .cloned()
                .collect::<Vec<_>>()
        })
    });

    view! {
        <aside class="w-full xl:sticky xl:top-[8.5rem] xl:max-h-[calc(100vh-10rem)] xl:w-56 xl:shrink-0 xl:self-start">
            <p class=cn!("mb-4", EYEBROW)>"On this page"</p>
            <input
                type="search"
                placeholder="Filter sections"
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
                class="mb-4 w-full border-b border-white/15 bg-transparent pb-2 font-mono text-xs text-white placeholder:text-white/30 focus:border-accent-400/60 focus:outline-none"
            />
            <nav class="max-h-[60vh] overflow-auto border-l border-white/10">
                <For each=move || filtered.get() key=|(_, slug, ..)| slug.clone() let:entry>
                    {
                        let (number, slug, title, _) = entry;
                        let this = slug.clone();
                        let is_active = Signal::derive(move || {
                            active.get().as_deref() == Some(this.as_str())
                        });
                        view! {
                            <button
                                type="button"
                                on:click=move |_| on_pick.run(slug.clone())
                                class=move || {
                                    cn!(
                                        "-ml-px flex w-full items-baseline gap-3 border-l py-1.5 pl-4 text-left text-sm transition-colors",
                                        if is_active.get() {
                                            "border-accent-400 text-white"
                                        } else {
                                            "border-transparent font-light text-white/50 hover:text-white/85"
                                        },
                                    )
                                }
                            >
                                <span class=move || {
                                    cn!(
                                        "font-mono text-[10px]",
                                        if is_active.get() { "text-accent-400" } else { "text-white/30" },
                                    )
                                }>{number}</span>
                                <span class="min-w-0 truncate">{title}</span>
                            </button>
                        }
                    }
                </For>
                <Show when=move || filtered.get().is_empty()>
                    <p class="py-3 pl-4 font-mono text-[11px] text-white/30">"No sections match."</p>
                </Show>
            </nav>
        </aside>
    }
}

#[component]
fn Section(
    number: String,
    section: DocSection,
    base_path: String,
    active: Signal<bool>,
) -> impl IntoView {
    let (copied, copy) = use_clipboard(Duration::from_millis(1800));
    let href = format!("{base_path}#{}", section.slug);
    let render = section.render.clone();

    let copy_link = move |_| {
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();
        copy(format!("{origin}{href}"));
        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&href));
        }
    };

    view! {
        <section id=section.slug.clone() class=SCROLL_MARGIN>
            <div class=cn!("mb-6 flex items-center gap-4", EYEBROW)>
                <span class=move || {
                    if active.get() { "text-accent-400" } else { "text-accent-400/60" }
                }>{number}</span>
                <span class="hidden truncate sm:inline">{format!("#{}", section.slug)}</span>
                <span class="h-px flex-1 bg-white/10"></span>
                <button
                    type="button"
                    on:click=copy_link
                    class="inline-flex items-center gap-1.5 uppercase transition-colors hover:text-accent-300"
                >
                    {move || {
                        if copied.get() {
                            view! { <Icon icon=icons::CHECK class="h-3 w-3 text-accent-400" /> }.into_any()
                        } else {
                            view! { <Icon icon=icons::LINK_2 class="h-3 w-3" /> }.into_any()
                        }
                    }}
                    {move || if copied.get() { "Copied" } else { "Copy link" }}
                </button>
            </div>
            <h2 class="text-2xl font-light tracking-tight md:text-3xl">{section.title.clone()}</h2>
            {section
                .summary
                .clone()
                .map(|s| {
                    view! {
                        <p class="mt-3 max-w-3xl text-sm leading-relaxed font-light text-white/55">
                            {s}
                        </p>
                    }
                })}
            <div class="mt-6 space-y-5 text-[15px] leading-7 font-light text-white/72">
                {move || render()}
            </div>
        </section>
    }
}

