//! The /legal/* pages and the shared shell + prose primitives they render with.
//!
//! These need real URLs rather than modals: a receipt, a support reply and a
//! chargeback response can all cite a URL. Prose comes from infinity-ui (`P`,
//! `Lead`, `List`, `Callout`) so the legal pages read as siblings of the docs.

mod eula;
mod privacy;
mod refunds;
mod terms;

use infinity_ui::prelude::*;

pub use eula::{EulaDocument, EulaPage};
pub use privacy::PrivacyPolicy;
pub use refunds::RefundPolicy;
pub use terms::Terms;

/// Inline link styling shared by the legal pages and every surface that points
/// at them (product page disclosure, purchase pages, dashboard).
pub const LINK: &str = "text-accent-300 underline underline-offset-2 hover:text-accent-200";

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
        <section id=anchor class="scroll-mt-[8rem] space-y-3">
            <h2 class="text-lg font-bold tracking-tight text-accent-400">
                {format!("{n}. {title}")}
            </h2>
            {children()}
        </section>
    }
}

#[component]
pub fn LegalPage(
    #[prop(into)] title: String,
    /// e.g. "Version 1.0". Omitted for documents that aren't versioned.
    #[prop(into, optional)]
    version: Option<String>,
    /// Effective date, already formatted for display.
    #[prop(into, optional)]
    effective: Option<String>,
    /// Short standfirst rendered above the rule.
    #[prop(optional)]
    intro: Option<ViewFn>,
    children: Children,
) -> impl IntoView {
    let meta = [version, effective.map(|e| format!("Effective {e}"))]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");

    view! {
        <AnimatedPage class="min-h-screen px-4 py-12 text-white">
            <div class="mx-auto w-full max-w-3xl">
                <div class=glass("rounded-2xl p-6 md:p-10")>
                    <h1 class="text-3xl font-bold tracking-tight md:text-4xl">{title}</h1>
                    {(!meta.is_empty())
                        .then(|| {
                            view! {
                                <p class="mt-2 text-xs tracking-[0.16em] text-white/45 uppercase">
                                    {meta}
                                </p>
                            }
                        })}
                    {intro.map(|intro| view! { <div class="mt-6 space-y-3">{intro.run()}</div> })}
                    <div class="mt-8 space-y-10 border-t border-white/10 pt-8">{children()}</div>
                </div>
            </div>
        </AnimatedPage>
    }
}
