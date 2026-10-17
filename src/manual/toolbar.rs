use infinity_ui::prelude::*;
use leptos::html::{Button, Div, Input, Ol, Ul};
use wasm_bindgen::JsCast;

use super::model::{OutlineChapter, SearchHit, highlight};
use super::shared::{
    CONTROL, CONTROL_IDLE, CONTROL_OPEN, FOCUS, LIST, POPOVER, Panel, contains, links_in,
    move_focus, use_dismiss, use_manual,
};
use super::types::AircraftName;

const ICON_BUTTON: &str = "inline-flex min-w-10 shrink-0 items-center justify-center gap-2 px-3";

/// The sticky bar: chapter menu, aircraft buttons, the narrow-screen Contents
/// button, and search.
#[component]
pub fn Toolbar(
    #[prop(into)] outline: Signal<Vec<OutlineChapter>>,
    #[prop(into)] current: Signal<Option<OutlineChapter>>,
    /// Text on the chapter menu button.
    #[prop(into)]
    menu_label: Signal<String>,
    #[prop(into)] on_main_page: Signal<bool>,
    #[prop(into)] on_aircraft_change: Callback<AircraftName>,
    panel: RwSignal<Option<Panel>>,
    query: RwSignal<String>,
    #[prop(into)] terms: Signal<Vec<String>>,
    #[prop(into)] hits: Signal<Vec<SearchHit>>,
    /// Id of the chapter's contents list, which the Contents button opens on narrow screens.
    sections_id: String,
    sections_toggle: NodeRef<Button>,
) -> impl IntoView {
    let ctx = use_manual();
    let close = Callback::new(move |()| panel.set(None));
    let open_with = move |which: Panel| Callback::new(move |()| panel.set(Some(which)));
    let sections_open = move || panel.get() == Some(Panel::Sections);
    let has_sections = move || current.with(|c| c.as_ref().is_some_and(|c| !c.sections.is_empty()));

    view! {
        <div class=cn!(
            "sticky top-[var(--manual-top)] z-20 flex h-[var(--manual-bar)] items-center gap-2 border-b border-white/[0.08] bg-[rgba(4,9,11,0.9)] px-4 backdrop-blur-[24px] md:gap-3 md:px-6",
            (ctx.manual.aircraft.len() > 1).then_some("flex-wrap content-center gap-y-2 md:flex-nowrap"),
        )>
            <ChapterMenu
                outline=outline
                current=current
                label=menu_label
                on_main_page=on_main_page
                open=Signal::derive(move || panel.get() == Some(Panel::Chapters))
                on_open=open_with(Panel::Chapters)
                on_close=close
            />
            <AircraftPicker on_change=on_aircraft_change />
            <Show when=has_sections>
                <button
                    node_ref=sections_toggle
                    type="button"
                    class=move || {
                        cn!(
                            CONTROL,
                            ICON_BUTTON,
                            FOCUS,
                            "min-[1100px]:hidden",
                            if sections_open() { CONTROL_OPEN } else { CONTROL_IDLE },
                        )
                    }
                    aria-label="Contents"
                    aria-expanded=move || sections_open().to_string()
                    aria-controls=sections_id.clone()
                    on:click=move |_| {
                        panel.set(if sections_open() { None } else { Some(Panel::Sections) })
                    }
                >
                    <Icon icon=LIST class="h-4 w-4 text-accent-400" />
                    <span class="hidden sm:inline">"Contents"</span>
                </button>
            </Show>
            <SearchBox
                open=Signal::derive(move || panel.get() == Some(Panel::Search))
                on_open=open_with(Panel::Search)
                on_close=close
                query=query
                terms=terms
                hits=hits
            />
        </div>
    }
}

