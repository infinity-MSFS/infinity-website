use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub const WORKER_URL: &str = match option_env!("CDN_WORKER_URL") {
    Some(url) if !url.is_empty() => url,
    _ => "https://manager.infinity-simulations.com",
};
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
    // The legacy `showManual` flag is deliberately not read: it hid a "manual
    // coming soon" placeholder, a state that no longer exists now the manual
    // ships in the in-sim EFB. Drop the manual surfaces with
    // `manualLocation: ""` instead.
    /// Where this aircraft's flight manual lives. `None` means the in-sim EFB
    /// wording shared by every Infinity aircraft; an empty string hides the
    /// manual tile and card entirely. See [`AircraftProduct::manual_location`].
    #[serde(default)]
    pub manual_location: Option<String>,
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

/// Where every Infinity aircraft carries its manual unless the catalog says
/// otherwise.
pub const DEFAULT_MANUAL_LOCATION: &str = "In the cockpit EFB, under the Aircraft tab";

impl AircraftProduct {
    /// The manual location to show, or `None` when the product has no manual.
    pub fn manual_location(&self) -> Option<&str> {
        let location = self
            .manual_location
            .as_deref()
            .unwrap_or(DEFAULT_MANUAL_LOCATION);
        (!location.trim().is_empty()).then_some(location)
    }

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

async fn delete_json<T: for<'de> Deserialize<'de>>(
    base: &str,
    path: &str,
    token: &str,
    fallback: &str,
) -> Result<T, String> {
    let url = format!("{base}{path}");
    let response = gloo_net::http::Request::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(read_error(response, fallback).await.1);
    }
    response.json::<T>().await.map_err(|e| e.to_string())
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

/// The beta programme this site currently runs. The worker is programme-aware
/// (`/v1/beta/:program/…`) so a second aircraft only needs a new id here.
pub const BETA_PROGRAM: &str = "dc10";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BetaStatus {
    /// No application on file for this account.
    #[default]
    None,
    Requested,
    /// Passed review: the documents are unlocked, but this is not yet a
    /// tester. Final approval comes after the paperwork is checked.
    Accepted,
    /// Paperwork checked and the beta role granted. They're in.
    Approved,
    Rejected,
    Withdrawn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct QuizScore {
    #[serde(default)]
    pub correct: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub percent: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct QuizChoice {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct QuizQuestion {
    pub id: String,
    #[serde(default)]
    pub topic: String,
    pub prompt: String,
    #[serde(default)]
    pub choices: Vec<QuizChoice>,
}

/// A written question. Never graded — it exists so a reviewer can hear an
/// applicant think, which a multiple-choice paper cannot test.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenQuestion {
    pub id: String,
    pub prompt: String,
    #[serde(default)]
    pub hint: Option<String>,
    #[serde(default)]
    pub min_length: usize,
    #[serde(default)]
    pub required: bool,
}

/// The paper as served: the answer key stays on the worker, so nothing here
/// tells the client which choice is right.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizPaper {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub questions: Vec<QuizQuestion>,
    #[serde(default)]
    pub open_questions: Vec<OpenQuestion>,
}

/// Signing state for one document, as reported alongside the beta status.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaDocumentState {
    pub id: String,
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub signed: bool,
    #[serde(default)]
    pub signed_at: Option<String>,
    /// Set when an older version was signed and the document has since changed.
    #[serde(default)]
    pub superseded_version: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaState {
    #[serde(default)]
    pub aircraft: String,
    #[serde(default)]
    pub status: BetaStatus,
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub submitted_at: Option<String>,
    #[serde(default)]
    pub decision_note: Option<String>,
    #[serde(default)]
    pub score: Option<QuizScore>,
    /// Populated only once accepted; empty at every other status.
    #[serde(default)]
    pub documents: Vec<BetaDocumentState>,
    #[serde(default)]
    pub documents_complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaContact {
    pub display_name: String,
    pub email: String,
    pub discord: String,
    pub timezone: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizAnswer {
    pub question_id: String,
    pub choice_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAnswer {
    pub question_id: String,
    pub answer: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaApplication {
    pub contact: BetaContact,
    pub why_you: String,
    pub answers: Vec<QuizAnswer>,
    pub open_answers: Vec<OpenAnswer>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaReceipt {
    pub application_id: String,
    #[serde(default)]
    pub score: QuizScore,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DocumentSection {
    pub heading: String,
    #[serde(default)]
    pub body: Vec<String>,
}

/// A document with its full text. Only served to accepted testers, and never
/// cached client-side: a revision must reach everyone the moment it deploys.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaDocument {
    pub id: String,
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub effective_date: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub sections: Vec<DocumentSection>,
    #[serde(default)]
    pub signed: bool,
    #[serde(default)]
    pub signed_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaDocuments {
    /// The name on the application — shown as the expected signer.
    #[serde(default)]
    pub signer_name: String,
    /// Details from a previous signature, so the second document doesn't ask
    /// for the same address twice.
    #[serde(default)]
    pub signer: Option<SignerIdentity>,
    #[serde(default)]
    pub documents: Vec<BetaDocument>,
    #[serde(default)]
    pub documents_complete: bool,
}

/// Where a signer can actually be reached. A binding agreement needs a party
/// that can be identified and served, which a display name and a Discord
/// handle are not.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostalAddress {
    pub line1: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line2: Option<String>,
    pub city: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    pub postal_code: String,
    pub country: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerIdentity {
    pub legal_name: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub address: PostalAddress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SignatureMethod {
    Drawn,
    Typed,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureSubmission {
    /// Echoed back so the worker can reject a signature against stale wording.
    pub document_version: String,
    pub typed_name: String,
    /// Snapshotted onto the receipt, so a later edit can't rewrite what a
    /// past signature claimed.
    pub signer: SignerIdentity,
    pub method: SignatureMethod,
    /// `data:image/png;base64,…` from the signature pad. Absent when typed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_image: Option<String>,
    pub agreed: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignReceipt {
    pub receipt_id: String,
    pub signed_at: String,
    #[serde(default)]
    pub documents_complete: bool,
}

pub async fn beta_status(token: &str) -> Result<BetaState, String> {
    get_json(
        WORKER_URL,
        &format!("/v1/beta/{BETA_PROGRAM}/status"),
        Some(token),
    )
    .await
}

pub async fn beta_quiz(token: &str) -> Result<QuizPaper, String> {
    get_json(
        WORKER_URL,
        &format!("/v1/beta/{BETA_PROGRAM}/quiz"),
        Some(token),
    )
    .await
}

pub async fn submit_beta_application(
    token: &str,
    application: &BetaApplication,
) -> Result<BetaReceipt, String> {
    post_json(
        WORKER_URL,
        &format!("/v1/beta/{BETA_PROGRAM}/apply"),
        token,
        application,
        "Couldn't submit your application",
    )
    .await
    .map_err(|(_, message)| message)
}

pub async fn beta_documents(token: &str) -> Result<BetaDocuments, String> {
    get_json(
        WORKER_URL,
        &format!("/v1/beta/{BETA_PROGRAM}/documents"),
        Some(token),
    )
    .await
}

pub async fn sign_beta_document(
    token: &str,
    document_id: &str,
    submission: &SignatureSubmission,
) -> Result<SignReceipt, String> {
    post_json(
        WORKER_URL,
        &format!("/v1/beta/{BETA_PROGRAM}/documents/{document_id}/sign"),
        token,
        submission,
        "Couldn't record your signature",
    )
    .await
    .map_err(|(_, message)| message)
}

// --- Beta review (developer-gated) -----------------------------------------

/// One graded answer with the correct choice attached, so a reviewer can see
/// *what* someone got wrong rather than only a percentage.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAnswer {
    pub question_id: String,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    /// Absent when the applicant left the question blank.
    #[serde(default)]
    pub chosen: Option<String>,
    #[serde(default)]
    pub correct_choice: Option<String>,
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub correct: bool,
}

/// Signature evidence. The listing carries the metadata; `image_url` and the
/// audit fields only come back from the per-application detail endpoint, which
/// presigns the ink for a short window.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSignature {
    pub document_id: String,
    pub document_version: String,
    pub signed_at: String,
    pub method: String,
    pub receipt_id: String,
    #[serde(default)]
    pub typed_name: Option<String>,
    #[serde(default)]
    pub signer: Option<SignerIdentity>,
    #[serde(default)]
    pub document_hash: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminOpenAnswer {
    pub question_id: String,
    /// Absent if the question was removed from the programme since.
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminApplication {
    pub application_id: String,
    /// Auth0 subject — the id every admin action addresses the applicant by.
    pub sub: String,
    #[serde(default)]
    pub status: BetaStatus,
    #[serde(default)]
    pub submitted_at: String,
    #[serde(default)]
    pub decided_at: Option<String>,
    #[serde(default)]
    pub decision_note: Option<String>,
    pub contact: BetaContact,
    #[serde(default)]
    pub why_you: String,
    #[serde(default)]
    pub score: QuizScore,
    #[serde(default)]
    pub answers: Vec<AdminAnswer>,
    #[serde(default)]
    pub open_answers: Vec<AdminOpenAnswer>,
    #[serde(default)]
    pub signatures: Vec<AdminSignature>,
    #[serde(default)]
    pub documents_complete: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminApplications {
    #[serde(default)]
    pub applications: Vec<AdminApplication>,
    /// Set when R2 had more objects than one page; feed it back to continue.
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminClientMeta {
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminApplicationDetail {
    #[serde(default)]
    pub signatures: Vec<AdminSignature>,
    #[serde(default)]
    pub client_meta: AdminClientMeta,
}

#[derive(Clone, Debug, Serialize)]
pub struct BetaDecision {
    pub sub: String,
    pub status: BetaStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The worker writes the record before touching Auth0, so a decision can land
/// with the role grant still outstanding. `role_synced` is what says which.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionOutcome {
    /// Role the worker tried to assign or remove. `None` when the
    /// programme isn't in `BETA_PROGRAM_MAP` at all.
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub role_synced: bool,
    #[serde(default)]
    pub role_error: Option<String>,
}

pub async fn beta_applications(token: &str) -> Result<AdminApplications, String> {
    get_json(
        WORKER_URL,
        &format!("/v1/admin/beta/{BETA_PROGRAM}/applications"),
        Some(token),
    )
    .await
}

pub async fn beta_application(token: &str, sub: &str) -> Result<AdminApplicationDetail, String> {
    // Subjects contain `|`, which has to survive as a query value.
    let encoded = String::from(js_sys::encode_uri_component(sub));
    get_json(
        WORKER_URL,
        &format!("/v1/admin/beta/{BETA_PROGRAM}/application?sub={encoded}"),
        Some(token),
    )
    .await
}

/// Erase an application outright so the account can apply again from scratch.
/// Destructive: the signed documents go with it. Rejecting is what keeps the
/// evidence — this is for putting a test account back to square one.
pub async fn remove_beta_application(token: &str, sub: &str) -> Result<(), String> {
    #[derive(Deserialize)]
    struct Ignored {}

    let encoded = String::from(js_sys::encode_uri_component(sub));
    delete_json::<Ignored>(
        WORKER_URL,
        &format!("/v1/admin/beta/{BETA_PROGRAM}/application?sub={encoded}"),
        token,
        "Couldn't remove that application",
    )
    .await
    .map(|_| ())
}

/// Fetch the executed agreements as a PDF. Returns the bytes and the filename
/// the worker chose, so the download lands with a sensible name.
pub async fn beta_export_pdf(
    token: &str,
    sub: &str,
    document_id: Option<&str>,
) -> Result<(Vec<u8>, String), String> {
    let encoded = String::from(js_sys::encode_uri_component(sub));
    let doc = document_id.unwrap_or("all");
    let url = format!("{WORKER_URL}/v1/admin/beta/{BETA_PROGRAM}/export?sub={encoded}&doc={doc}",);
    let response = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(read_error(response, "Couldn't build that PDF").await.1);
    }

    // The worker names the file; fall back to something sane if the header
    // didn't survive (a proxy stripping it, say).
    let filename = response
        .headers()
        .get("content-disposition")
        .and_then(|value| {
            value
                .split("filename=\"")
                .nth(1)?
                .split('"')
                .next()
                .map(str::to_string)
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("{BETA_PROGRAM}-{doc}.pdf"));

    let bytes = response.binary().await.map_err(|e| e.to_string())?;
    Ok((bytes, filename))
}

pub async fn decide_beta_application(
    token: &str,
    decision: &BetaDecision,
) -> Result<DecisionOutcome, String> {
    post_json(
        WORKER_URL,
        &format!("/v1/admin/beta/{BETA_PROGRAM}/decision"),
        token,
        decision,
        "Couldn't record that decision",
    )
    .await
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
