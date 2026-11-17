//! Link a Discord account to the signed-in website account.
//!
//! Linking is a full-page round trip: the worker hands back Discord's
//! authorize URL, the browser goes there, and Discord sends it back to the
//! page it came from with `?discord=linked|cancelled|taken|error`. Pages read
//! that once with [`take_link_result`] and show it with [`LinkResultNotice`].

use infinity_ui::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsValue;

use crate::api::{self, DiscordLink};
use crate::auth::use_auth;

/// How the last link attempt went, read from `?discord=` and then removed
/// from the URL so a reload doesn't show it again.
pub fn take_link_result() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let params = web_sys::UrlSearchParams::new_with_str(&location.search().ok()?).ok()?;
    let result = params.get("discord")?;
    params.delete("discord");

    let rest = String::from(params.to_string());
    let path = location.pathname().unwrap_or_default();
    let hash = location.hash().unwrap_or_default();
    let clean = if rest.is_empty() {
        format!("{path}{hash}")
    } else {
        format!("{path}?{rest}{hash}")
    };
    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&clean));
    }
    Some(result)
}

/// A left-ruled line saying how linking went.
#[component]
pub fn LinkResultNotice(result: String, #[prop(optional)] spacing: &'static str) -> impl IntoView {
    let (tone, title, message) = match result.as_str() {
        "linked" => (
            "border-accent-400/60 text-accent-100",
            "Discord connected",
            "You've been added to the Infinity Discord server.",
        ),
        "cancelled" => (
            "border-amber-400/60 text-amber-100",
            "Discord not connected",
            "Linking was cancelled on Discord. Try again whenever you're ready.",
        ),
        "taken" => (
            "border-red-400/60 text-red-200",
            "Discord not connected",
            "That Discord account is already linked to a different Infinity account.",
        ),
        _ => (
            "border-red-400/60 text-red-200",
            "Discord not connected",
            "Something went wrong linking Discord. Try again.",
        ),
    };
    view! {
        <div class=cn!("border-l pl-4 text-sm leading-relaxed", tone, spacing)>
            <p class="mb-1 font-mono text-[11px] tracking-[0.2em] uppercase">{title}</p>
            <p class="font-light opacity-80">{message}</p>
        </div>
    }
}

/// Connect button, or the linked username with a disconnect action.
///
/// `return_to` is the site path Discord sends the browser back to.
/// `on_change` runs after a disconnect, so the page can refetch.
#[component]
pub fn DiscordConnect(
    #[prop(into)] link: Signal<Option<DiscordLink>>,
    return_to: &'static str,
    #[prop(into, optional)] on_change: Option<Callback<()>>,
) -> impl IntoView {
    let auth = use_auth();
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let connect = move |()| {
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                busy.set(false);
                error.set(Some("Your session expired. Sign in again.".into()));
                return;
            };
            match api::start_discord_link(&token, return_to).await {
                // Stay busy: the page is about to navigate away.
                Ok(url) => {
                    if let Some(window) = web_sys::window()
                        && window.location().assign(&url).is_err()
                    {
                        busy.set(false);
                        error.set(Some("Couldn't open Discord.".into()));
                    }
                }
                Err(message) => {
                    busy.set(false);
                    error.set(Some(message));
                }
            }
        });
    };

    let disconnect = move |()| {
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                busy.set(false);
                error.set(Some("Your session expired. Sign in again.".into()));
                return;
            };
            let outcome = api::unlink_discord(&token).await;
            busy.set(false);
            match outcome {
                Ok(()) => {
                    if let Some(on_change) = on_change {
                        on_change.run(());
                    }
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    let linked_as = Signal::derive(move || {
        link.get()
            .filter(|link| link.linked)
            .map(|link| link.username.unwrap_or_default())
    });

    view! {
        <div class="flex flex-col gap-2">
            {move || match linked_as.get() {
                Some(username) => {
                    view! {
                        <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
                            <span class="inline-flex items-center gap-2 text-sm text-white/80">
                                <Icon icon=icons::CIRCLE_CHECK class="h-4 w-4 text-accent-300" />
                                "Connected as "
                                <span class="font-medium text-white">{format!("@{username}")}</span>
                            </span>
                            <Button
                                variant=Variant::Ghost
                                size=Size::Sm
                                icon=icons::CIRCLE_X
                                loading=busy
                                on_click=Callback::new(disconnect)
                            >
                                "Disconnect"
                            </Button>
                        </div>
                    }
                        .into_any()
                }
                None => {
                    view! {
                        <div>
                            <Button
                                variant=Variant::Solid
                                icon=icons::LINK_2
                                loading=busy
                                on_click=Callback::new(connect)
                            >
                                "Connect Discord"
                            </Button>
                        </div>
                    }
                        .into_any()
                }
            }}
            {move || {
                error.get().map(|message| view! { <p class="text-sm text-red-300">{message}</p> })
            }}
        </div>
    }
}
