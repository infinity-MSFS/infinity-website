use std::sync::Arc;

use infinity_ui::prelude::*;
use leptos::html::{Div, Nav};

use super::model::{OutlineChapter, OutlineSection};
use super::shared::{FOCUS, POPOVER, use_manual};

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
                    "fixed inset-x-3 top-[calc(var(--manual-top)_+_var(--manual-bar)_+_0.5rem)] z-[25] max-h-[calc(100vh_-_var(--manual-top)_-_var(--manual-bar)_-_1.5rem)] flex-col rounded-3xl p-4",
                    if open.get() { "flex" } else { "hidden" },
                    "min-[1100px]:sticky min-[1100px]:inset-x-auto min-[1100px]:top-[calc(var(--manual-top)_+_var(--manual-bar)_+_1.5rem)] min-[1100px]:z-auto min-[1100px]:flex min-[1100px]:max-h-[calc(100vh_-_var(--manual-top)_-_var(--manual-bar)_-_3rem)] min-[1100px]:w-80 min-[1100px]:shrink-0 min-[1100px]:bg-white/[0.045] min-[1100px]:shadow-[0_12px_40px_rgba(0,0,0,0.28)] min-[1100px]:backdrop-blur-[16px]",
                )
            }
        >
            <div class="mb-3 flex shrink-0 items-center gap-2 text-sm font-semibold tracking-[0.02em] text-white/80">
                <Icon icon=icons::BOOK_OPEN class="h-4 w-4 text-accent-400" />
                "Contents"
            </div>
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
        <ol class=cn!(
            "list-none",
            (depth > 0).then_some("mt-0.5 mb-1 ml-4 border-l border-white/10 pl-1.5"),
        )>
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
                                        "mt-0.5 flex items-baseline gap-2.5 rounded-[0.875rem] border px-2.5 py-[0.4375rem] leading-[1.4] transition-colors duration-150",
                                        FOCUS,
                                        if depth == 0 { "text-sm" } else { "text-[0.8125rem]" },
                                        if is_active.get() {
                                            "border-accent-400/[0.28] bg-accent-400/[0.12] text-white"
                                        } else {
                                            "border-transparent text-white/[0.58] hover:bg-white/5 hover:text-white"
                                        },
                                    )
                                }
                                href=href.clone()
                                on:click=ctx.on_link(href)
                                aria-current=move || is_active.get().then_some("location")
                            >
                                <span class="min-w-7 shrink-0 text-xs font-semibold tabular-nums text-accent-400/85">
                                    {node.number.clone()}
                                </span>
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
