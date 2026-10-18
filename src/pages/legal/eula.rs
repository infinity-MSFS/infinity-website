//! The EULA, shared by the product-page modal and the standalone
//! /legal/eula/:slug page so both render exactly one source of text.
//!
//! The numbered sections are hardcoded here; the company name, product
//! reference, disclaimer and any extra sections come from the remote catalog
//! (Worker KV `website-catalog`). Nothing legal should live only in KV — a
//! mutable store with no version history can't evidence what a buyer accepted.

use infinity_ui::prelude::*;
use leptos_router::hooks::use_params_map;

use super::{Internal, LINK, LegalPage, P, PROSE};
use crate::api::AircraftProduct;
use crate::auth::use_auth;
use crate::brand::{COPYRIGHT_YEAR, PUBLISHER};
use crate::components::EYEBROW;
use crate::nav;
use crate::state::use_catalog;

const HEADING: &str = "mb-2 font-mono text-[11px] tracking-[0.2em] text-accent-400/80 uppercase";

/// Each clause sits under a hairline rather than in a box, so the agreement
/// reads the same in the product page's modal as on its own page.
const CLAUSE: &str = "border-t border-white/10 pt-5";

/// Our own products get a generated notice, so the year lives in the repo and
/// moves in code review rather than in a KV edit someone has to remember. A
/// third-party or freeware rights holder keeps their own wording verbatim —
/// their rights statement is theirs to set, not ours.
fn copyright_line(product: &AircraftProduct) -> String {
    let holder = product.eula_company.as_deref().unwrap_or(PUBLISHER);
    match &product.eula_copyright {
        Some(notice) if holder != PUBLISHER => notice.clone(),
        _ => format!("Copyright © {COPYRIGHT_YEAR} {holder}. All rights reserved."),
    }
}

