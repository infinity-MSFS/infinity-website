//! E-signing surface for the closed beta paperwork.
//!
//! Stage two of the application: an accepted tester reads the NDA or the test
//! agreement here and signs it. Three things make this a signature rather than
//! a checkbox, and all three are deliberate:
//!
//!   * The text is fetched from the worker on every visit — there is no local
//!     copy that could drift from what the worker hashes into the receipt.
//!   * Signing unlocks only after the document has actually been scrolled to
//!     the end, so "I have read it" is at least plausibly true.
//!   * The version rendered is sent back with the signature; if the document
//!     changed while the page sat open, the worker rejects it rather than
//!     binding someone to words they never saw.
//!
//! The signature itself is drawn on a canvas or typed, and the worker stores
//! the ink alongside the timestamp, document hash, IP and user agent.

use infinity_ui::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;

use crate::api::{
    self, BetaDocument, BetaDocuments, PostalAddress, SignatureMethod, SignatureSubmission,
    SignerIdentity,
};
use crate::auth::use_auth;
use crate::nav;

/// How close to the bottom counts as "read it". A few pixels of slack keeps
/// fractional scroll heights and zoom levels from making the gate unreachable.
const SCROLL_SLACK: i32 = 24;

#[derive(Clone, PartialEq, Eq, Debug)]
enum Status {
    Idle,
    Sending,
    Failed(String),
    Signed {
        receipt: String,
        at: String,
        complete: bool,
    },
}

