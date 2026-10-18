use std::sync::Arc;
use std::time::Duration;

use infinity_ui::prelude::*;

use super::blocks::block_list;
use super::model::{OutlineChapter, OutlineSection};
use super::shared::{APPLIES, FOCUS, use_manual};
use super::types::ManualChapter;
use crate::components::{EYEBROW, HoverRule};

const TITLE: &str = "text-4xl leading-[1.05] font-extralight tracking-tight text-white md:text-6xl";
const LEDE: &str = "mt-8 max-w-2xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65";
const META: &str = "mb-8 flex flex-wrap items-center gap-x-3 gap-y-2";
/// Clickable hairline row: the parent of a `HoverRule`.
const ROW: &str = "group relative border-t border-white/10 transition-colors hover:bg-white/[0.03]";

/// Mono number and label with a hairline running to the edge, like the site's
/// `SectionLabel` but for labels built at runtime.
fn group_label(num: &'static str, label: String) -> impl IntoView {
    view! {
        <div class=cn!("mb-8 flex items-center gap-4", EYEBROW)>
            <span class="text-accent-400/80">{num}</span>
            <span>{label}</span>
            <span class="h-px flex-1 bg-white/10"></span>
        </div>
    }
}

/// The manual's front page: every chapter, then the introduction.
#[component]
pub fn MainPage(outline: Vec<OutlineChapter>, intro: Vec<Arc<OutlineSection>>) -> impl IntoView {
    let ctx = use_manual();
    let manual = ctx.manual;
    let aircraft = ctx.aircraft.get_untracked();
    let label = ctx.chapter_label;

    view! {
        <header class="max-w-4xl">
            <p class=cn!(META, EYEBROW)>
                <span class="text-accent-400/80">"Manual"</span>
                {manual
                    .revision
                    .map(|r| {
                        view! {
                            <span class="text-white/25">"/"</span>
                            <span>{r}</span>
                        }
                    })}
            </p>
            <h1 class=TITLE>{manual.title}</h1>
            {manual.summary.map(|s| view! { <p class=LEDE>{s}</p> })}
            {aircraft
                .filter(|_| manual.aircraft.len() > 1)
                .map(|aircraft| {
                    view! {
                        <p class="mt-6 text-[0.9375rem] leading-[1.6] font-light text-white/55">
                            "Showing the " <strong class="font-semibold text-accent-300">{aircraft}</strong>
                            ". Switch aircraft with the buttons at the top of the page."
                        </p>
                    }
                })}
        </header>

        <section>
            {group_label("01", format!("{label}s"))}
            <ol class="list-none border-b border-white/10">
                {outline
                    .into_iter()
                    .map(|entry| {
                        let chapter = entry.chapter;
                        let badge = ctx.applies_to(chapter.applies_to);
                        let count = entry.flat.len();
                        let href = ctx.href(Some(chapter.slug), None);
                        view! {
                            <li class=ROW>
                                <HoverRule />
                                <a
                                    class=cn!(
                                        FOCUS,
                                        "grid gap-x-8 gap-y-2 py-6 md:grid-cols-[9rem_minmax(0,1fr)_auto] md:items-baseline md:px-2",
                                    )
                                    href=href.clone()
                                    on:click=ctx.on_link(href)
                                >
                                    <span class="flex flex-col gap-1.5">
                                        <span class="font-mono text-[11px] tracking-[0.25em] text-accent-400/80 uppercase">
                                            {format!("{label} {}", entry.number)}
                                        </span>
                                        {badge.map(|b| view! { <span class=APPLIES>{b}</span> })}
                                    </span>
                                    <span class="flex min-w-0 flex-col gap-2">
                                        <span class="text-xl leading-[1.3] font-light text-white md:text-2xl">
                                            {chapter.title}
                                        </span>
                                        {chapter
                                            .summary
                                            .map(|s| {
                                                view! {
                                                    <span class="max-w-2xl text-sm leading-[1.6] font-light text-white/55">
                                                        {s}
                                                    </span>
                                                }
                                            })}
                                    </span>
                                    <span class="inline-flex items-center gap-3 font-mono text-[11px] text-white/40">
                                        {if count == 1 { "1 topic".to_string() } else { format!("{count} topics") }}
                                        <Icon
                                            icon=icons::ARROW_RIGHT
                                            class="h-4 w-4 text-white/60 transition-transform group-hover:translate-x-1 group-hover:text-accent-300"
                                        />
                                    </span>
                                </a>
                            </li>
                        }
                    })
                    .collect_view()}
            </ol>
        </section>

        {(!intro.is_empty())
            .then(|| {
                view! {
                    <section>
                        {group_label("02", "Introduction".to_string())}
                        <div class="flex flex-col gap-16">
                            {intro
                                .into_iter()
                                .map(|node| view! { <Section chapter_slug=None node=node heading_level=3 /> })
                                .collect_view()}
                        </div>
                    </section>
                }
            })}
    }
}

