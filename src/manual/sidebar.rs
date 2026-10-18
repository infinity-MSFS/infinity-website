use std::sync::Arc;

use infinity_ui::prelude::*;
use leptos::html::{Div, Nav};

use super::model::{OutlineChapter, OutlineSection};
use super::shared::{FOCUS, POPOVER, use_manual};
use crate::components::EYEBROW;

/// The current chapter's contents: a sticky column on wide screens, and a
/// panel opened from the bar's Contents button on narrow ones.
#[component]
pub fn Sidebar(
    id: String,
    chapter: OutlineChapter,
    #[prop(into)] active_slug: Signal<Option<String>>,
    /// Narrow screens only: the list opens over the page from the Contents button.
    #[prop(into)]
    open: Signal<bool>,
    nav_ref: NodeRef<Nav>,
    scroll_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        <nav
            id=id
            node_ref=nav_ref
            aria-label="Contents"
            class=move || {
                cn!(
                    POPOVER,
                    "fixed inset-x-3 top-[calc(var(--manual-top)_+_var(--manual-bar)_+_0.5rem)] z-[25] max-h-[calc(100vh_-_var(--manual-top)_-_var(--manual-bar)_-_1.5rem)] flex-col p-5",
                    if open.get() { "flex" } else { "hidden" },
                    "min-[1100px]:sticky min-[1100px]:inset-x-auto min-[1100px]:top-[calc(var(--manual-top)_+_var(--manual-bar)_+_1.5rem)] min-[1100px]:z-auto min-[1100px]:flex min-[1100px]:max-h-[calc(100vh_-_var(--manual-top)_-_var(--manual-bar)_-_3rem)] min-[1100px]:w-72 min-[1100px]:shrink-0 min-[1100px]:border-0 min-[1100px]:bg-transparent min-[1100px]:p-0",
                )
            }
        >
            <p class=cn!("mb-4 shrink-0", EYEBROW)>"Contents"</p>
            <div node_ref=scroll_ref class="-mx-1 min-h-0 flex-1 overflow-y-auto overscroll-contain px-1">
                {section_list(chapter.chapter.slug, &chapter.sections, active_slug, 0)}
            </div>
        </nav>
    }
}

fn section_list(
    chapter_slug: &'static str,
    nodes: &[Arc<OutlineSection>],
    active_slug: Signal<Option<String>>,
    depth: usize,
) -> AnyView {
    let ctx = use_manual();
    view! {
        // One hairline for the whole tree: nested entries indent, and the
        // active one marks the line wherever it sits.
        <ol class=cn!("list-none", (depth == 0).then_some("border-l border-white/10"))>
            {nodes
                .iter()
                .map(|node| {
                    let href = ctx.href(Some(chapter_slug), Some(&node.slug));
                    let slug = node.slug.clone();
                    let is_active = Signal::derive(move || {
                        active_slug.with(|a| a.as_deref() == Some(slug.as_str()))
                    });
                    view! {
                        <li>
                            <a
                                data-toc=node.slug.clone()
                                class=move || {
                                    cn!(
                                        "-ml-px flex items-baseline gap-3 border-l py-1.5 pr-2 leading-[1.4] transition-colors duration-150",
                                        FOCUS,
                                        match depth {
                                            0 => "pl-4 text-sm",
                                            1 => "pl-8 text-[0.8125rem]",
                                            _ => "pl-12 text-[0.8125rem]",
                                        },
                                        if is_active.get() {
                                            "border-accent-400 text-white"
                                        } else {
                                            "border-transparent font-light text-white/50 hover:text-white/85"
                                        },
                                    )
                                }
                                href=href.clone()
                                on:click=ctx.on_link(href)
                                aria-current=move || is_active.get().then_some("location")
                            >
                                <span class=move || {
                                    cn!(
                                        "min-w-8 shrink-0 font-mono text-[10px] tabular-nums",
                                        if is_active.get() { "text-accent-400" } else { "text-white/30" },
                                    )
                                }>{node.number.clone()}</span>
                                <span class="min-w-0">{node.section.title}</span>
                            </a>
                            {(!node.children.is_empty())
                                .then(|| section_list(chapter_slug, &node.children, active_slug, depth + 1))}
                        </li>
                    }
                })
                .collect_view()}
        </ol>
    }
    .into_any()
}
