use infinity_ui::prelude::*;

use super::model::{Inline, parse_inline};
use super::shared::{FOCUS, OCTAGON_ALERT, use_manual};
use super::types::{BlockBody, CalloutTone, ManualBlock};

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
                    <code class="rounded-md border border-white/10 bg-black/30 px-[0.4em] py-[0.1em] font-mono text-[0.88em] text-accent-200">
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

const BLOCK_TITLE: &str = "text-xs font-semibold uppercase tracking-[0.16em] text-white/40";

fn block_view(block: &'static ManualBlock) -> AnyView {
    match block.body {
        BlockBody::Text(content) => paragraphs(content, "").into_any(),

        BlockBody::Callout {
            tone,
            title,
            content,
        } => {
            let (label, tone, icon) = match tone {
                CalloutTone::Note => ("Note", Tone::Accent, icons::INFO),
                CalloutTone::Caution => ("Caution", Tone::Warning, icons::TRIANGLE_ALERT),
                CalloutTone::Warning => ("Warning", Tone::Danger, OCTAGON_ALERT),
            };
            view! {
                <Callout tone=tone icon=icon title=title.unwrap_or(label).to_string()>
                    {paragraphs(content, "text-[0.9rem] leading-[1.7] text-white/[0.86]")}
                </Callout>
            }
                .into_any()
        }

        BlockBody::List { ordered, items } => {
            let rows = items
                .iter()
                .map(|item| view! { <li class="pl-1">{inline(item)}</li> })
                .collect_view();
            let class = "space-y-1.5 pl-[1.375rem] text-[0.9375rem] leading-[1.75] marker:text-accent-400/85";
            if ordered {
                view! {
                    <ol class=cn!(class, "list-decimal marker:font-semibold marker:tabular-nums")>
                        {rows}
                    </ol>
                }
                    .into_any()
            } else {
                view! { <ul class=cn!(class, "list-disc")>{rows}</ul> }.into_any()
            }
        }

        BlockBody::Checklist { title, items } => view! {
            <div class="rounded-[1.25rem] border border-white/10 bg-black/20 px-5 py-4">
                {title.map(|t| view! { <div class=cn!("mb-3", BLOCK_TITLE)>{t}</div> })}
                <ol class="list-none">
                    {items
                        .iter()
                        .map(|entry| {
                            view! {
                                <li class="flex items-baseline gap-2 py-[0.3125rem] text-[0.9375rem] leading-normal">
                                    <span class="text-white/[0.76]">{inline(entry.item)}</span>
                                    <span
                                        class="min-w-6 flex-1 border-b-2 border-dotted border-white/20"
                                        aria-hidden="true"
                                    ></span>
                                    <span class="text-right font-semibold tracking-[0.04em] text-accent-200">
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
            let pad = if dense { "px-3 py-2" } else { "px-4 py-3" };
            let last = rows.len().saturating_sub(1);
            view! {
                <div class="overflow-hidden rounded-[1.25rem] border border-white/10 bg-black/15">
                    {title
                        .map(|t| {
                            view! {
                                <div class=cn!(
                                    BLOCK_TITLE,
                                    "border-b border-white/10 bg-white/[0.03] px-4 py-3",
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
                                                        "sticky top-0 z-[1] whitespace-nowrap border-b border-white/10 bg-[rgb(11,17,19)] text-[0.6875rem] font-semibold uppercase tracking-[0.14em] text-white/40",
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
                                            "align-top text-white/[0.76]",
                                            pad,
                                            (r < last).then_some("border-b border-white/[0.06]"),
                                            (r % 2 == 0).then_some("bg-white/[0.02]"),
                                        );
                                        view! {
                                            <tr>
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
                <figure class="rounded-[1.25rem] border border-white/10 bg-black/15 p-3.5">
                    {title.map(|t| view! { <div class=cn!("mb-3", BLOCK_TITLE)>{t}</div> })}
                    <img
                        class="block h-auto w-full rounded-[0.875rem] bg-black/30"
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
                                <figcaption class="mt-3 text-sm leading-[1.6] text-white/[0.58]">
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
                                view! {
                                    <Badge tone=Tone::Info size=Size::Sm class="mb-2">
                                        {l}
                                    </Badge>
                                }
                            })}
                        {block_view(block)}
                    </div>
                }
            })
            .collect_view();
        view! { <div class="mt-5 flex flex-col gap-[1.125rem]">{views}</div> }
    })
}