/// One button per aircraft; the manual shows only what applies to the one
/// picked. Native radio buttons, so arrow keys move between them.
#[component]
fn AircraftPicker(on_change: Callback<AircraftName>) -> impl IntoView {
    let ctx = use_manual();
    let group = use_id("manual-aircraft");
    (ctx.manual.aircraft.len() > 1).then(|| {
        view! {
            <fieldset class="order-first flex h-10 min-w-0 flex-[1_0_100%] gap-[0.1875rem] rounded-2xl border border-white/10 bg-black/20 p-[0.1875rem] md:order-none md:flex-none">
                <legend class="sr-only">"Aircraft"</legend>
                {ctx
                    .manual
                    .aircraft
                    .iter()
                    .copied()
                    .map(|name| {
                        let picked = move || ctx.aircraft.get() == Some(name);
                        view! {
                            <label class=move || {
                                cn!(
                                    "relative flex flex-1 cursor-pointer items-center justify-center whitespace-nowrap rounded-xl px-3.5 text-[0.8125rem] font-semibold tracking-[0.02em] transition-colors duration-150 md:flex-none",
                                    "has-[:focus-visible]:outline has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-accent-400",
                                    if picked() {
                                        "bg-accent-400/[0.16] text-accent-200 shadow-[inset_0_0_0_1px_rgb(var(--ac-400)/0.32)]"
                                    } else {
                                        "text-white/[0.58] hover:bg-white/5 hover:text-white"
                                    },
                                )
                            }>
                                <input
                                    class="pointer-events-none absolute h-px w-px opacity-0"
                                    type="radio"
                                    name=group.clone()
                                    value=name
                                    prop:checked=picked
                                    on:change=move |_| on_change.run(name)
                                />
                                {name}
                            </label>
                        }
                    })
                    .collect_view()}
            </fieldset>
        }
    })
}

/// Closes `on_close` when focus moves somewhere outside `root`.
fn close_on_focus_out(
    open: Signal<bool>,
    root: NodeRef<Div>,
    on_close: Callback<()>,
) -> impl Fn(web_sys::FocusEvent) + 'static {
    move |ev| {
        let next = ev
            .related_target()
            .and_then(|t| t.dyn_into::<web_sys::Node>().ok());
        if let Some(next) = next
            && open.get_untracked()
            && !contains(root.get_untracked(), &next)
        {
            on_close.run(());
        }
    }
}