#[component]
pub fn ChapterPage(outline: Vec<OutlineChapter>, entry: OutlineChapter) -> impl IntoView {
    let ctx = use_manual();
    let label = ctx.chapter_label;
    let position = outline
        .iter()
        .position(|e| e.chapter.slug == entry.chapter.slug);
    let prev = position
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| outline.get(i));
    let next = position.and_then(|i| outline.get(i + 1));
    let badge = ctx.applies_to(entry.chapter.applies_to);
    let home = ctx.href(None, None);
    let slug = entry.chapter.slug;

    let pager_link = |target: &OutlineChapter, forward: bool| {
        let href = ctx.href(Some(target.chapter.slug), None);
        let title = format!("{}. {}", target.number, target.chapter.title);
        view! {
            <a
                class=cn!(
                    ROW,
                    FOCUS,
                    "flex flex-col gap-2 px-2 py-6",
                    forward.then_some("items-end text-right sm:col-start-2"),
                )
                href=href.clone()
                on:click=ctx.on_link(href)
            >
                <HoverRule />
                <span class=cn!("inline-flex items-center gap-2", EYEBROW)>
                    {if forward {
                        view! {
                            "Next"
                            <Icon
                                icon=icons::ARROW_RIGHT
                                class="h-3.5 w-3.5 transition-transform group-hover:translate-x-1"
                            />
                        }
                            .into_any()
                    } else {
                        view! {
                            <Icon
                                icon=icons::ARROW_LEFT
                                class="h-3.5 w-3.5 transition-transform group-hover:-translate-x-1"
                            />
                            "Previous"
                        }
                            .into_any()
                    }}
                </span>
                <span class="text-lg font-light text-white">{title}</span>
            </a>
        }
    };
    let pager = (prev.is_some() || next.is_some()).then(|| {
        view! {
            <nav class="grid sm:grid-cols-2 sm:gap-8" aria-label="Previous and next">
                {prev.map(|p| pager_link(p, false))}
                {next.map(|n| pager_link(n, true))}
            </nav>
        }
    });

    view! {
        <header class="max-w-4xl">
            <p class=cn!(META, EYEBROW)>
                <span class="text-accent-400/80">{format!("{label} {}", entry.number)}</span>
                <span class="text-white/25">"/"</span>
                <a
                    class=cn!("transition-colors hover:text-white", FOCUS)
                    href=home.clone()
                    on:click=ctx.on_link(home)
                >
                    {ctx.manual.title}
                </a>
                {badge
                    .map(|b| {
                        view! {
                            <span class="text-white/25">"/"</span>
                            <span class=APPLIES>{b}</span>
                        }
                    })}
            </p>
            <h1 class=TITLE>{entry.chapter.title}</h1>
            {entry.chapter.summary.map(|s| view! { <p class=LEDE>{s}</p> })}
        </header>

        {entry
            .sections
            .is_empty()
            .then(|| {
                view! {
                    <p class="border-t border-white/10 pt-6 font-mono text-[11px] text-white/35">
                        {format!("This {} hasn't been written yet.", label.to_lowercase())}
                    </p>
                }
            })}
        {entry
            .sections
            .iter()
            .cloned()
            .map(|node| view! { <Section chapter_slug=Some(slug) node=node heading_level=2 /> })
            .collect_view()}

        {pager}
    }
}

#[component]
fn Section(
    /// Missing for the introduction, which lives on the main page.
    chapter_slug: Option<&'static str>,
    node: Arc<OutlineSection>,
    /// Heading level of a top-level section: 2 in a chapter, 3 on the main page.
    heading_level: usize,
) -> AnyView {
    let ctx = use_manual();
    let section = node.section;
    let depth = node.depth;
    let badge = ctx.applies_to(section.applies_to);
    let empty = node.blocks.is_empty() && node.children.is_empty();

    let heading_class = match depth {
        0 => "text-2xl leading-[1.25] font-light tracking-tight text-white md:text-3xl",
        1 => "text-lg leading-[1.45] font-normal text-white",
        _ => "text-base leading-normal font-medium text-white/90",
    };
    let title = section.title;
    let heading = match (depth + heading_level).min(6) {
        2 => view! { <h2 class=heading_class>{title}</h2> }.into_any(),
        3 => view! { <h3 class=heading_class>{title}</h3> }.into_any(),
        4 => view! { <h4 class=heading_class>{title}</h4> }.into_any(),
        5 => view! { <h5 class=heading_class>{title}</h5> }.into_any(),
        _ => view! { <h6 class=heading_class>{title}</h6> }.into_any(),
    };
    let number = node.number.clone();
    let copy = view! {
        <CopyLink chapter_slug=chapter_slug slug=node.slug.clone() title=title compact={depth > 0} />
    };

    // A top-level section opens with a numbered hairline label carrying the
    // copy-link button; nested ones put the number beside the heading.
    let head = if depth == 0 {
        view! {
            <div class=cn!("mb-5 flex items-center gap-4", EYEBROW)>
                {(!number.is_empty()).then(|| view! { <span class="text-accent-400/80">{number}</span> })}
                {badge.map(|b| view! { <span class=APPLIES>{b}</span> })}
                <span class="h-px flex-1 bg-white/10"></span>
                {copy}
            </div>
            {heading}
        }
            .into_any()
    } else {
        view! {
            <div class="flex items-baseline gap-3">
                {(!number.is_empty())
                    .then(|| {
                        view! {
                            <span class="min-w-10 shrink-0 font-mono text-[11px] tabular-nums text-accent-400/70">
                                {number}
                            </span>
                        }
                    })}
                <div class="flex min-w-0 flex-1 flex-wrap items-baseline gap-x-3 gap-y-1.5">
                    {heading}
                    {badge.map(|b| view! { <span class=APPLIES>{b}</span> })}
                </div>
                {copy}
            </div>
        }
            .into_any()
    };

    view! {
        <section
            id=node.slug.clone()
            data-manual-section=""
            class=cn!(
                "scroll-mt-[var(--manual-landing)]",
                match depth {
                    0 => "",
                    1 => "mt-10 border-t border-white/[0.07] pt-6",
                    _ => "mt-6",
                },
            )
        >
            {head}
            {section
                .summary
                .map(|s| {
                    view! {
                        <p class="mt-3 max-w-3xl text-sm leading-relaxed font-light text-white/55">{s}</p>
                    }
                })}
            {block_list(&node.blocks)}
            {empty
                .then(|| {
                    view! {
                        <p class="mt-4 font-mono text-[11px] text-white/35">
                            "This topic hasn't been written yet."
                        </p>
                    }
                })}
            {node
                .children
                .iter()
                .cloned()
                .map(|child| {
                    view! {
                        <Section chapter_slug=chapter_slug node=child heading_level=heading_level />
                    }
                })
                .collect_view()}
        </section>
    }
    .into_any()
}