#[component]
pub fn BetaSignDocument() -> impl IntoView {
    let auth = use_auth();
    let go = nav::use_go();
    let params = use_params_map();
    let doc_id = Signal::derive(move || params.read().get("doc").unwrap_or_default());

    let bundle = RwSignal::new(None::<BetaDocuments>);
    let load_error = RwSignal::new(None::<String>);
    let status = RwSignal::new(Status::Idle);

    let method = RwSignal::new("draw".to_string());
    let typed_name = RwSignal::new(String::new());
    // The binding party. Prefilled from a previous signature where there is
    // one, so the second document doesn't ask for the same address twice.
    let legal_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let line1 = RwSignal::new(String::new());
    let line2 = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let region = RwSignal::new(String::new());
    let postal_code = RwSignal::new(String::new());
    let country = RwSignal::new(String::new());
    let agreed = RwSignal::new(false);
    let has_ink = RwSignal::new(false);
    let read_to_end = RwSignal::new(false);

    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let scroller = NodeRef::<leptos::html::Div>::new();

    let load = move || {
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                return;
            };
            match api::beta_documents(&token).await {
                Ok(loaded) => {
                    if typed_name.get_untracked().is_empty() {
                        typed_name.set(loaded.signer_name.clone());
                    }
                    if let Some(previous) = loaded.signer.clone() {
                        let fill = |field: RwSignal<String>, value: String| {
                            if field.get_untracked().trim().is_empty() {
                                field.set(value);
                            }
                        };
                        fill(legal_name, previous.legal_name);
                        fill(email, previous.email);
                        fill(phone, previous.phone.unwrap_or_default());
                        fill(line1, previous.address.line1);
                        fill(line2, previous.address.line2.unwrap_or_default());
                        fill(city, previous.address.city);
                        fill(region, previous.address.region.unwrap_or_default());
                        fill(postal_code, previous.address.postal_code);
                        fill(country, previous.address.country);
                    } else if legal_name.get_untracked().is_empty() {
                        legal_name.set(loaded.signer_name.clone());
                    }
                    load_error.set(None);
                    bundle.set(Some(loaded));
                }
                Err(err) => load_error.set(Some(err)),
            }
        });
    };

    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            return;
        }
        load();
    });

    let document = Signal::derive(move || {
        let id = doc_id.get();
        bundle.with(|b| {
            b.as_ref()
                .and_then(|b| b.documents.iter().find(|d| d.id == id).cloned())
        })
    });

    Effect::new(move |previous: Option<String>| {
        let id = doc_id.get();
        let _ = document.get();

        // Both documents live on this one route, so leptos swaps the param
        // without remounting: consent has to be cleared by hand, or the second
        // document arrives pre-agreed on the strength of reading the first.
        if previous.as_deref().is_some_and(|seen| seen != id) {
            agreed.set(false);
            read_to_end.set(false);
            status.set(Status::Idle);
            if let Some(el) = scroller.get_untracked() {
                el.set_scroll_top(0);
            }
        }

        // A document short enough to fit without scrolling can never fire a
        // scroll event, so the gate has to settle itself once the pane has been
        // measured, or it could never open.
        if let Some(el) = scroller.get()
            && el.scroll_height() <= el.client_height() + SCROLL_SLACK
        {
            read_to_end.set(true);
        }

        id
    });

    let on_scroll = move |_| {
        let Some(el) = scroller.get_untracked() else {
            return;
        };
        if el.scroll_top() + el.client_height() >= el.scroll_height() - SCROLL_SLACK {
            read_to_end.set(true);
        }
    };

    let sending = Signal::derive(move || status.get() == Status::Sending);
    let drawing = Signal::derive(move || method.get() == "draw");

    let optional = |field: RwSignal<String>| {
        let value = field.get().trim().to_string();
        (!value.is_empty()).then_some(value)
    };

    let blocker = move || -> Option<String> {
        if !read_to_end.get() {
            return Some("Scroll to the end of the document before signing.".into());
        }
        if legal_name.get().trim().chars().count() < 2 {
            return Some("Enter your full legal name.".into());
        }
        let address = email.get();
        if address.trim().is_empty() || !address.contains('@') {
            return Some("Enter a valid email address.".into());
        }
        if line1.get().trim().is_empty() {
            return Some("Enter your street address.".into());
        }
        if city.get().trim().is_empty() {
            return Some("Enter your city or town.".into());
        }
        if postal_code.get().trim().is_empty() {
            return Some("Enter your postal or ZIP code.".into());
        }
        if country.get().trim().is_empty() {
            return Some("Enter your country.".into());
        }
        if typed_name.get().trim().chars().count() < 2 {
            return Some("Type your name as your signature.".into());
        }
        if drawing.get() && !has_ink.get() {
            return Some("Draw your signature in the box, or switch to a typed signature.".into());
        }
        if !agreed.get() {
            return Some("Tick the box to confirm you intend to sign.".into());
        }
        None
    };
    let blocked = Signal::derive(move || blocker().is_some());

    let sign = move |()| {
        let Some(doc) = document.get_untracked() else {
            return;
        };
        let drawn = drawing.get_untracked();
        let image = drawn
            .then(|| {
                canvas_ref
                    .get_untracked()
                    .and_then(|c| c.to_data_url_with_type("image/png").ok())
            })
            .flatten();
        if drawn && image.is_none() {
            status.set(Status::Failed(
                "Your browser wouldn't export the drawn signature. Switch to a typed signature."
                    .into(),
            ));
            return;
        }

        status.set(Status::Sending);
        spawn_local(async move {
            let Some(token) = auth.token().await else {
                status.set(Status::Failed(
                    "Your session expired. Sign in again and re-sign.".into(),
                ));
                return;
            };
            let signer = SignerIdentity {
                legal_name: legal_name.get_untracked().trim().to_string(),
                email: email.get_untracked().trim().to_string(),
                phone: optional(phone),
                address: PostalAddress {
                    line1: line1.get_untracked().trim().to_string(),
                    line2: optional(line2),
                    city: city.get_untracked().trim().to_string(),
                    region: optional(region),
                    postal_code: postal_code.get_untracked().trim().to_string(),
                    country: country.get_untracked().trim().to_string(),
                },
            };
            let submission = SignatureSubmission {
                document_version: doc.version.clone(),
                typed_name: typed_name.get_untracked().trim().to_string(),
                signer,
                method: if drawn {
                    SignatureMethod::Drawn
                } else {
                    SignatureMethod::Typed
                },
                signature_image: image,
                agreed: true,
            };
            match api::sign_beta_document(&token, &doc.id, &submission).await {
                Ok(receipt) => {
                    status.set(Status::Signed {
                        receipt: receipt.receipt_id,
                        at: receipt.signed_at,
                        complete: receipt.documents_complete,
                    });
                    // Refresh so the "already signed" view is authoritative
                    // rather than inferred from the response.
                    load();
                }
                Err(err) => status.set(Status::Failed(err)),
            }
        });
    };

    view! {
        <AnimatedPage class="relative z-10 min-h-screen px-4 py-10 text-white md:px-8">
            <div class="mx-auto max-w-3xl">
                <button
                    type="button"
                    class="mb-6 inline-flex items-center gap-2 text-sm text-white/50 transition-colors hover:text-white"
                    on:click=move |_| go.run(nav::BETA.to_string())
                >
                    <Icon icon=icons::ARROW_LEFT class="h-4 w-4" />
                    "Back to the beta page"
                </button>

                <Show
                    when=move || auth.is_authenticated.get()
                    fallback=|| {
                        view! {
                            <Alert
                                tone=Tone::Info
                                title="Sign in required"
                                description="These documents are only available to accepted testers."
                            />
                        }
                    }
                >
                    {move || {
                        load_error
                            .get()
                            .map(|message| {
                                view! {
                                    <Alert
                                        tone=Tone::Danger
                                        title="Couldn't load the document"
                                        description=message
                                    />
                                }
                            })
                    }}

                    {move || {
                        let Some(doc) = document.get() else {
                            return load_error
                                .with(|e| {
                                    if e.is_some() {
                                        ().into_any()
                                    } else {
                                        view! {
                                            <Panel class="space-y-3 p-6">
                                                <Skeleton class="h-6 w-64" />
                                                <Skeleton class="h-4 w-full" />
                                                <Skeleton class="h-4 w-5/6" />
                                                <Skeleton class="h-4 w-4/6" />
                                            </Panel>
                                        }
                                            .into_any()
                                    }
                                });
                        };
                        let already_signed = doc.signed;
                        // The status branch runs on every status change, so it needs
                        // its own copy rather than borrowing the one the header uses.
                        let signed_doc = doc.clone();
                        view! {
                            <DocumentHeader doc=doc.clone() />

                            <Panel class="mb-6 overflow-hidden p-0">
                                <div
                                    node_ref=scroller
                                    on:scroll=on_scroll
                                    class="max-h-[60vh] overflow-y-auto px-6 py-6"
                                >
                                    <DocumentBody doc=doc.clone() />
                                </div>
                                <div class=move || {
                                    cn!(
                                        "flex items-center gap-2 border-t px-6 py-3 text-xs transition-colors",
                                        if read_to_end.get() {
                                            "border-emerald-400/20 bg-emerald-500/[0.06] text-emerald-300"
                                        } else {
                                            "border-white/[0.06] bg-white/[0.02] text-white/45"
                                        },
                                    )
                                }>
                                    {move || {
                                        let icon = if read_to_end.get() {
                                            icons::CIRCLE_CHECK
                                        } else {
                                            icons::CHEVRON_DOWN
                                        };
                                        view! { <Icon icon=icon class="h-4 w-4" /> }
                                    }}
                                    {move || {
                                        if read_to_end.get() {
                                            "You've read to the end."
                                        } else {
                                            "Scroll to the end to unlock signing."
                                        }
                                    }}
                                </div>
                            </Panel>

                            {move || match status.get() {
                                Status::Signed { receipt, at, complete } => {
                                    view! { <SignedReceipt receipt=receipt at=at complete=complete go=go /> }
                                        .into_any()
                                }
                                _ if already_signed => {
                                    view! { <AlreadySigned doc=signed_doc.clone() go=go /> }.into_any()
                                }
                                _ => {
                                    view! {
                                        <SignatureBlock
                                            method=method
                                            typed_name=typed_name
                                            legal_name=legal_name
                                            email=email
                                            phone=phone
                                            line1=line1
                                            line2=line2
                                            city=city
                                            region=region
                                            postal_code=postal_code
                                            country=country
                                            agreed=agreed
                                            has_ink=has_ink
                                            canvas_ref=canvas_ref
                                            sending=sending
                                            blocked=blocked
                                            blocker=Signal::derive(blocker)
                                            failure=Signal::derive(move || match status.get() {
                                                Status::Failed(message) => Some(message),
                                                _ => None,
                                            })
                                            on_sign=Callback::new(sign)
                                        />
                                    }
                                        .into_any()
                                }
                            }}
                        }
                            .into_any()
                    }}
                </Show>
            </div>
        </AnimatedPage>
    }
}

