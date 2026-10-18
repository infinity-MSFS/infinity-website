use infinity_ui::prelude::*;
use infinity_ui::{AccountItem, AccountMenu, BackgroundGradientAnimation, TopNav};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;

use crate::auth::{provide_auth, use_auth};
use crate::background;
use crate::brand::{COPYRIGHT_YEAR, InfinityMark, PUBLISHER};
use crate::nav;
use crate::pages;
use crate::state::provide_remote_state;

#[component]
pub fn App() -> impl IntoView {
    let low_performance = webgl_support().is_low_performance();
    provide_auth();
    provide_remote_state();

    view! {
        <Router>
            {if low_performance {
                view! {
                    <div class="min-h-screen" style=format!("background: {};", background::ramp())>
                        <Chrome/>
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <BackgroundGradientAnimation palette=background::PALETTE>
                        <Chrome/>
                    </BackgroundGradientAnimation>
                }
                    .into_any()
            }}
        </Router>
    }
}

#[component]
fn Chrome() -> impl IntoView {
    let location = use_location();
    let path = Signal::derive(move || location.pathname.get());
    let is_home = Signal::derive(move || path.get() == nav::HOME);

    Effect::new(move |_| {
        let Some(body) = document().body() else {
            return;
        };
        let _ = web_sys::HtmlElement::style(&body)
            .set_property("overflow", if is_home.get() { "hidden" } else { "auto" });
    });

    view! {
        // `flex flex-col` + a `flex-1` routes wrapper is what keeps SiteFooter
        // at the bottom of a fixed-height (h-screen) scroll container. Without
        // it a short route renders the footer mid-screen with dead space below.
        <div class=move || {
                   cn!(
                       "relative flex h-screen w-screen flex-col overflow-x-hidden bg-black/80",
                       if is_home.get() { "overflow-y-hidden" } else { "overflow-y-auto" },
                   )
               }>
                   <SiteNav />
                   <div class="flex-1">
                       <Routes fallback=|| view! { <pages::NotFound /> }>
                           <Route path=path!("/") view=pages::Home />
                           <Route path=path!("/about") view=pages::Manager />
                           <Route path=path!("/aircraft") view=pages::AircraftListing />
                           <Route path=path!("/aircraft/:slug") view=pages::AircraftDetail />
                           <Route path=path!("/docs") view=pages::DocsLanding />
                           <Route path=path!("/docs/developer") view=pages::DeveloperDocs />
                           <Route path=path!("/docs/developer/:slug") view=pages::DeveloperDocs />
                           <Route path=path!("/user") view=pages::UserDashboard />
                           <Route path=path!("/beta") view=pages::BetaSignup />
                           <Route path=path!("/beta/admin") view=pages::BetaAdmin />
                           <Route path=path!("/beta/sign/:doc") view=pages::BetaSignDocument />
                           <Route path=path!("/purchase-success") view=pages::PurchaseSuccess />
                           <Route path=path!("/purchase-cancelled") view=pages::PurchaseCancelled />
                           // Legal. The EULA is per-slug because it interpolates
                           // catalog fields.
                           <Route path=path!("/legal/refunds") view=pages::RefundPolicy />
                           <Route path=path!("/legal/privacy") view=pages::PrivacyPolicy />
                           <Route path=path!("/legal/terms") view=pages::Terms />
                           <Route path=path!("/legal/eula/:slug") view=pages::EulaPage />
                           <Route path=path!("/ata") view=pages::AtaDocuments />
                           <Route path=path!("/ata/:chapter") view=pages::AtaDocuments />
                           // One route with an optional chapter, so moving between the
                           // overview and a chapter doesn't remount the reader.
                           <Route path=path!("/manual/:manual/:chapter?") view=crate::manual::ManualPage />
                       </Routes>
                   </div>
                   <Show when=move || !is_home.get()>
                       <SiteFooter />
                   </Show>
               </div>
    }
}

#[component]
fn SiteNav() -> impl IntoView {
    let go = nav::use_go();
    let location = use_location();
    let active = Signal::derive(move || location.pathname.get());
    let items = Signal::derive(nav::top_nav_items);

    view! {
        <TopNav
            // `site-nav` scopes the tab overrides in input.css.
            class="site-nav shrink-0"
            items=items
            active=active
            on_select=go
            brand=ViewFn::from(|| {
                view! {
                    <InfinityMark class="h-7 w-7" glow="none" />
                    <span class="hidden text-lg font-light tracking-tight sm:inline">"Infinity"</span>
                }
            })
            actions=ViewFn::from(move || {
                view! {
                    <GithubLink />
                    <AccountControls />
                }
            })
            mobile_actions=ViewFn::from(move || {
                view! {
                    <GithubLink inline=true />
                    <AccountControls inline=true />
                }
            })
        />
    }
}

const FOOTER_LINK: &str =
    "font-mono text-[11px] tracking-[0.2em] text-white/45 uppercase transition-colors hover:text-accent-300";