#[component]
fn CopyLink(
    chapter_slug: Option<&'static str>,
    slug: String,
    title: &'static str,
    compact: bool,
) -> impl IntoView {
    let ctx = use_manual();
    let (copied, copy) = use_clipboard(Duration::from_millis(2000));

    let on_click = move |_| {
        let href = ctx.href(chapter_slug, Some(&slug));
        let Some(window) = web_sys::window() else {
            return;
        };
        let base = window.location().href().unwrap_or_default();
        let Ok(url) = web_sys::Url::new_with_base(&href, &base) else {
            return;
        };
        // Also put the link in the address bar, which is the fallback when the
        // clipboard isn't available (it needs a secure context).
        if let Ok(history) = window.history() {
            let state = history.state().unwrap_or(wasm_bindgen::JsValue::NULL);
            let _ = history.replace_state_with_url(&state, "", Some(&url.href()));
        }
        copy(url.href());
    };

    view! {
        <button
            type="button"
            class=cn!(
                "inline-flex shrink-0 items-center gap-1.5 transition-colors duration-150 hover:text-accent-300",
                FOCUS,
                if compact { "p-1 text-white/35" } else { "uppercase" },
            )
            on:click=on_click
            aria-label=format!("Copy link to {title}")
            title="Copy link"
        >
            {move || {
                if copied.get() {
                    view! { <Icon icon=icons::CHECK class="h-3 w-3 text-accent-400" /> }
                } else {
                    view! { <Icon icon=icons::LINK_2 class="h-3 w-3" /> }
                }
            }}
            {(!compact)
                .then(|| {
                    view! {
                        <span class="hidden sm:inline">
                            {move || if copied.get() { "Copied" } else { "Copy link" }}
                        </span>
                    }
                })}
        </button>
    }
}

/// A chapter the URL names that doesn't apply to the aircraft picked.
#[component]
pub fn ChapterUnavailable(chapter: &'static ManualChapter) -> impl IntoView {
    let ctx = use_manual();
    let aircraft = ctx.aircraft.get_untracked().unwrap_or_default();
    view! {
        <NotFoundPanel
            title=chapter.title.to_string()
            lede=format!(
                "This doesn't apply to the {aircraft}. Pick another aircraft at the top of the page, or go back to the overview.",
            )
        />
    }
}

#[component]
pub fn ChapterNotFound() -> impl IntoView {
    let label = use_manual().chapter_label;
    view! {
        <NotFoundPanel
            title=format!("{label} not found")
            lede=format!("This {} doesn't exist or has moved.", label.to_lowercase())
        />
    }
}

#[component]
fn NotFoundPanel(title: String, lede: String) -> impl IntoView {
    let ctx = use_manual();
    let home = ctx.href(None, None);
    view! {
        <div class="max-w-4xl py-12 md:py-20">
            <p class=cn!(META, EYEBROW)>{ctx.manual.title}</p>
            <h1 class=TITLE>{title}</h1>
            <p class=LEDE>{lede}</p>
            <a
                class=cn!(
                    "group mt-10 inline-flex items-center gap-2 text-lg font-medium text-white transition-colors hover:text-accent-300",
                    FOCUS,
                )
                href=home.clone()
                on:click=ctx.on_link(home)
            >
                <Icon
                    icon=icons::ARROW_LEFT
                    class="h-4 w-4 transition-transform group-hover:-translate-x-1"
                />
                "Back to the overview"
            </a>
        </div>
    }
}