#[component]
fn DocumentHeader(doc: BetaDocument) -> impl IntoView {
    view! {
        <div class="mb-6">
            <div class="mb-3 flex flex-wrap items-center gap-2">
                <Badge tone=Tone::Info icon=icons::FILE_TEXT>"Beta paperwork"</Badge>
                <Badge tone=Tone::Neutral size=Size::Xs>{format!("v{}", doc.version)}</Badge>
                {(!doc.effective_date.is_empty())
                    .then(|| {
                        view! {
                            <span class="text-xs text-white/40">
                                {format!("Effective {}", doc.effective_date)}
                            </span>
                        }
                    })}
            </div>
            <h1 class="mb-2 text-3xl font-bold">{doc.title.clone()}</h1>
            <p class="max-w-2xl text-sm leading-relaxed text-white/60">{doc.summary.clone()}</p>
        </div>
    }
}

#[component]
fn DocumentBody(doc: BetaDocument) -> impl IntoView {
    view! {
        <div class="space-y-6">
            {doc
                .sections
                .into_iter()
                .map(|section| {
                    view! {
                        <section>
                            <h2 class="mb-2 text-sm font-semibold tracking-[0.04em] text-white">
                                {section.heading}
                            </h2>
                            <div class="space-y-3">
                                {section
                                    .body
                                    .into_iter()
                                    .map(|paragraph| {
                                        view! {
                                            <p class="text-sm leading-7 text-white/65">
                                                {paragraph}
                                            </p>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </section>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn SignatureBlock(
    method: RwSignal<String>,
    typed_name: RwSignal<String>,
    legal_name: RwSignal<String>,
    email: RwSignal<String>,
    phone: RwSignal<String>,
    line1: RwSignal<String>,
    line2: RwSignal<String>,
    city: RwSignal<String>,
    region: RwSignal<String>,
    postal_code: RwSignal<String>,
    country: RwSignal<String>,
    agreed: RwSignal<bool>,
    has_ink: RwSignal<bool>,
    canvas_ref: NodeRef<leptos::html::Canvas>,
    #[prop(into)] sending: Signal<bool>,
    #[prop(into)] blocked: Signal<bool>,
    #[prop(into)] blocker: Signal<Option<String>>,
    #[prop(into)] failure: Signal<Option<String>>,
    #[prop(into)] on_sign: Callback<()>,
) -> impl IntoView {
    let tabs = Signal::derive(|| {
        vec![
            TabItem::new("draw", "Draw").with_icon(icons::PENCIL),
            TabItem::new("type", "Type").with_icon(icons::KEY),
        ]
    });

    view! {
        <Panel class="p-6">
            <div class="mb-5 flex items-center gap-3">
                <Icon icon=icons::PENCIL class="h-5 w-5 text-accent-400" />
                <h2 class="text-xl font-bold">"Sign this agreement"</h2>
            </div>

            <p class="mb-5 text-sm leading-relaxed text-white/55">
                "This is a binding agreement, so we need to know who is bound by it. Give the details of the person signing — not a nickname, and not a company unless you are signing on its behalf."
            </p>

            <div class="grid gap-4 md:grid-cols-2">
                <Field label="Full legal name" required=true hint="As it appears on your ID.">
                    <TextInput
                        value=legal_name
                        on_input=Callback::new(move |v| legal_name.set(v))
                        placeholder="Jane Quinn Public"
                        disabled=sending
                    />
                </Field>
                <Field label="Email" required=true hint="Where notices under these agreements go.">
                    <TextInput
                        value=email
                        on_input=Callback::new(move |v| email.set(v))
                        placeholder="you@example.com"
                        input_type="email"
                        disabled=sending
                    />
                </Field>
            </div>

            <Field label="Phone (optional)" class="mt-4">
                <TextInput
                    value=phone
                    on_input=Callback::new(move |v| phone.set(v))
                    placeholder="+1 555 0100"
                    disabled=sending
                />
            </Field>

            <div class="mt-6 mb-3 flex items-center gap-2 text-xs font-medium uppercase tracking-[0.14em] text-white/45">
                <Icon icon=icons::MAP class="h-4 w-4 text-accent-400" />
                "Address"
            </div>
            <div class="grid gap-4">
                <Field label="Street address" required=true>
                    <TextInput
                        value=line1
                        on_input=Callback::new(move |v| line1.set(v))
                        placeholder="1 Example Street"
                        disabled=sending
                    />
                </Field>
                <Field label="Apartment, suite, etc. (optional)">
                    <TextInput
                        value=line2
                        on_input=Callback::new(move |v| line2.set(v))
                        placeholder="Apartment 4"
                        disabled=sending
                    />
                </Field>
            </div>
            <div class="mt-4 grid gap-4 md:grid-cols-3">
                <Field label="City or town" required=true>
                    <TextInput
                        value=city
                        on_input=Callback::new(move |v| city.set(v))
                        placeholder="Denver"
                        disabled=sending
                    />
                </Field>
                <Field label="State / region" hint="If your country uses one.">
                    <TextInput
                        value=region
                        on_input=Callback::new(move |v| region.set(v))
                        placeholder="CO"
                        disabled=sending
                    />
                </Field>
                <Field label="Postal / ZIP code" required=true>
                    <TextInput
                        value=postal_code
                        on_input=Callback::new(move |v| postal_code.set(v))
                        placeholder="80202"
                        disabled=sending
                    />
                </Field>
            </div>
            <Field label="Country" required=true class="mt-4">
                <TextInput
                    value=country
                    on_input=Callback::new(move |v| country.set(v))
                    placeholder="United States"
                    disabled=sending
                />
            </Field>

            <Divider class="my-6" />

            <div class="mb-3 flex items-center gap-2 text-xs font-medium uppercase tracking-[0.14em] text-white/45">
                <Icon icon=icons::PENCIL class="h-4 w-4 text-accent-400" />
                "Your signature"
            </div>
            <Field label="Sign your name" required=true hint="Draw it, or type it — both count.">
                <TextInput
                    value=typed_name
                    on_input=Callback::new(move |v| typed_name.set(v))
                    placeholder="Jane Q. Public"
                    disabled=sending
                />
            </Field>

            <div class="mt-5 mb-3">
                <Tabs
                    value=Signal::derive(move || method.get())
                    on_change=Callback::new(move |v| method.set(v))
                    items=tabs
                />
            </div>

            <Show
                when=move || method.get() == "draw"
                fallback=move || view! { <TypedSignature typed_name=typed_name /> }
            >
                <SignaturePad canvas_ref=canvas_ref has_ink=has_ink />
            </Show>

            <Well class="mt-5 p-4">
                <Checkbox
                    checked=agreed
                    on_change=Callback::new(move |v| agreed.set(v))
                    label="I have read this document in full and intend to sign it electronically. My electronic signature is the legal equivalent of my handwritten one."
                />
            </Well>

            {move || {
                failure
                    .get()
                    .map(|message| {
                        view! {
                            <Alert
                                tone=Tone::Danger
                                title="Not signed"
                                description=message
                                class="mt-5"
                            />
                        }
                    })
            }}

            <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div class="text-xs leading-5 text-white/40">
                    {move || {
                        blocker
                            .get()
                            .unwrap_or_else(|| {
                                "Signing records these details, the time, your IP address, and a hash of this exact document."
                                    .to_string()
                            })
                    }}
                </div>
                <Button
                    variant=Variant::Solid
                    icon=icons::CIRCLE_CHECK
                    loading=sending
                    disabled=blocked
                    on_click=on_sign
                    class="shrink-0"
                >
                    "Sign document"
                </Button>
            </div>
        </Panel>
    }
}

/// The typed alternative: a name rendered as a signature, which is what a typed
/// e-signature is in practice. Kept visually parallel to the drawn pad so the
/// two options read as the same act.
#[component]
fn TypedSignature(typed_name: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="flex h-[180px] items-center justify-center rounded-2xl border border-white/10 bg-white px-6">
            <span
                class="truncate text-4xl text-slate-900"
                style="font-family: 'Segoe Script', 'Brush Script MT', 'Snell Roundhand', cursive;"
            >
                {move || {
                    let name = typed_name.get().trim().to_string();
                    if name.is_empty() { "Your name".to_string() } else { name }
                }}
            </span>
        </div>
        <p class="mt-2 text-xs text-white/40">
            "Your typed name is your signature. It carries the same weight as the drawn one."
        </p>
    }
}

/// Canvas signature pad.
///
/// Sized once on mount from the element's own box, scaled by the device pixel
/// ratio so the ink isn't soft on a hi-dpi screen, and painted white so the
/// exported PNG is legible wherever it's later opened rather than being
/// transparent ink on whatever background the viewer happens to have.
#[component]
fn SignaturePad(
    canvas_ref: NodeRef<leptos::html::Canvas>,
    has_ink: RwSignal<bool>,
) -> impl IntoView {
    let drawing = RwSignal::new(false);

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else {
            return;
        };
        // Setting `width` resets the context transform, so re-running this would
        // stack another `scale` on top. The marker makes setup idempotent across
        // effect re-runs while still initialising a freshly mounted canvas.
        if canvas.has_attribute("data-pad-ready") {
            return;
        }
        let rect = canvas.get_bounding_client_rect();
        if rect.width() <= 0.0 {
            return;
        }
        let dpr = window().device_pixel_ratio().max(1.0);
        canvas.set_width((rect.width() * dpr) as u32);
        canvas.set_height((rect.height() * dpr) as u32);

        let Some(ctx) = context(&canvas) else {
            return;
        };
        let _ = ctx.scale(dpr, dpr);
        ctx.set_fill_style_str("#ffffff");
        ctx.fill_rect(0.0, 0.0, rect.width(), rect.height());
        ctx.set_stroke_style_str("#0f172a");
        ctx.set_line_width(2.2);
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        let _ = canvas.set_attribute("data-pad-ready", "1");
        has_ink.set(false);
    });

    let clear = move |()| {
        let Some(canvas) = canvas_ref.get_untracked() else {
            return;
        };
        let Some(ctx) = context(&canvas) else {
            return;
        };
        let rect = canvas.get_bounding_client_rect();
        ctx.set_fill_style_str("#ffffff");
        ctx.fill_rect(0.0, 0.0, rect.width(), rect.height());
        has_ink.set(false);
    };

    view! {
        <div class="relative">
            <canvas
                node_ref=canvas_ref
                class="h-[180px] w-full cursor-crosshair rounded-2xl border border-white/10 bg-white"
                style="touch-action: none;"
                on:pointerdown=move |ev: web_sys::PointerEvent| {
                    ev.prevent_default();
                    let Some(canvas) = canvas_ref.get_untracked() else {
                        return;
                    };
                    let Some(ctx) = context(&canvas) else {
                        return;
                    };
                    let _ = canvas.set_pointer_capture(ev.pointer_id());
                    ctx.begin_path();
                    ctx.move_to(f64::from(ev.offset_x()), f64::from(ev.offset_y()));
                    drawing.set(true);
                }
                on:pointermove=move |ev: web_sys::PointerEvent| {
                    if !drawing.get_untracked() {
                        return;
                    }
                    let Some(canvas) = canvas_ref.get_untracked() else {
                        return;
                    };
                    let Some(ctx) = context(&canvas) else {
                        return;
                    };
                    ctx.line_to(f64::from(ev.offset_x()), f64::from(ev.offset_y()));
                    ctx.stroke();
                    if !has_ink.get_untracked() {
                        has_ink.set(true);
                    }
                }
                on:pointerup=move |_| drawing.set(false)
                on:pointercancel=move |_| drawing.set(false)
                on:pointerleave=move |_| drawing.set(false)
            ></canvas>

            <button
                type="button"
                class="absolute top-3 right-3 rounded-lg border border-slate-300 bg-white/90 px-2.5 py-1 text-xs font-medium text-slate-600 transition-colors hover:bg-white hover:text-slate-900"
                on:click=move |_| clear(())
            >
                "Clear"
            </button>

            <div class="pointer-events-none absolute inset-x-8 bottom-9 border-b border-dashed border-slate-300"></div>
        </div>
        <p class="mt-2 text-xs text-white/40">
            "Draw with a mouse, trackpad, pen or finger. Clear and redo it as many times as you like."
        </p>
    }
}

fn context(canvas: &web_sys::HtmlCanvasElement) -> Option<web_sys::CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()
        .flatten()?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .ok()
}

#[component]
fn SignedReceipt(
    receipt: String,
    at: String,
    complete: bool,
    go: Callback<String>,
) -> impl IntoView {
    view! {
        <Panel class="p-8 text-center">
            <div class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl border border-emerald-400/30 bg-emerald-500/10">
                <Icon icon=icons::CIRCLE_CHECK class="h-6 w-6 text-emerald-300" />
            </div>
            <h2 class="mb-2 text-xl font-bold">"Signed"</h2>
            <p class="mx-auto mb-1 max-w-md text-sm leading-relaxed text-white/60">
                {if complete {
                    "That's both documents done. Watch Discord — your build follows."
                } else {
                    "One down. Head back and sign the other one."
                }}
            </p>
            <p class="mb-6 font-mono text-xs text-white/35">
                {format!("Receipt {receipt} · {at}")}
            </p>
            <Button
                variant=Variant::Solid
                icon=icons::ARROW_LEFT
                on_click=nav::go_to(go, nav::BETA)
            >
                "Back to the beta page"
            </Button>
        </Panel>
    }
}

#[component]
fn AlreadySigned(doc: BetaDocument, go: Callback<String>) -> impl IntoView {
    view! {
        <Panel class="p-6">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center">
                <div class="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-emerald-400/30 bg-emerald-500/10">
                    <Icon icon=icons::CIRCLE_CHECK class="h-5 w-5 text-emerald-300" />
                </div>
                <div class="min-w-0 flex-1">
                    <div class="font-semibold text-white">
                        {format!("You signed this (v{})", doc.version)}
                    </div>
                    <div class="text-sm text-white/55">
                        {doc
                            .signed_at
                            .clone()
                            .map(|at| format!("Signed {at}"))
                            .unwrap_or_else(|| "On file with us.".to_string())}
                    </div>
                </div>
                <Button
                    variant=Variant::Glass
                    icon=icons::ARROW_LEFT
                    on_click=nav::go_to(go, nav::BETA)
                    class="shrink-0"
                >
                    "Back"
                </Button>
            </div>
        </Panel>
    }
}
