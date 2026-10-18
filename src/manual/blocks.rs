use infinity_ui::prelude::*;

use super::model::{Inline, parse_inline};
use super::shared::{APPLIES, FOCUS, OCTAGON_ALERT, use_manual};
use super::types::{BlockBody, CalloutTone, ManualBlock};
use crate::components::EYEBROW;

/// Renders inline markup: **bold**, *italic*, `code` and [label](target).
pub fn inline(text: &str) -> AnyView {
    parse_inline(text)
        .into_iter()
        .map(|part| match part {
            Inline::Text(text) => text.to_string().into_any(),
            Inline::Strong(text) => {
                view! { <strong class="font-semibold text-white">{text.to_string()}</strong> }
                    .into_any()
            }
            Inline::Code(text) => {
                view! {
                    <code class="rounded-sm bg-white/[0.06] px-[0.35em] py-[0.1em] font-mono text-[0.88em] text-accent-200">
                        {text.to_string()}
                    </code>
                }
                    .into_any()
            }
            Inline::Link { label, target } => content_link(label, target),
            Inline::Em(text) => view! { <em class="italic">{text.to_string()}</em> }.into_any(),
        })
        .collect_view()
        .into_any()
}

const LINK: &str = "text-accent-200 underline decoration-accent-400/45 underline-offset-[3px] hover:decoration-current";

fn content_link(label: &str, target: &str) -> AnyView {
    let ctx = use_manual();
    let resolved = ctx.resolve_link(target);
    if resolved.external {
        let web = resolved.href.starts_with("//")
            || resolved
                .href
                .get(..5)
                .is_some_and(|s| s.eq_ignore_ascii_case("http:"))
            || resolved
                .href
                .get(..6)
                .is_some_and(|s| s.eq_ignore_ascii_case("https:"));
        return view! {
            <a
                class=cn!(LINK, FOCUS)
                href=resolved.href
                target=web.then_some("_blank")
                rel=web.then_some("noopener noreferrer")
            >
                {inline(label)}
            </a>
        }
        .into_any();
    }
    let href = resolved.href;
    view! {
        <a class=cn!(LINK, FOCUS) href=href.clone() on:click=ctx.on_link(href)>
            {inline(label)}
        </a>
    }
    .into_any()
}

/// A blank line starts a paragraph; a single newline is a line break.
fn paragraphs(text: &str, class: &'static str) -> impl IntoView + use<> {
    let mut groups: Vec<Vec<&str>> = vec![Vec::new()];
    for line in text.trim().split('\n') {
        if line.trim().is_empty() {
            if groups.last().is_some_and(|g| !g.is_empty()) {
                groups.push(Vec::new());
            }
        } else if let Some(group) = groups.last_mut() {
            group.push(line);
        }
    }
    groups
        .into_iter()
        .filter(|lines| !lines.is_empty())
        .enumerate()
        .map(|(i, lines)| {
            view! {
                <p class=cn!(
                    "text-[0.9375rem] leading-[1.8]",
                    class,
                    (i > 0).then_some("mt-3.5"),
                )>
                    {lines
                        .into_iter()
                        .enumerate()
                        .map(|(j, line)| {
                            view! {
                                {(j > 0).then(|| view! { <br /> })}
                                {inline(line)}
                            }
                        })
                        .collect_view()}
                </p>
            }
        })
        .collect_view()
}

const BLOCK_TITLE: &str = EYEBROW;

