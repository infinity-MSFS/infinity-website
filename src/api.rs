use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub const WORKER_URL: &str = match option_env!("CDN_WORKER_URL") {
    Some(url) if !url.is_empty() => url,
    _ => "https://manager.infinity-simulations.com",
};
pub const USER_API_URL: &str = env!("USER_API_URL");
pub const SUPPORT_API_URL: &str = env!("SUPPORT_API_URL");

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ManagerPlatform {
    pub url: String,
    pub version: String,
    #[serde(default)]
    pub pub_date: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}
pub type Platforms = IndexMap<String, ManagerPlatform>;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ManagerLatest {
    pub channel: String,
    pub version: String,
    #[serde(default)]
    pub pub_date: Option<String>,
    #[serde(default)]
    pub platforms: Platforms,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    #[serde(rename = "type", default)]
    pub kind: String,
    pub url: String,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct SpecItem {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AircraftProduct {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub developer: String,
    #[serde(default)]
    pub card_image: String,
    #[serde(default)]
    pub entitlement_ids: Vec<String>,
    #[serde(default = "yes")]
    pub storefront: bool,
    #[serde(default)]
    pub collaborators: Option<String>,
    #[serde(default)]
    pub platform_note: Option<String>,
    #[serde(default)]
    pub price_display: Option<String>,
    #[serde(default)]
    pub short_description: Option<String>,
    #[serde(default)]
    pub long_description: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub specs: Vec<SpecItem>,
    #[serde(default)]
    pub media: Vec<MediaItem>,
    #[serde(default)]
    pub support_discord_url: Option<String>,
    #[serde(default)]
    pub tutorial_url: Option<String>,
    #[serde(default = "yes")]
    pub show_manual: bool,
    #[serde(default)]
    pub eula_company: Option<String>,
    #[serde(default)]
    pub eula_product_ref: Option<String>,
    #[serde(default)]
    pub eula_copyright: Option<String>,
    #[serde(default)]
    pub eula_disclaimer: Option<String>,
    #[serde(default)]
    pub eula_extra_sections: Vec<EulaSection>,
    #[serde(default)]
    pub owner_discount: Option<OwnerDiscount>,
    #[serde(default)]
    pub dev_only: bool,
    #[serde(default)]
    pub is_server_down: bool,
}

fn yes() -> bool {
    true
}

impl AircraftProduct {
    pub fn entitlements(&self) -> Vec<String> {
        if self.entitlement_ids.is_empty() {
            vec![self.slug.clone()]
        } else {
            self.entitlement_ids.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerDiscount {
    pub requires_product: String,
    #[serde(default)]
    pub price_cents: u32,
    #[serde(default)]
    pub price_display: Option<String>,
}

impl OwnerDiscount {
    pub fn display(&self) -> String {
        self.price_display
            .clone()
            .unwrap_or_else(|| format!("${:.2}", f64::from(self.price_cents) / 100.0))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EulaSection {
    pub heading: String,
    #[serde(default, deserialize_with = "one_or_many")]
    pub body: Vec<String>,
}

fn one_or_many<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(d)? {
        OneOrMany::One(text) => vec![text],
        OneOrMany::Many(list) => list,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteCatalog {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub products: Vec<AircraftProduct>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct ApiError {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
}

async fn read_error(response: gloo_net::http::Response, fallback: &str) -> (String, String) {
    let status = response.status();
    let body = response.json::<ApiError>().await.unwrap_or_default();
    let message = if body.message.is_empty() {
        format!("{fallback} ({status})")
    } else {
        body.message
    };
    (body.code, message)
}

fn bearer(
    request: gloo_net::http::RequestBuilder,
    token: Option<&str>,
) -> gloo_net::http::RequestBuilder {
    match token {
        Some(token) => request.header("Authorization", &format!("Bearer {token}")),
        None => request,
    }
}

async fn get_json<T: for<'de> Deserialize<'de>>(
    base: &str,
    path: &str,
    token: Option<&str>,
) -> Result<T, String> {
    let url = format!("{base}{path}");
    let response = bearer(gloo_net::http::Request::get(&url), token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(read_error(response, "Request failed").await.1);
    }
    response.json::<T>().await.map_err(|e| e.to_string())
}

async fn post_json<B: Serialize, T: for<'de> Deserialize<'de>>(
    base: &str,
    path: &str,
    token: &str,
    body: &B,
    fallback: &str,
) -> Result<T, (String, String)> {
    let url = format!("{base}{path}");
    let response = gloo_net::http::Request::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .json(body)
        .map_err(|e| ("internal".to_string(), e.to_string()))?
        .send()
        .await
        .map_err(|e| ("network".to_string(), e.to_string()))?;

    if !response.ok() {
        let (code, message) = read_error(response, fallback).await;
        let code = if code.is_empty() {
            "internal".to_string()
        } else {
            code
        };
        return Err((code, message));
    }
    response
        .json::<T>()
        .await
        .map_err(|e| ("internal".to_string(), e.to_string()))
}

pub async fn manager_latest() -> Result<ManagerLatest, String> {
    get_json(WORKER_URL, "/v1/manager/latest", None).await
}

pub async fn website_catalog(token: Option<&str>) -> Result<WebsiteCatalog, String> {
    get_json(WORKER_URL, "/v1/website/catalog", token).await
}

#[derive(Clone, Debug, Deserialize)]
pub struct Checkout {
    pub url: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub discounted: bool,
}

#[derive(Clone, Debug)]
pub struct CheckoutError {
    pub code: String,
    pub message: String,
}

pub async fn create_checkout(
    slug: &str,
    token: &str,
    email: Option<String>,
    name: Option<String>,
) -> Result<Checkout, CheckoutError> {
    #[derive(Serialize)]
    struct Prefill {
        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    }

    post_json(
        WORKER_URL,
        &format!("/v1/website/checkout/{slug}"),
        token,
        &Prefill { email, name },
        "Couldn't prepare your checkout",
    )
    .await
    .map_err(|(code, message)| CheckoutError { code, message })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BetaAccess {
    #[default]
    None,
    Requested,
    Accepted,
}

#[derive(Clone, Debug, Serialize)]
pub struct Hardware {
    pub cpu: Option<String>,
    pub gpu: Option<String>,
    pub ram_gb: Option<String>,
    pub storage: String,
    pub monitors: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BetaApplication {
    pub auth0_user_id: Option<String>,
    pub display_name: String,
    pub email: String,
    pub discord: String,
    pub timezone: String,
    pub experience: String,
    pub expected_usage: String,
    pub hardware: Hardware,
    pub can_discord_call: bool,
    pub can_screen_share: bool,
    pub willing_test_nightlies: bool,
    pub availability: Option<String>,
    pub testing_style: String,
    pub why_you: String,
    pub past_bug_reports: Option<String>,
    pub notes: Option<String>,
    #[serde(skip)]
    pub pre_release_ack: bool,
    #[serde(skip)]
    pub contact_ok: bool,
}

pub async fn beta_status(token: &str) -> Result<BetaAccess, String> {
    #[derive(Deserialize)]
    struct Status {
        #[serde(default)]
        beta_access: BetaAccess,
    }

    get_json::<Status>(USER_API_URL, "/beta/status", Some(token))
        .await
        .map(|status| status.beta_access)
}

pub async fn submit_beta_application(
    token: &str,
    application: &BetaApplication,
) -> Result<Option<String>, String> {
    #[derive(Serialize)]
    struct Payload<'a> {
        #[serde(flatten)]
        application: &'a BetaApplication,
        consents: Consents,
        client_meta: ClientMeta,
    }
    #[derive(Serialize)]
    struct Consents {
        pre_release_ack: bool,
        contact_ok: bool,
    }
    #[derive(Serialize)]
    struct ClientMeta {
        page: &'static str,
    }
    #[derive(Deserialize)]
    struct Accepted {
        #[serde(default)]
        application_id: Option<String>,
    }

    let payload = Payload {
        application,
        consents: Consents {
            pre_release_ack: application.pre_release_ack,
            contact_ok: application.contact_ok,
        },
        client_meta: ClientMeta {
            page: "closed-beta-signup",
        },
    };

    post_json::<_, Accepted>(
        USER_API_URL,
        "/beta/apply",
        token,
        &payload,
        "Couldn't submit your application",
    )
    .await
    .map(|accepted| accepted.application_id)
    .map_err(|(_, message)| message)
}

pub async fn change_password(token: &str, current: &str, next: &str) -> Result<(), String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body<'a> {
        current_password: &'a str,
        new_password: &'a str,
    }
    #[derive(Deserialize)]
    struct Ignored {}

    post_json::<_, Ignored>(
        WORKER_URL,
        "/v1/account/password",
        token,
        &Body {
            current_password: current,
            new_password: next,
        },
        "Couldn't change your password. Try again.",
    )
    .await
    .map(|_| ())
    .map_err(|(_, message)| message)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportTicket {
    pub name: String,
    pub email: String,
    pub discord_name: String,
    pub subject: String,
    pub message: String,
    pub user_email: String,
}

pub async fn submit_support_ticket(token: &str, ticket: &SupportTicket) -> Result<(), String> {
    #[derive(Deserialize)]
    struct Ignored {}

    post_json::<_, Ignored>(
        SUPPORT_API_URL,
        "/support",
        token,
        ticket,
        "Couldn't send your message",
    )
    .await
    .map(|_| ())
    .map_err(|(_, message)| message)
}

pub const MIN_PASSWORD_LENGTH: usize = 8;

pub fn provider_label(sub: Option<&str>) -> Option<&'static str> {
    match sub?.split('|').next()? {
        "google-oauth2" => Some("Google"),
        "windowslive" => Some("Microsoft"),
        "discord" => Some("Discord"),
        "github" => Some("GitHub"),
        "facebook" => Some("Facebook"),
        "apple" => Some("Apple"),
        "twitter" => Some("X"),
        _ => None,
    }
}