/// Site-wide footer, and the only place the legal pages are reachable from
/// every route. The nav deliberately doesn't carry them: they aren't
/// destinations anyone browses to.
///
/// Not rendered on the home route — it locks `overflow-y-hidden` on both body
/// and the app container, so a footer there would be unreachable anyway.
#[component]
fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="relative z-10 mt-16 shrink-0 border-t border-white/10">
            <div class="mx-auto max-w-6xl px-6 py-8 md:px-10">
                <div class="flex flex-col gap-5 md:flex-row md:items-center md:justify-between">
                    <a href=nav::HOME class="flex items-center gap-2.5 text-white/80 transition-colors hover:text-white">
                        <InfinityMark class="h-5 w-5" glow="none" />
                        <span class="text-sm font-extralight tracking-[0.2em] uppercase">"Infinity"</span>
                    </a>
                    <nav class="flex flex-wrap items-center gap-x-6 gap-y-2">
                        <a href=nav::LEGAL_REFUNDS class=FOOTER_LINK>
                            "Refund Policy"
                        </a>
                        <a href=nav::LEGAL_PRIVACY class=FOOTER_LINK>
                            "Privacy"
                        </a>
                        <a href=nav::LEGAL_TERMS class=FOOTER_LINK>
                            "Terms"
                        </a>
                        <a href=nav::AIRCRAFT class=FOOTER_LINK>
                            "Aircraft"
                        </a>
                        <a
                            href=nav::DISCORD_URL
                            target="_blank"
                            rel="noopener noreferrer"
                            class=FOOTER_LINK
                        >
                            "Discord"
                        </a>
                    </nav>
                </div>
                <p class="mt-6 border-t border-white/5 pt-5 text-xs font-light text-white/35">
                    {format!("© {COPYRIGHT_YEAR} {PUBLISHER}. All rights reserved.")}
                </p>
                <p class="mt-1 text-xs font-light text-white/35">
                    {format!(
                        "Microsoft Flight Simulator is a trademark of Microsoft Corporation. {PUBLISHER} is not affiliated with, endorsed by, or sponsored by Microsoft Corporation.",
                    )}
                </p>
            </div>
        </footer>
    }
}

/// Link out to the GitHub organisation: an icon in the desktop bar, a labelled
/// row in the mobile panel.
#[component]
fn GithubLink(#[prop(optional)] inline: bool) -> impl IntoView {
    if inline {
        return view! {
            <a
                href=nav::GITHUB_ORG
                target="_blank"
                rel="noreferrer noopener"
                class="flex items-center gap-2 rounded-lg px-3 py-2.5 text-base font-medium text-slate-300 transition-colors hover:bg-white/[0.05] hover:text-white"
            >
                <Icon icon=icons::GITHUB class="h-4 w-4" />
                "GitHub"
                <Icon icon=icons::EXTERNAL_LINK class="ml-auto h-3.5 w-3.5 opacity-50" />
            </a>
        }
            .into_any();
    }

    view! {
        <Tooltip text="Our code on GitHub" placement=Placement::Bottom>
            <a
                href=nav::GITHUB_ORG
                target="_blank"
                rel="noreferrer noopener"
                aria-label="Infinity on GitHub"
                class="inline-flex h-9 w-9 items-center justify-center rounded-lg text-white/60 transition-colors hover:bg-white/[0.06] hover:text-white"
            >
                <Icon icon=icons::GITHUB class="h-[18px] w-[18px]" />
            </a>
        </Tooltip>
    }
        .into_any()
}

#[component]
fn AccountControls(#[prop(optional)] inline: bool) -> impl IntoView {
    let go = nav::use_go();
    let auth = use_auth();

    let menu_items = Signal::derive(move || {
        vec![
            AccountItem::new("dashboard", "Account Dashboard").icon(icons::USER),
            AccountItem::new("logout", "Sign Out")
                .icon(icons::LOG_OUT)
                .tone(Tone::Danger),
        ]
    });

    let on_select = Callback::new(move |id: String| match id.as_str() {
        "dashboard" => go.run(nav::USER.to_string()),
        "logout" => auth.logout(),
        _ => {}
    });

    view! {
        {move || {
            if auth.loading.get() {
                return view! {
                    <div class=cn!(
                        "flex items-center gap-3",
                        inline.then_some("px-3 py-2"),
                    )>
                        <Skeleton class="h-4 w-20" />
                        <Skeleton class="h-8 w-8 rounded-full" />
                    </div>
                }
                    .into_any();
            }

            match auth.user.get() {
                Some(user) => {
                    let developer = auth.entitlements.get().is_developer();
                    view! {
                        <AccountMenu
                            name=user.display_name()
                            email=user.email.clone()
                            picture=user.picture.clone()
                            badge=developer.then(|| ("Dev".to_string(), Tone::Dev))
                            items=menu_items
                            on_select=on_select
                            inline=inline
                        />
                    }
                        .into_any()
                }
                None => {
                    view! {
                        <Button
                            variant=Variant::Solid
                            icon=icons::LOG_IN
                            full_width=inline
                            on_click=Callback::new(move |()| auth.login())
                        >
                            "Sign In"
                        </Button>
                    }
                        .into_any()
                }
            }
        }}
    }
}