/// The complete EULA body — preamble, optional per-product disclaimer, the
/// standard sections, any catalog-supplied extra sections, and the footer. The
/// modal and the standalone page both render this and supply their own chrome.
#[component]
pub fn EulaDocument(product: AircraftProduct) -> impl IntoView {
    let company = product
        .eula_company
        .clone()
        .unwrap_or_else(|| PUBLISHER.to_string());
    let product_ref = product
        .eula_product_ref
        .clone()
        .unwrap_or_else(|| product.name.clone());

    view! {
        <div class=cn!("space-y-6", PROSE)>
            <div class="border-l border-amber-300/50 py-1 pl-5">
                <p class="mb-3 font-mono text-[11px] tracking-[0.2em] text-amber-300/90">
                    "IMPORTANT - READ CAREFULLY:"
                </p>
                <p class="text-white/80">
                    {format!(
                        "This End-User License Agreement (\"EULA\") is a legal agreement between you (the \"User\" or \"you\") and {company} (the \"Author\") for the {product_ref} (\"Add-on\") for Microsoft Flight Simulator.",
                    )}
                </p>
                <p class="mt-3 text-white/80">
                    "By downloading, installing, or using the Add-on, you agree to be bound by the terms of this EULA. If you do not agree, do not install or use the Add-on and delete all copies immediately."
                </p>
            </div>

            {product
                .eula_disclaimer
                .clone()
                .map(|text| {
                    view! {
                        <div class="border-l border-red-400/50 py-1 pl-5">
                            <p class="mb-3 font-mono text-[11px] tracking-[0.2em] text-red-300">
                                "DISCLAIMER:"
                            </p>
                            <p class="text-white/80">{text}</p>
                        </div>
                    }
                })}

            <StandardSections company=company />

            {product
                .eula_extra_sections
                .clone()
                .into_iter()
                .map(|section| {
                    view! {
                        <div class=CLAUSE>
                            <h3 class=HEADING>{section.heading}</h3>
                            {section
                                .body
                                .into_iter()
                                .map(|para| view! { <p class="mb-2">{para}</p> })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}

            <div class="border-t border-white/10 pt-5">
                <p class="font-mono text-[11px] text-white/40">{copyright_line(&product)}</p>
                <p class="mt-1 font-mono text-[11px] text-white/40">
                    "Microsoft Flight Simulator is a trademark of Microsoft Corporation."
                </p>
            </div>
        </div>
    }
}

#[component]
fn StandardSections(company: String) -> impl IntoView {
    view! {
        <div class=CLAUSE>
            <h3 class=HEADING>"1. GRANT OF LICENSE"</h3>
            <p>
                "The Author grants you a limited, non-exclusive, non-transferable, royalty-free license to install and use the Add-on solely for personal, non-commercial entertainment purposes within Microsoft Flight Simulator."
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"2. PERMITTED USE"</h3>
            <p>
                "You may install and use the Add-on on personal computers you own or control for private, non-commercial use only. You may make a reasonable number of copies of the Add-on for back-up or archival purposes, for your own use."
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"3. RESTRICTIONS"</h3>
            <p class="mb-2">"You may NOT:"</p>
            <ul class="list-disc space-y-2 pl-5 marker:text-accent-400/60">
                <li>
                    "Redistribute, sell, rent, lease, sublicense, or otherwise transfer the Add-on or any part thereof, except where applicable law gives you a right to do so that cannot be excluded by agreement."
                </li>
                <li>
                    "Modify, adapt, translate, reverse engineer, decompile, disassemble, or create derivative works based on the Add-on, except to the extent such acts are permitted by applicable law notwithstanding this limitation."
                </li>
                <li>
                    "Use the Add-on for any commercial purpose, including but not limited to flight training, professional simulation, or public exhibition where admission is charged."
                </li>
                <li>
                    "Remove or alter any copyright, trademark, or proprietary notices in the Add-on files."
                </li>
            </ul>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"4. OWNERSHIP"</h3>
            <p>
                {format!(
                    "The Add-on is licensed, not sold. {company} retains all right, title, and interest in and to the Add-on, including all intellectual property rights.",
                )}
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"5. NO WARRANTY"</h3>
            <p>
                "THE ADD-ON IS PROVIDED \"AS IS\" WITHOUT WARRANTY OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, OR NON-INFRINGEMENT."
            </p>
            <p class="mt-2">
                "Some jurisdictions do not allow the exclusion of implied warranties or of statutory guarantees for digital content. Where that is the case, this section applies only to the extent permitted by law, and your rights under Section 9 are unaffected."
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"6. LIMITATION OF LIABILITY"</h3>
            <p>
                "IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR EXEMPLARY DAMAGES ARISING FROM THE USE OR INABILITY TO USE THE ADD-ON."
            </p>
            <p class="mt-2">
                "Nothing in this section excludes or limits liability for death or personal injury caused by negligence, for fraud or fraudulent misrepresentation, or for any other liability that cannot lawfully be excluded or limited. Where liability may be limited but not excluded, it is limited to the amount you paid for the Add-on."
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"7. TERMINATION"</h3>
            <p>
                "This license terminates automatically if you violate any term. Upon termination, you must destroy all copies of the Add-on."
            </p>
            <p class="mt-2">
                "This license also ends if your purchase of the Add-on is refunded, or if a payment dispute is resolved in your favour and the money is returned to you. In that case your entitlement is removed from your account and the Infinity Manager will no longer download, reinstall or update the Add-on. You are asked to remove the Add-on from your simulator; nothing in this section requires you to provide proof that you have done so, and no refund is conditional on it."
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"8. REFUNDS"</h3>
            <p>
                "Your purchase of the Add-on is a separate transaction from this license. It is made through our authorised reseller and merchant of record, currently Lemon Squeezy. Refunds, and the time limits that apply to them, are set out in full in the "
                <Internal href=nav::LEGAL_REFUNDS>"Infinity Refund Policy"</Internal>
                ". That document is authoritative on refunds; nothing in this EULA limits it, and where the two differ the Refund Policy governs. Nothing in this EULA affects any refund, cancellation, withdrawal or conformity right you have under the consumer law of the country where you live."
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"9. CONSUMER RIGHTS"</h3>
            <p>
                "If you are a consumer, you have rights under the law of the country where you live that cannot be excluded or limited by agreement — including under the Consumer Rights Act 2015 in the United Kingdom, Directives 2011/83/EU and (EU) 2019/770 in the European Union and EEA, and the consumer guarantees under the Australian Consumer Law. Nothing in this EULA, including Sections 5, 6, 7 and 8, excludes, restricts or modifies those rights, and no term of this EULA applies to the extent that it would. Where those rights conflict with a term of this EULA, those rights prevail."
            </p>
        </div>
        <div class=CLAUSE>
            <h3 class=HEADING>"10. GOVERNING LAW"</h3>
            <p>
                "This EULA is governed by the laws of the State of Iowa, United States of America, without regard to its conflict-of-laws rules. If you are a consumer, this does not deprive you of the protection of the mandatory consumer-protection laws of the country where you habitually reside, and you may bring proceedings in the courts of that country where its law gives you that right."
            </p>
        </div>
    }
}

/// Standalone, linkable EULA at /legal/eula/:slug. Resolving the product
/// mirrors `AircraftDetail`, including hiding non-storefront products from
/// everyone but developers.
#[component]
pub fn EulaPage() -> impl IntoView {
    let go = nav::use_go();
    let params = use_params_map();
    let catalog = use_catalog();
    let loading = catalog.0.loading();
    let error = catalog.0.error;
    let slug = Signal::derive(move || params.read().get("slug").unwrap_or_default());
    let staff = use_auth();
    let product = Signal::derive(move || {
        let developer = staff.entitlements.get().is_developer();
        catalog
            .by_slug(&slug.get())
            .filter(|p| developer || p.storefront)
    });

    view! {
        {move || {
            if loading.get() {
                return view! {
                    <div class=cn!("flex min-h-screen items-center justify-center", EYEBROW)>
                        "Loading…"
                    </div>
                }
                    .into_any();
            }
            if let Some(message) = error.get() {
                return view! {
                    <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
                        <p class=cn!("mb-6", EYEBROW)>"Infinity / Legal"</p>
                        <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                            "Catalog "
                            <span class="font-semibold text-accent-300">"unavailable."</span>
                        </h1>
                        <p class="max-w-md border-l border-red-400/60 pl-5 font-light text-white/60">
                            {message}
                        </p>
                    </div>
                }
                    .into_any();
            }
            match product.get() {
                Some(product) => {
                    let href = format!("{}/{}", nav::AIRCRAFT, product.slug);
                    let name = product.name.clone();
                    view! {
                        <LegalPage
                            title="End-User License Agreement"
                            intro=ViewFn::from(move || {
                                let (href, name) = (href.clone(), name.clone());
                                view! {
                                    <P>
                                        "For "
                                        <a href=href class=LINK>
                                            {name}
                                        </a>
                                        ". Refund terms are in the "
                                        <Internal href=nav::LEGAL_REFUNDS>"Refund Policy"</Internal>
                                        ", which governs over this agreement on refunds."
                                    </P>
                                }
                            })
                        >
                            <EulaDocument product=product />
                        </LegalPage>
                    }
                        .into_any()
                }
                None => {
                    view! {
                        <div class="mx-auto flex min-h-screen max-w-6xl flex-col justify-center px-6 text-white md:px-10">
                            <p class=cn!("mb-6", EYEBROW)>"Infinity / Legal"</p>
                            <h1 class="mb-4 text-4xl font-extralight tracking-tight md:text-5xl">
                                "Agreement "
                                <span class="font-semibold text-accent-300">"not found."</span>
                            </h1>
                            <p class="mb-8 max-w-md border-l border-accent-400/40 pl-5 font-light text-white/60">
                                "This product doesn't exist or isn't available."
                            </p>
                            <button
                                type="button"
                                on:click=move |_| go.run(nav::AIRCRAFT.to_string())
                                class="group inline-flex items-center gap-2 self-start text-lg font-medium transition-colors hover:text-accent-300"
                            >
                                <Icon
                                    icon=icons::ARROW_LEFT
                                    class="h-4 w-4 transition-transform group-hover:-translate-x-1"
                                />
                                "Back to Aircraft"
                            </button>
                        </div>
                    }
                        .into_any()
                }
            }
        }}
    }
}
