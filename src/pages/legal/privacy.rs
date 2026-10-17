//! Privacy notice covering the processors this site actually talks to: Auth0
//! (identity), the support API, and the closed-beta API. Every claim here
//! should be traceable to code — the Auth0 scopes in auth0.js, the support
//! ticket in api.rs, and the beta application form. If a new endpoint starts
//! receiving personal data, it belongs in section 2.

use infinity_ui::prelude::*;

use super::{External, Internal, LegalPage, Section};
use crate::brand::PUBLISHER;
use crate::nav;

const PRIVACY_VERSION: &str = "1.0";
const PRIVACY_EFFECTIVE: &str = "20 September 2026";

#[component]
pub fn PrivacyPolicy() -> impl IntoView {
    view! {
        <LegalPage
            title="Privacy Policy"
            version=format!("Version {PRIVACY_VERSION}")
            effective=PRIVACY_EFFECTIVE
            intro=ViewFn::from(|| {
                view! {
                    <Lead>
                        {format!(
                            "This explains what {PUBLISHER} collects when you use the Infinity website and the Infinity Manager, why we collect it, who else sees it, and what you can ask us to do about it.",
                        )}
                    </Lead>
                    <Callout title="The short version">
                        <List>
                            <ListItem>
                                "We collect your email and name so you can sign in and so we know which aircraft you own. That's the core of it."
                            </ListItem>
                            <ListItem>
                                <strong>"We never see your card details."</strong>
                                " Payments go to Lemon Squeezy, our merchant of record."
                            </ListItem>
                            <ListItem>
                                "We don't sell your data and we don't run ad trackers."
                            </ListItem>
                            <ListItem>
                                "You can ask us for a copy of your data, or to delete it — see section 6."
                            </ListItem>
                        </List>
                    </Callout>
                }
            })
        >
            <Section n="1" anchor="who" title="Who is responsible">
                <P>
                    {format!(
                        "{PUBLISHER}, a limited liability company formed in the State of Iowa, is the controller of the personal data described here. Contact us through the ",
                    )}
                    <Internal href=nav::USER>"support form"</Internal>
                    " on your dashboard or on "
                    <External href=nav::DISCORD_URL>"Discord"</External>
                    "."
                </P>
            </Section>

            <Section n="2" anchor="what" title="What we collect, and who processes it">
                <P>
                    <strong>"Your account (Auth0)."</strong>
                    " Signing in is handled by Auth0. We request the "
                    <code>"openid profile email"</code>
                    " scopes, so Auth0 gives us your email address, your display name and your profile picture, plus an account identifier. Auth0 also stores which aircraft you own, as permissions on your account, and holds your password if you signed up with one rather than with Google, Microsoft, Discord, GitHub, Facebook, Apple or X. Your session is kept in your browser's local storage so you stay signed in."
                </P>
                <P>
                    <strong>"Your purchases (Lemon Squeezy)."</strong>
                    " Lemon Squeezy is the merchant of record and takes the payment. We pass it your email address and name to pre-fill the checkout, and it tells us which order belongs to which account so we can grant the aircraft. "
                    <strong>
                        "We never receive or store your card number, and we do not have access to your payment method."
                    </strong>
                    " Lemon Squeezy holds your billing address and tax location; we don't."
                </P>
                <P>
                    <strong>"Support requests."</strong>
                    " When you use the support form on your dashboard, your name, email address, Discord handle and the message you write are sent to our support service, hosted on Fly.io, so we can reply and keep a record of the conversation."
                </P>
                <P>
                    <strong>"Closed beta applications."</strong>
                    " If you apply for the closed beta, the form collects what you type into it: display name, email, Discord handle, timezone, your experience and how you expect to use the product, your self-reported PC specifications (CPU, GPU, RAM, storage, monitors), your availability and testing preferences, and any notes you add. This goes to our beta service, also hosted on Fly.io."
                </P>
                <P>
                    <strong>"The Infinity Manager."</strong>
                    " The Manager signs in with the same account and asks our servers which aircraft you own and which versions are current, so it can install and update them."
                </P>
                <Callout tone=Tone::Warning title="What we don't do">
                    <p>
                        "We don't run advertising trackers, we don't sell or rent personal data to anyone, and we don't use your data to build a profile for advertising. We don't ask for your date of birth, your address, or any special-category data."
                    </p>
                </Callout>
            </Section>

            <Section n="3" anchor="why" title="Why we use it">
                <List>
                    <ListItem>
                        <strong>"To give you what you bought"</strong>
                        " — signing you in, recording which aircraft you own, and letting the Manager install and update them. Without this the product does not work."
                    </ListItem>
                    <ListItem>
                        <strong>"To take payment and handle refunds"</strong>
                        ", including keeping the order record we need to apply the "
                        <Internal href=nav::LEGAL_REFUNDS>"Refund Policy"</Internal>
                        "."
                    </ListItem>
                    <ListItem>
                        <strong>"To answer you"</strong>
                        " when you contact support, and to consider a beta application if you send one."
                    </ListItem>
                    <ListItem>
                        <strong>"To keep the service working and secure"</strong>
                        " — for example rate-limiting checkout and password attempts to prevent abuse."
                    </ListItem>
                    <ListItem>
                        <strong>"To meet our legal obligations"</strong>
                        ", including tax and accounting records."
                    </ListItem>
                </List>
                <P>
                    "If you are in the UK or EU: we rely on performance of our contract with you for the first two, our legitimate interests in running and securing the service for the next two, your consent for a beta application, and our legal obligations for the last."
                </P>
            </Section>

            <Section n="4" anchor="sharing" title="Who else sees your data">
                <P>
                    "Only the service providers we need to run the product. Each one is bound to use the data only to provide its service to us:"
                </P>
                <List>
                    <ListItem>
                        <strong>"Auth0"</strong>
                        " (Okta) — sign-in, accounts and entitlements."
                    </ListItem>
                    <ListItem>
                        <strong>"Lemon Squeezy"</strong>
                        " — merchant of record, payments, tax and receipts."
                    </ListItem>
                    <ListItem>
                        <strong>"Cloudflare"</strong>
                        " — hosting for this website and our API worker."
                    </ListItem>
                    <ListItem>
                        <strong>"Fly.io"</strong>
                        " — hosting for the support and closed-beta services."
                    </ListItem>
                    <ListItem>
                        <strong>"Discord"</strong>
                        " — if you choose to contact us there, your messages are subject to Discord's own privacy policy."
                    </ListItem>
                </List>
                <P>
                    "We may also disclose data where the law requires it, or to establish or defend legal claims — for example, responding to a payment dispute."
                </P>
                <P>
                    "These providers operate internationally, so your data may be processed outside your country, including in the United States. Where UK or EU law requires it, those transfers rely on the providers' standard contractual clauses or an equivalent safeguard."
                </P>
            </Section>

            <Section n="5" anchor="retention" title="How long we keep it">
                <List>
                    <ListItem>
                        <strong>"Your account"</strong>
                        " — for as long as you have one. Delete your account and we remove it, subject to the records below."
                    </ListItem>
                    <ListItem>
                        <strong>"Order and refund records"</strong>
                        " — for as long as tax and accounting law requires, typically several years, and after a refund. We can't delete these on request."
                    </ListItem>
                    <ListItem>
                        <strong>"Support conversations"</strong>
                        " — while they're useful for helping you, and then deleted."
                    </ListItem>
                    <ListItem>
                        <strong>"Beta applications"</strong>
                        " — until the beta programme ends, or until you ask us to remove yours."
                    </ListItem>
                </List>
            </Section>

            <Section n="6" anchor="rights" title="Your rights">
                <P>
                    "Wherever you live, you can ask us to show you what we hold, correct it, or delete it, and we will do our best to help. Depending on where you live you may have these as enforceable rights — under the UK and EU GDPR, the California Consumer Privacy Act, or similar laws elsewhere:"
                </P>
                <List>
                    <ListItem>
                        <strong>"Access"</strong>
                        " — a copy of the personal data we hold about you."
                    </ListItem>
                    <ListItem>
                        <strong>"Correction"</strong>
                        " — fix anything inaccurate. You can change your own name, email and password from your "
                        <Internal href=nav::USER>"dashboard"</Internal>
                        "."
                    </ListItem>
                    <ListItem>
                        <strong>"Deletion"</strong>
                        " — remove your account and data, except records we are legally required to keep."
                    </ListItem>
                    <ListItem>
                        <strong>"Portability"</strong>
                        " — your data in a machine-readable form."
                    </ListItem>
                    <ListItem>
                        <strong>"Objection and restriction"</strong>
                        " — object to processing we base on legitimate interests, or ask us to pause it."
                    </ListItem>
                    <ListItem>
                        <strong>"Withdraw consent"</strong>
                        " — where we asked for it, at any time, without affecting what we did before."
                    </ListItem>
                </List>
                <P>
                    <strong>
                        "We will never charge you, deny you service, or give you a worse price for exercising any of these."
                    </strong>
                    " To make a request, use the support form or Discord. We'll respond within one month. If you're in the UK or EU and you think we've got it wrong, you can complain to your national data protection authority — in the UK, the Information Commissioner's Office."
                </P>
            </Section>

            <Section n="7" anchor="storage" title="Cookies and browser storage">
                <P>
                    "We don't use advertising or analytics cookies. Your sign-in session is kept in your browser's local storage because the site needs it to know you're signed in; clearing your browser data signs you out. Auth0 and Cloudflare may set cookies that are strictly necessary for sign-in and for protecting the service against abuse."
                </P>
            </Section>

            <Section n="8" anchor="children" title="Children">
                <P>
                    "The Infinity store is not directed at children under 13, and we don't knowingly collect their personal data. If you believe a child has given us data, contact us and we'll delete it."
                </P>
            </Section>

            <Section n="9" anchor="changes" title="Changes to this policy">
                <P>
                    "We may update this policy; the version number and effective date at the top will change. If a change materially affects how we use data you have already given us, we'll make that clear rather than relying on the date alone."
                </P>
            </Section>
        </LegalPage>
    }
}
