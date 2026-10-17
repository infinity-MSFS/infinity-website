use std::sync::Arc;
use std::time::Duration;

use infinity_ui::prelude::*;

use super::blocks::block_list;
use super::model::{OutlineChapter, OutlineSection};
use super::shared::{FOCUS, PANEL, use_manual};
use super::types::ManualChapter;

const TITLE: &str =
    "text-[1.875rem] font-semibold leading-[1.2] tracking-[-0.015em] text-white md:text-[2.25rem]";
const LEDE: &str = "mt-4 max-w-3xl text-base leading-[1.75] text-white/[0.58]";
const HEADER: &str = "p-7 md:p-9";
const EYEBROW: &str = "mb-4 flex flex-wrap items-center gap-x-3 gap-y-2 text-xs uppercase tracking-[0.16em] text-white/40";
const GROUP_HEADING: &str = "mt-3 text-xs font-semibold uppercase tracking-[0.16em] text-white/40";
const CARD_HOVER: &str =
    "transition-colors duration-150 hover:border-accent-400/35 hover:bg-accent-400/[0.06]";

/// The manual's front page: every chapter, then the introduction.
#[component]
pub fn MainPage(outline: Vec<OutlineChapter>, intro: Vec<Arc<OutlineSection>>) -> impl IntoView {
    let ctx = use_manual();
    let manual = ctx.manual;
    let aircraft = ctx.aircraft.get_untracked();
    let label = ctx.chapter_label;

    view! {
        <header class=cn!(PANEL, HEADER)>
            <div class=EYEBROW>
                <Badge tone=Tone::Accent>"Manual"</Badge>
                {manual.revision.map(|r| view! { <span>{r}</span> })}
            </div>
            <h1 class=TITLE>{manual.title}</h1>
            {manual.summary.map(|s| view! { <p class=LEDE>{s}</p> })}
            {aircraft
                .filter(|_| manual.aircraft.len() > 1)
                .map(|aircraft| {
                    view! {
                        <p class="mt-5 text-[0.9375rem] leading-[1.6] text-white/[0.58]">
                            "Showing the " <strong class="font-semibold text-white">{aircraft}</strong>
                            ". Switch aircraft with the buttons at the top of the page."
                        </p>
                    }
                })}
        </header>

        <h2 class=GROUP_HEADING>{format!("{label}s")}</h2>
        <ol class="grid list-none gap-4 sm:grid-cols-2 min-[1100px]:grid-cols-3">
            {outline
                .into_iter()
                .map(|entry| {
                    let chapter = entry.chapter;
                    let badge = ctx.applies_to(chapter.applies_to);
                    let count = entry.flat.len();
                    let href = ctx.href(Some(chapter.slug), None);
                    view! {
                        <li class="flex">
                            <a
                                class=cn!(PANEL, CARD_HOVER, FOCUS, "flex flex-1 flex-col gap-2 px-6 py-[1.375rem]")
                                href=href.clone()
                                on:click=ctx.on_link(href)
                            >
                                <span class="flex flex-wrap items-center gap-2">
                                    <span class="text-xs font-semibold uppercase tracking-[0.16em] text-accent-400/90">
                                        {format!("{label} {}", entry.number)}
                                    </span>
                                    {badge.map(|b| view! { <Badge tone=Tone::Info size=Size::Sm>{b}</Badge> })}
                                </span>
                                <span class="text-xl font-semibold leading-[1.3] text-white">
                                    {chapter.title}
                                </span>
                                {chapter
                                    .summary
                                    .map(|s| {
                                        view! {
                                            <span class="flex-1 text-sm leading-[1.6] text-white/[0.58]">
                                                {s}
                                            </span>
                                        }
                                    })}
                                <span class="text-xs text-white/40">
                                    {if count == 1 { "1 topic".to_string() } else { format!("{count} topics") }}
                                </span>
                            </a>
                        </li>
                    }
                })
                .collect_view()}
        </ol>

        {(!intro.is_empty())
            .then(|| {
                view! {
                    <h2 class=GROUP_HEADING>"Introduction"</h2>
                    {intro
                        .into_iter()
                        .map(|node| view! { <Section chapter_slug=None node=node heading_level=3 /> })
                        .collect_view()}
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
                    PANEL,
                    CARD_HOVER,
                    FOCUS,
                    "flex flex-col gap-1 px-5 py-4",
                    forward.then_some("items-end text-right sm:col-start-2"),
                )
                href=href.clone()
                on:click=ctx.on_link(href)
            >
                <span class="inline-flex items-center gap-1 text-xs uppercase tracking-[0.14em] text-white/40">
                    {if forward {
                        view! {
                            "Next"
                            <Icon icon=icons::CHEVRON_RIGHT class="h-3.5 w-3.5" />
                        }
                            .into_any()
                    } else {
                        view! {
                            <Icon icon=icons::CHEVRON_LEFT class="h-3.5 w-3.5" />
                            "Previous"
                        }
                            .into_any()
                    }}
                </span>
                <span class="text-base font-semibold text-white">{title}</span>
            </a>
        }
    };
    let pager = (prev.is_some() || next.is_some()).then(|| {
        view! {
            <nav class="grid gap-4 sm:grid-cols-2" aria-label="Previous and next">
                {prev.map(|p| pager_link(p, false))}
                {next.map(|n| pager_link(n, true))}
            </nav>
        }
    });

    view! {
        <header class=cn!(PANEL, HEADER)>
            <div class=EYEBROW>
                <Badge tone=Tone::Accent>{format!("{label} {}", entry.number)}</Badge>
                <a
                    class=cn!("text-inherit hover:text-white", FOCUS)
                    href=home.clone()
                    on:click=ctx.on_link(home)
                >
                    {ctx.manual.title}
                </a>
                {badge.map(|b| view! { <Badge tone=Tone::Info size=Size::Sm>{b}</Badge> })}
            </div>
            <h1 class=TITLE>{entry.chapter.title}</h1>
            {entry.chapter.summary.map(|s| view! { <p class=LEDE>{s}</p> })}
        </header>

        {entry
            .sections
            .is_empty()
            .then(|| {
                view! {
                    <p class=cn!(PANEL, "p-6 text-sm italic text-white/40")>
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
        0 => "text-[1.5rem] font-semibold leading-[1.3] tracking-[-0.01em] text-white",
        1 => "text-[1.125rem] font-semibold leading-[1.45] text-white",
        _ => "text-base font-semibold leading-normal text-white",
    };
    let title = section.title;
    let heading = match (depth + heading_level).min(6) {
        2 => view! { <h2 class=heading_class>{title}</h2> }.into_any(),
        3 => view! { <h3 class=heading_class>{title}</h3> }.into_any(),
        4 => view! { <h4 class=heading_class>{title}</h4> }.into_any(),
        5 => view! { <h5 class=heading_class>{title}</h5> }.into_any(),
        _ => view! { <h6 class=heading_class>{title}</h6> }.into_any(),
    };

    let number = (!node.number.is_empty()).then(|| {
        let class = if depth == 0 {
            "mt-0.5 min-w-10 shrink-0 rounded-xl border border-accent-400/[0.22] bg-accent-400/10 px-2 py-[0.3125rem] text-center text-xs font-bold leading-[1.4] tabular-nums text-accent-200"
        } else {
            "min-w-10 shrink-0 pt-0.5 text-center text-[0.8125rem] font-semibold tabular-nums text-accent-400/85"
        };
        view! { <span class=class>{node.number.clone()}</span> }
    });

    view! {
        <section
            id=node.slug.clone()
            data-manual-section=""
            class=cn!(
                "scroll-mt-[var(--manual-landing)]",
                match depth {
                    0 => cn!(PANEL, "p-6 md:px-8 md:py-7"),
                    1 => "mt-7 border-t border-white/[0.07] pt-6".to_string(),
                    _ => "mt-6".to_string(),
                },
            )
        >
            <div class="flex items-start gap-3">
                {number}
                <div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1.5">
                    {heading}
                    {badge.map(|b| view! { <Badge tone=Tone::Info size=Size::Sm>{b}</Badge> })}
                </div>
                <CopyLink
                    chapter_slug=chapter_slug
                    slug=node.slug.clone()
                    title=title
                    compact={depth > 0}
                />
            </div>
            {section
                .summary
                .map(|s| {
                    view! {
                        <p class="mt-3 text-[0.9375rem] leading-[1.65] text-white/[0.58]">{s}</p>
                    }
                })}
            {block_list(&node.blocks)}
            {empty
                .then(|| {
                    view! {
                        <p class="mt-4 text-sm italic text-white/40">
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
                "inline-flex shrink-0 items-center gap-1.5 rounded-[0.875rem] border text-[0.8125rem] leading-[1.4] text-white/[0.58] transition-colors duration-150 hover:border-accent-400/30 hover:bg-accent-400/10 hover:text-white",
                FOCUS,
                if compact {
                    "border-transparent bg-transparent p-1.5"
                } else {
                    "border-white/10 bg-black/20 px-2.5 py-1.5"
                },
            )
            on:click=on_click
            aria-label=format!("Copy link to {title}")
            title="Copy link"
        >
            {move || {
                let icon = if copied.get() { icons::CHECK } else { icons::LINK_2 };
                view! { <Icon icon=icon class="h-3.5 w-3.5 text-accent-400" /> }
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
        <div class=cn!(PANEL, "px-7 py-10 text-center")>
            <h1 class=TITLE>{title}</h1>
            <p class=cn!(LEDE, "mx-auto")>{lede}</p>
            <a
                class=cn!(
                    "mt-6 inline-flex rounded-lg bg-accent-600 px-6 py-2 font-medium text-white transition-colors hover:bg-accent-700",
                    FOCUS,
                )
                href=home.clone()
                on:click=ctx.on_link(home)
            >
                "Back to the overview"
            </a>
        </div>
    }
}