fn block_view(block: &'static ManualBlock) -> AnyView {
    match block.body {
        BlockBody::Text(content) => paragraphs(content, "font-light text-white/70").into_any(),

        BlockBody::Callout {
            tone,
            title,
            content,
        } => {
            // A hairline in the tone's colour with a mono label. Warnings get
            // a heavier rule so they read first.
            let (label, icon, rule, head, body) = match tone {
                CalloutTone::Note => (
                    "Note",
                    icons::INFO,
                    "border-l border-accent-400/50",
                    "text-accent-300/90",
                    "leading-[1.7] font-light text-white/70",
                ),
                CalloutTone::Caution => (
                    "Caution",
                    icons::TRIANGLE_ALERT,
                    "border-l border-amber-400/60",
                    "text-amber-300",
                    "leading-[1.7] font-light text-amber-100/85",
                ),
                CalloutTone::Warning => (
                    "Warning",
                    OCTAGON_ALERT,
                    "border-l-2 border-red-400/70",
                    "text-red-300",
                    "leading-[1.7] font-light text-red-100/90",
                ),
            };
            view! {
                <div role="note" class=cn!("py-1 pl-5", rule)>
                    <p class=cn!(
                        "mb-2 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] tracking-[0.25em] uppercase",
                        head,
                    )>
                        <Icon icon=icon class="h-3.5 w-3.5 shrink-0" />
                        <span>{label}</span>
                        {title
                            .map(|t| {
                                view! {
                                    <span class="text-white/25">"/"</span>
                                    <span>{t}</span>
                                }
                            })}
                    </p>
                    {paragraphs(content, body)}
                </div>
            }
                .into_any()
        }

        BlockBody::List { ordered, items } => {
            let rows = items
                .iter()
                .map(|item| view! { <li class="pl-1">{inline(item)}</li> })
                .collect_view();
            let class = "space-y-1.5 pl-[1.375rem] text-[0.9375rem] leading-[1.75] font-light text-white/70 marker:text-accent-400/80";
            if ordered {
                view! {
                    <ol class=cn!(class, "list-decimal marker:font-mono marker:text-[0.8em] marker:tabular-nums")>
                        {rows}
                    </ol>
                }
                    .into_any()
            } else {
                view! { <ul class=cn!(class, "list-disc")>{rows}</ul> }.into_any()
            }
        }

        BlockBody::Checklist { title, items } => view! {
            <div class="border-y border-white/10 py-4">
                {title.map(|t| view! { <div class=cn!("mb-3", BLOCK_TITLE)>{t}</div> })}
                <ol class="list-none">
                    {items
                        .iter()
                        .map(|entry| {
                            view! {
                                <li class="flex items-baseline gap-2 py-[0.3125rem] text-[0.9375rem] leading-normal">
                                    <span class="font-light text-white/70">{inline(entry.item)}</span>
                                    <span
                                        class="min-w-6 flex-1 border-b border-dotted border-white/25"
                                        aria-hidden="true"
                                    ></span>
                                    <span class="text-right font-mono text-[0.8125rem] tracking-[0.08em] text-accent-200">
                                        {inline(entry.action)}
                                    </span>
                                </li>
                            }
                        })
                        .collect_view()}
                </ol>
            </div>
        }
        .into_any(),

        BlockBody::Table {
            title,
            columns,
            rows,
            dense,
        } => {
            let pad = if dense { "py-2 pr-4" } else { "py-3 pr-6" };
            let last = rows.len().saturating_sub(1);
            view! {
                <div class="border-y border-white/10">
                    {title
                        .map(|t| {
                            view! {
                                <div class=cn!(
                                    BLOCK_TITLE,
                                    "border-b border-white/10 py-3",
                                )>{t}</div>
                            }
                        })}
                    <div class="max-h-[32rem] overflow-auto">
                        <table
                            class=cn!(
                                "w-full border-separate border-spacing-0 text-left",
                                if dense { "text-xs leading-[1.45]" } else { "text-sm leading-[1.6]" },
                            )
                            aria-label=title
                        >
                            <thead>
                                <tr>
                                    {columns
                                        .iter()
                                        .map(|column| {
                                            view! {
                                                <th
                                                    scope="col"
                                                    class=cn!(
                                                        "sticky top-0 z-[1] whitespace-nowrap border-b border-white/10 bg-[rgb(5,9,11)] font-mono text-[10px] font-normal tracking-[0.2em] text-white/45 uppercase",
                                                        pad,
                                                    )
                                                >
                                                    {inline(column)}
                                                </th>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            </thead>
                            <tbody>
                                {rows
                                    .iter()
                                    .enumerate()
                                    .map(|(r, row)| {
                                        let cell = cn!(
                                            "align-top font-light text-white/70",
                                            pad,
                                            (r < last).then_some("border-b border-white/[0.06]"),
                                        );
                                        view! {
                                            <tr class="transition-colors hover:bg-white/[0.02]">
                                                {row
                                                    .iter()
                                                    .map(|value| {
                                                        view! { <td class=cell.clone()>{inline(value)}</td> }
                                                    })
                                                    .collect_view()}
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    </div>
                </div>
            }
                .into_any()
        }

        BlockBody::Figure {
            src,
            alt,
            title,
            caption,
            width,
            height,
        } => {
            let src = use_manual().resolve_asset(src);
            view! {
                <figure>
                    {title.map(|t| view! { <div class=cn!("mb-3", BLOCK_TITLE)>{t}</div> })}
                    <img
                        class="block h-auto w-full rounded-sm border border-white/10 bg-black/30"
                        src=src
                        alt=alt
                        width=width
                        height=height
                        loading="lazy"
                        decoding="async"
                    />
                    {caption
                        .map(|c| {
                            view! {
                                <figcaption class="mt-3 border-l border-white/15 pl-4 text-sm leading-[1.6] font-light text-white/55">
                                    {inline(c)}
                                </figcaption>
                            }
                        })}
                </figure>
            }
                .into_any()
        }
    }
}

pub fn block_list(blocks: &[&'static ManualBlock]) -> impl IntoView + use<> {
    let ctx = use_manual();
    (!blocks.is_empty()).then(|| {
        let views = blocks
            .iter()
            .map(|block| {
                let label = ctx.applies_to(block.applies_to);
                view! {
                    <div>
                        {label
                            .map(|l| {
                                view! { <p class=cn!("mb-2", APPLIES)>{l}</p> }
                            })}
                        {block_view(block)}
                    </div>
                }
            })
            .collect_view();
        view! { <div class="mt-6 flex flex-col gap-6">{views}</div> }
    })
}
