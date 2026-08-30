//! This is gonna be the only messy part of the port, since auth0 doesnt ship any wasm bindings natively, we will create some js helpers in auth0.js then bind that in wasm here

use std::collections::BTreeSet;

use infinity_ui::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

pub const AUTH0_DOMAIN: &str = env!("AUTH0_DOMAIN");
pub const AUTH0_CLIENT_ID: &str = env!("AUTH0_CLIENT_ID");
pub const AUTH0_AUDIENCE: &str = env!("AUTH0_AUDIENCE");

#[wasm_bindgen(module = "/auth0.js")]
extern "C" {
    #[wasm_bindgen(js_name = "init")]
    async fn js_init(domain: &str, client_id: &str, audience: &str) -> JsValue;

    #[wasm_bindgen(js_name = "login")]
    async fn js_login(return_to: Option<String>);

    #[wasm_bindgen(js_name = "logout")]
    async fn js_logout();

    #[wasm_bindgen(js_name = "token")]
    async fn js_token(force: bool) -> JsValue;

    #[wasm_bindgen(js_name = "refresh")]
    async fn js_refresh() -> JsValue;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct User {
    pub sub: Option<String>,
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
}

impl User {
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.nickname.clone())
            .or_else(|| self.email.clone())
            .unwrap_or_else(|| "Account".to_string())
    }

    pub fn connection(&self) -> Option<&str> {
        self.sub.as_deref()?.split('|').next()
    }

    pub fn is_database_user(&self) -> bool {
        self.connection() == Some("auth0")
    }
}

const ROLE_CLAIMS: [&str; 1] = ["developer"];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entitlements {
    pub products: Vec<String>,
    pub channels: Vec<String>,
    pub roles: Vec<String>,
}

impl Entitlements {
    fn from_permissions(permissions: Vec<String>) -> Self {
        let mut products = Vec::new();
        let mut channels = BTreeSet::from(["stable".to_string()]);
        let mut roles = Vec::new();
        for permission in permissions {
            if let Some(channel) = permission.strip_prefix("channel:") {
                if !channel.is_empty() {
                    channels.insert(channel.to_string());
                }
            } else if ROLE_CLAIMS.contains(&permission.as_str()) {
                roles.push(permission);
            } else {
                products.push(permission);
            }
        }
        Self {
            products,
            channels: channels.into_iter().collect(),
            roles,
        }
    }

    pub fn is_developer(&self) -> bool {
        self.roles.iter().any(|role| role == "developer")
    }

    pub fn owns_any(&self, ids: &[String]) -> bool {
        ids.iter().any(|id| self.products.contains(id))
    }
}

/// JWT verification is all handled server side thus we just decode it here
fn permissions_from_token(token: &str) -> Vec<String> {
    #[derive(Deserialize)]
    struct Claims {
        #[serde(default)]
        permissions: Vec<String>,
    }

    let Some(payload) = token.split('.').nth(1) else {
        return Vec::new();
    };

    let normalised = payload.replace('-', "+").replace('_', "/");
    let padded = match normalised.len() % 4 {
        0 => normalised,
        n => format!("{normalised}{}", "=".repeat(4 - n)),
    };
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let Ok(binary) = window.atob(&padded) else {
        return Vec::new();
    };
    let bytes: Vec<u8> = binary.chars().map(|c| c as u32 as u8).collect();
    serde_json::from_slice::<Claims>(&bytes)
        .map(|claims| claims.permissions)
        .unwrap_or_default()
}

#[derive(Clone, Copy)]
pub struct Auth {
    pub user: RwSignal<Option<User>>,
    pub entitlements: RwSignal<Entitlements>,
    pub loading: RwSignal<bool>,
    pub is_authenticated: Signal<bool>,
    pub syncing: RwSignal<bool>,
}

impl Auth {
    pub fn login(&self) {
        spawn_local(async move {
            js_login(None).await;
        });
    }

    pub fn logout(&self) {
        let user = self.user;
        let entitlements = self.entitlements;
        spawn_local(async move {
            js_logout().await;
            user.set(None);
            entitlements.set(Entitlements::default());
        })
    }

    pub async fn token(&self) -> Option<String> {
        js_token(false).await.as_string()
    }

    pub async fn refresh_entitlements(&self, force: bool) {
        let Some(token) = js_token(force).await.as_string() else {
            self.entitlements.set(Entitlements::default());
            return;
        };
        self.entitlements
            .set(Entitlements::from_permissions(permissions_from_token(
                &token,
            )));
    }

    /// pull for new token after a purchase so the user can see their product right away
    pub fn sync_until_granted(&self, target: Option<String>) {
        const BACKOFF_MS: [u64; 10] = [
            2000, 2000, 4000, 4000, 8000, 8000, 15000, 15000, 30000, 30000,
        ];

        let auth = *self;
        spawn_local(async move {
            auth.syncing.set(true);
            let baseline: BTreeSet<String> = auth
                .entitlements
                .get_untracked()
                .products
                .into_iter()
                .collect();

            let granted = |entitlements: &Entitlements| match target.as_deref() {
                Some(id) => entitlements.products.iter().any(|p| p == id),
                None => entitlements.products.iter().any(|p| !baseline.contains(p)),
            };

            auth.refresh_entitlements(false).await;
            if granted(&auth.entitlements.get_untracked()) {
                auth.syncing.set(false);
                return;
            }

            for delay in BACKOFF_MS {
                sleep(delay).await;
                auth.refresh_entitlements(true).await;
                if granted(&auth.entitlements.get_untracked()) {
                    break;
                }
            }
            auth.syncing.set(false);
        })
    }
}

async fn sleep(ms: u64) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

pub fn provide_auth() -> Auth {
    let user = RwSignal::new(None::<User>);
    let entitlements = RwSignal::new(Entitlements::default());
    let loading = RwSignal::new(true);
    let syncing = RwSignal::new(false);
    let is_authenticated = Signal::derive(move || user.with(Option::is_some));

    let auth = Auth {
        user,
        entitlements,
        loading,
        is_authenticated,
        syncing,
    };
    provide_context(auth);

    spawn_local(async move {
        let session = js_init(AUTH0_DOMAIN, AUTH0_CLIENT_ID, AUTH0_AUDIENCE).await;
        if let Some(json) = session.as_string() {
            user.set(serde_json::from_str::<User>(&json).ok());
            auth.refresh_entitlements(false).await;
        }
        loading.set(false);
    });

    let visibility = window_event_listener(leptos::ev::visibilitychange, move |_| {
        if !auth.is_authenticated.get_untracked() {
            return;
        }
        if !document().hidden() {
            spawn_local(async move {
                auth.refresh_entitlements(false).await;
            });
        }
    });
    on_cleanup(move || visibility.remove());
    auth
}

pub fn use_auth() -> Auth {
    expect_context::<Auth>()
}
