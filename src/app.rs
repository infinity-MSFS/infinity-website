use infinity_ui::prelude::*;
use infinity_ui::{AccountItem, AccountMenu, BackgroundGradientAnimation, TopNav};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;

use crate::auth::{provide_auth, use_auth};
use crate::brand::{GLOW_NAV, InfinityMark};
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
                    <div class="min-h-screen bg-gradient-to-br from-gray-900 via-blue-900 to-gray-900">

                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <BackgroundGradientAnimation>
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
        <div class=move || {
                   cn!(
                       "relative h-screen w-screen overflow-x-hidden bg-black/80",
                       if is_home.get() { "overflow-y-hidden" } else { "overflow-y-auto" },
                   )
               }>
                   <SiteNav />
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
                       <Route path=path!("/purchase-success") view=pages::PurchaseSuccess />
                       <Route path=path!("/purchase-cancelled") view=pages::PurchaseCancelled />
                       <Route path=path!("/ata") view=pages::AtaDocuments />
                       <Route path=path!("/ata/:chapter") view=pages::AtaDocuments />
                   </Routes>
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
            items=items
            active=active
            on_select=go
            brand=ViewFn::from(|| {
                view! {
                    <InfinityMark class="w-9 h-9" glow=GLOW_NAV />
                    <span class="hidden sm:inline">"Infinity"</span>
                }
            })
            actions=ViewFn::from(move || view! { <AccountControls /> })
            mobile_actions=ViewFn::from(move || view! { <AccountControls inline=true /> })
        />
    }
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
