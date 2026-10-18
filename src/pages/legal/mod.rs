//! The /legal/* pages and the shared shell + prose primitives they render with.
//!
//! These need real URLs rather than modals: a receipt, a support reply and a
//! chargeback response can all cite a URL. The prose primitives (`P`, `Lead`,
//! `List`, `ListItem`, `Callout`) mirror infinity-ui's names and props so the
//! documents read the same in source, but draw in the site's editorial style:
//! hairlines and mono labels rather than glass cards. Each page imports them
//! from here explicitly, which shadows the prelude's versions.

mod eula;
mod privacy;
mod refunds;
mod terms;

use infinity_ui::prelude::*;

use crate::components::EYEBROW;

pub use eula::{EulaDocument, EulaPage};
pub use privacy::PrivacyPolicy;
pub use refunds::RefundPolicy;
pub use terms::Terms;

/// Inline link styling shared by the legal pages and every surface that points
/// at them (product page disclosure, purchase pages, dashboard).
pub const LINK: &str = "text-accent-300 underline decoration-accent-300/40 underline-offset-4 transition-colors hover:text-accent-200 hover:decoration-accent-200";

/// An in-site link. A plain anchor is enough: the router intercepts same-origin
/// clicks.
#[component]
pub fn Internal(href: &'static str, children: Children) -> impl IntoView {
    view! {
        <a href=href class=LINK>
            {children()}
        </a>
    }
}

#[component]
pub fn External(href: &'static str, children: Children) -> impl IntoView {
    view! {
        <a href=href target="_blank" rel="noopener noreferrer" class=LINK>
            {children()}
        </a>
    }
}

/// The standfirst: the one paragraph that carries the accent rule.
#[component]
pub fn Lead(children: Children) -> impl IntoView {
    view! {
        <p class="max-w-2xl border-l border-accent-400/40 pl-5 text-base leading-relaxed font-light text-white/65 md:text-lg">
            {children()}
        </p>
    }
}

#[component]
pub fn P(children: Children) -> impl IntoView {
    view! { <p>{children()}</p> }
}

#[component]
pub fn List(children: Children) -> impl IntoView {
    view! { <ul class="space-y-3">{children()}</ul> }
}

/// A short accent dash stands in for the bullet.
#[component]
pub fn ListItem(children: Children) -> impl IntoView {
    view! {
        <li class="relative pl-6">
            <span class="absolute top-[0.8rem] left-0 h-px w-3 bg-accent-400/60"></span>
            {children()}
        </li>
    }
}

/// A set-apart note: a mono label over a hairline, toned only in the label.
#[component]
pub fn Callout(
    #[prop(optional)] tone: Tone,
    #[prop(into, optional)] title: Option<String>,
    children: Children,
) -> impl IntoView {
    let (rule, label) = match tone {
        Tone::Warning => ("border-amber-300/50", "text-amber-300/90"),
        Tone::Danger => ("border-red-400/50", "text-red-300"),
        _ => ("border-accent-400/50", "text-accent-400/80"),
    };

    view! {
        <aside class=cn!("border-l py-1 pl-5", rule)>
            {title
                .map(|t| {
                    view! {
                        <p class=cn!(
                            "mb-3 font-mono text-[11px] tracking-[0.25em] uppercase", label
                        )>{t}</p>
                    }
                })}
            <div class="space-y-3">{children()}</div>
        </aside>
    }
}

/// A numbered clause. `anchor` becomes the element id, so the section is
/// deep-linkable from a receipt or a support reply.
#[component]
pub fn Section(
    n: &'static str,
    anchor: &'static str,
    title: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section id=anchor class="scroll-mt-[8rem]">
            <div class=cn!("mb-5 flex items-center gap-4", EYEBROW)>
                <span class="text-accent-400/80">{format!("{n:0>2}")}</span>
                <span class="hidden truncate sm:inline">{format!("#{anchor}")}</span>
                <span class="h-px flex-1 bg-white/10"></span>
            </div>
            <h2 class="text-2xl font-light tracking-tight">{title}</h2>
            <div class="mt-5 space-y-4">{children()}</div>
        </section>
    }
}

/// Body copy for every legal surface, including the EULA inside the product
/// page's modal: light, readable, with bold and code lifted off the grey.
pub const PROSE: &str = "font-light text-white/72 [&_code]:font-mono [&_code]:text-[0.9em] [&_code]:text-white/85 [&_strong]:font-medium [&_strong]:text-white/90";

#[component]
pub fn LegalPage(
    #[prop(into)] title: String,
    /// e.g. "Version 1.0". Omitted for documents that aren't versioned.
    #[prop(into, optional)]
    version: Option<String>,
    /// Effective date, already formatted for display.
    #[prop(into, optional)]
    effective: Option<String>,
    /// Short standfirst rendered above the first section.
    #[prop(optional)]
    intro: Option<ViewFn>,
    children: Children,
) -> impl IntoView {
    let meta = [version, effective.map(|e| format!("Effective {e}"))]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("  /  ");

    view! {
        <AnimatedPage class="min-h-screen text-white">
            <div class="mx-auto max-w-6xl px-6 py-10 md:px-10 md:py-14">
                <header class="mb-16 max-w-3xl">
                    <p class=cn!("mb-10", EYEBROW)>"Infinity / Legal"</p>
                    <h1 class="text-4xl leading-[1.05] font-extralight tracking-tight md:text-6xl">
                        {title}
                    </h1>
                    {(!meta.is_empty())
                        .then(|| {
                            view! { <p class="mt-6 font-mono text-[11px] text-white/40">{meta}</p> }
                        })}
                    {intro
                        .map(|intro| {
                            view! {
                                <div class=cn!(
                                    "mt-8 space-y-6 text-[15px] leading-7", PROSE
                                )>{intro.run()}</div>
                            }
                        })}
                </header>
                <div class=cn!("max-w-3xl space-y-16 text-[15px] leading-7", PROSE)>
                    {children()}
                </div>
            </div>
        </AnimatedPage>
    }
}