#[component]
fn ChapterMenu(
    outline: Signal<Vec<OutlineChapter>>,
    current: Signal<Option<OutlineChapter>>,
    label: Signal<String>,
    on_main_page: Signal<bool>,
    open: Signal<bool>,
    on_open: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let ctx = use_manual();
    let label_id = use_id("manual-menu-label");
    let button_id = use_id("manual-menu-button");
    let list_id = use_id("manual-menu-list");
    let root = NodeRef::<Div>::new();
    let button = NodeRef::<Button>::new();
    let list = NodeRef::<Ul>::new();
    use_dismiss(
        move || open.get_untracked(),
        move || on_close.run(()),
        move |node| contains(root.get_untracked(), node),
    );

    let on_button_key = move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "Escape" if open.get_untracked() => {
            ev.prevent_default();
            on_close.run(());
        }
        "ArrowDown" | "ArrowUp" => {
            ev.prevent_default();
            on_open.run(());
            // The list renders on the next tick; focus once it's there.
            request_animation_frame(move || {
                let links = links_in(list.get_untracked());
                let target = links
                    .iter()
                    .find(|a| a.has_attribute("aria-current"))
                    .or(links.first());
                if let Some(target) = target {
                    let _ = target.focus();
                }
            });
        }
        _ => {}
    };

    let on_list_key = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.prevent_default();
            on_close.run(());
            if let Some(button) = button.get_untracked() {
                let _ = button.focus();
            }
        } else if move_focus(&links_in(list.get_untracked()), &ev.key()) {
            ev.prevent_default();
        }
    };

    let items = move || {
        let overview = (
            String::new(),
            "Overview".to_string(),
            ctx.href(None, None),
            on_main_page.get(),
        );
        let current_slug = current.with(|c| c.as_ref().map(|c| c.chapter.slug));
        let chapters = outline.with(|outline| {
            outline
                .iter()
                .map(|entry| {
                    (
                        entry.number.clone(),
                        entry.chapter.title.to_string(),
                        ctx.href(Some(entry.chapter.slug), None),
                        current_slug == Some(entry.chapter.slug),
                    )
                })
                .collect::<Vec<_>>()
        });
        std::iter::once(overview)
            .chain(chapters)
            .collect::<Vec<_>>()
    };

    view! {
        <div
            node_ref=root
            class="flex min-w-0 flex-1 items-center gap-3 md:flex-initial"
            on:focusout=close_on_focus_out(open, root, on_close)
        >
            <span
                id=label_id.clone()
                class="hidden shrink-0 text-xs font-medium uppercase tracking-[0.18em] text-white/35 min-[1100px]:block"
            >
                {ctx.chapter_label}
            </span>
            <div class="relative min-w-0 flex-1 md:w-80 md:min-w-[10rem] md:flex-[0_1_20rem]">
                <button
                    node_ref=button
                    id=button_id.clone()
                    type="button"
                    class=move || {
                        cn!(
                            CONTROL,
                            FOCUS,
                            "flex w-full items-center justify-between gap-3 px-4 text-left",
                            if open.get() { CONTROL_OPEN } else { CONTROL_IDLE },
                        )
                    }
                    aria-labelledby=format!("{label_id} {button_id}")
                    aria-expanded=move || open.get().to_string()
                    aria-controls=list_id.clone()
                    on:click=move |_| if open.get_untracked() { on_close.run(()) } else { on_open.run(()) }
                    on:keydown=on_button_key
                >
                    <span class="min-w-0 truncate">{move || label.get()}</span>
                    <span class=move || {
                        cn!(
                            "flex shrink-0 text-accent-400 transition-transform duration-150",
                            open.get().then_some("rotate-180"),
                        )
                    }>
                        <Icon icon=icons::CHEVRON_DOWN class="h-4 w-4" />
                    </span>
                </button>
                <Show when=move || open.get()>
                    <ul
                        id=list_id.clone()
                        node_ref=list
                        class=cn!(
                            POPOVER,
                            "absolute top-[calc(100%_+_0.5rem)] left-0 z-30 flex max-h-[min(26rem,calc(100vh_-_var(--manual-top)_-_var(--manual-bar)_-_2rem))] w-[max(100%,18rem)] list-none flex-col gap-0.5 overflow-y-auto overscroll-contain rounded-[1.375rem] p-2",
                        )
                        on:keydown=on_list_key
                    >
                        {move || {
                            items()
                                .into_iter()
                                .map(|(number, title, href, is_current)| {
                                    view! {
                                        <li>
                                            <a
                                                class=cn!(
                                                    "flex items-center gap-3 rounded-xl border px-3 py-2.5 text-sm transition-colors duration-150",
                                                    FOCUS,
                                                    if is_current {
                                                        "border-accent-400/20 bg-accent-400/15 text-accent-200"
                                                    } else {
                                                        "border-transparent text-white/70 hover:bg-white/5 hover:text-white focus-visible:bg-white/5 focus-visible:text-white"
                                                    },
                                                )
                                                href=href.clone()
                                                on:click=ctx.on_link(href)
                                                aria-current=is_current.then_some("page")
                                            >
                                                <span class="min-w-4 shrink-0 text-xs font-semibold tabular-nums text-accent-400/90">
                                                    {number}
                                                </span>
                                                <span class="min-w-0 flex-1 truncate">{title}</span>
                                                {is_current
                                                    .then(|| {
                                                        view! {
                                                            <Icon
                                                                icon=icons::CHECK
                                                                class="h-4 w-4 shrink-0 text-accent-400"
                                                            />
                                                        }
                                                    })}
                                            </a>
                                        </li>
                                    }
                                })
                                .collect_view()
                        }}
                    </ul>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn SearchBox(
    open: Signal<bool>,
    on_open: Callback<()>,
    on_close: Callback<()>,
    query: RwSignal<String>,
    terms: Signal<Vec<String>>,
    hits: Signal<Vec<SearchHit>>,
) -> impl IntoView {
    let panel_id = use_id("manual-search");
    let root = NodeRef::<Div>::new();
    let toggle = NodeRef::<Button>::new();
    let input = NodeRef::<Input>::new();
    let results = NodeRef::<Ol>::new();
    use_dismiss(
        move || open.get_untracked(),
        move || on_close.run(()),
        move |node| contains(root.get_untracked(), node),
    );

    // On narrow screens the field only appears once opened; put the cursor in it.
    Effect::watch(
        move || open.get(),
        move |open, _, _| {
            if *open {
                request_animation_frame(move || {
                    if let Some(input) = input.get_untracked() {
                        let _ = input.focus();
                    }
                });
            }
        },
        false,
    );

    let clear_and_close = move || {
        query.set(String::new());
        on_close.run(());
        // The field hides with the panel on narrow screens, so hand focus back
        // to the button that opened it.
        if let Some(toggle) = toggle.get_untracked()
            && toggle.offset_parent().is_some()
        {
            let _ = toggle.focus();
        }
    };

    let on_input_key = move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "Escape" => {
            ev.prevent_default();
            clear_and_close();
        }
        "ArrowDown" => {
            if let Some(first) = links_in(results.get_untracked()).first() {
                ev.prevent_default();
                let _ = first.focus();
            }
        }
        _ => {}
    };

    view! {
        <div
            node_ref=root
            class="md:ml-auto min-[1100px]:relative min-[1100px]:min-w-0 min-[1100px]:flex-[0_1_22rem]"
            on:focusout=close_on_focus_out(open, root, on_close)
        >
            <button
                node_ref=toggle
                type="button"
                class=move || {
                    cn!(
                        CONTROL,
                        ICON_BUTTON,
                        FOCUS,
                        "min-[1100px]:hidden",
                        if open.get() { CONTROL_OPEN } else { CONTROL_IDLE },
                    )
                }
                aria-label="Search the manual"
                aria-expanded=move || open.get().to_string()
                aria-controls=panel_id.clone()
                on:click=move |_| if open.get_untracked() { on_close.run(()) } else { on_open.run(()) }
            >
                <Icon icon=icons::SEARCH class="h-4 w-4 text-accent-400" />
            </button>
            <div
                id=panel_id.clone()
                class=move || {
                    cn!(
                        "absolute inset-x-0 top-full border-b border-white/10 bg-[rgb(4,9,11)] px-4 pt-3 pb-4",
                        if open.get() { "block" } else { "hidden" },
                        "min-[1100px]:static min-[1100px]:block min-[1100px]:border-0 min-[1100px]:bg-transparent min-[1100px]:p-0",
                    )
                }
            >
                <label class="flex h-10 cursor-text items-center gap-2 rounded-2xl border border-white/10 bg-black/25 px-3.5 text-accent-400 focus-within:border-accent-400/45">
                    <Icon icon=icons::SEARCH class="h-3.5 w-3.5" />
                    <input
                        node_ref=input
                        type="search"
                        class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-white outline-none placeholder:text-white/40"
                        prop:value=move || query.get()
                        placeholder="Search the manual"
                        aria-label="Search the manual"
                        on:focus=move |_| on_open.run(())
                        on:input=move |ev| {
                            query.set(event_target_value(&ev));
                            on_open.run(());
                        }
                        on:keydown=on_input_key
                    />
                </label>
                <Show when=move || open.get() && terms.with(|t| !t.is_empty())>
                    <SearchResults
                        hits=hits
                        terms=terms
                        list_ref=results
                        on_exit=Callback::new(move |()| {
                            if let Some(input) = input.get_untracked() {
                                let _ = input.focus();
                            }
                        })
                    />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn SearchResults(
    hits: Signal<Vec<SearchHit>>,
    terms: Signal<Vec<String>>,
    list_ref: NodeRef<Ol>,
    /// Called to leave the list, back to the search field.
    on_exit: Callback<()>,
) -> impl IntoView {
    let ctx = use_manual();

    let on_key = move |ev: web_sys::KeyboardEvent| {
        let links = links_in(list_ref.get_untracked());
        let active = document().active_element();
        let at_first = links
            .iter()
            .position(|link| active.as_ref().is_some_and(|a| a.is_same_node(Some(link))))
            .is_none_or(|i| i == 0);
        let key = ev.key();
        if key == "Escape" || (key == "ArrowUp" && at_first) {
            ev.prevent_default();
            on_exit.run(());
        } else if move_focus(&links, &key) {
            ev.prevent_default();
        }
    };

    view! {
        <div class=cn!(
            "mt-3 max-h-[calc(100vh_-_var(--manual-top)_-_var(--manual-bar)_-_6rem)] overflow-y-auto overscroll-contain",
            "min-[1100px]:absolute min-[1100px]:top-[calc(100%_+_0.5rem)] min-[1100px]:right-0 min-[1100px]:z-30 min-[1100px]:mt-0 min-[1100px]:max-h-[min(32rem,calc(100vh_-_var(--manual-top)_-_var(--manual-bar)_-_2rem))] min-[1100px]:w-[28rem] min-[1100px]:rounded-[1.375rem] min-[1100px]:p-2",
            "min-[1100px]:border min-[1100px]:border-white/10 min-[1100px]:bg-[rgb(6,14,16)] min-[1100px]:shadow-[0_16px_48px_rgba(0,0,0,0.5)]",
        )>
            <p class="mb-1 px-2.5 text-xs text-white/40" aria-live="polite">
                {move || {
                    let count = hits.with(Vec::len);
                    if count == 1 { "1 match".to_string() } else { format!("{count} matches") }
                }}
            </p>
            <ol node_ref=list_ref class="list-none" aria-label="Search results" on:keydown=on_key>
                {move || {
                    let terms = terms.get();
                    hits.get()
                        .into_iter()
                        .map(|SearchHit { entry, snippet }| {
                            let chapter_slug = entry.chapter.as_ref().map(|c| c.slug);
                            let section_slug = entry.section.as_ref().map(|s| s.slug.clone());
                            let path = match (&entry.chapter, &entry.section) {
                                (None, _) => "Overview".to_string(),
                                (Some(c), Some(_)) => format!("{}. {}", c.number, c.title),
                                (Some(c), None) => format!("{} {}", ctx.chapter_label, c.number),
                            };
                            let title = match entry.section.as_ref().filter(|s| !s.number.is_empty()) {
                                Some(section) => format!("{} {}", section.number, entry.title),
                                None => entry.title.to_string(),
                            };
                            let href = ctx.href(chapter_slug, section_slug.as_deref());
                            view! {
                                <li>
                                    <a
                                        class=cn!(
                                            "flex flex-col gap-0.5 rounded-[0.875rem] px-2.5 py-2 text-white/[0.58] hover:bg-white/5 focus-visible:bg-white/5",
                                            FOCUS,
                                        )
                                        href=href.clone()
                                        on:click=ctx.on_link(href)
                                    >
                                        <span class="text-[0.6875rem] uppercase tracking-[0.08em] text-white/40">
                                            {path}
                                        </span>
                                        <span class="text-sm font-medium text-white">
                                            {highlighted(&title, &terms)}
                                        </span>
                                        {snippet
                                            .map(|s| {
                                                view! {
                                                    <span class="text-xs leading-normal">
                                                        {highlighted(&s, &terms)}
                                                    </span>
                                                }
                                            })}
                                    </a>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ol>
        </div>
    }
}

fn highlighted(text: &str, terms: &[String]) -> AnyView {
    highlight(text, terms)
        .into_iter()
        .map(|(hit, part)| {
            if hit {
                view! {
                    <mark class="rounded-[0.2rem] bg-accent-400/[0.28] px-[0.1em] text-white">
                        {part.to_string()}
                    </mark>
                }
                .into_any()
            } else {
                part.to_string().into_any()
            }
        })
        .collect_view()
        .into_any()
}
