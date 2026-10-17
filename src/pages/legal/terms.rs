//! Terms of Service — the account-and-platform layer. Deliberately does NOT
//! restate the licence (that is the EULA) or the refund terms (that is the
//! Refund Policy); it covers the ground neither reaches: accounts, acceptable
//! use, the Manager, freeware, community conduct and takedowns. Section 3 sets
//! the precedence order so the three documents can't be played against each
//! other.

use infinity_ui::prelude::*;

use super::{External, Internal, LegalPage, Section};
use crate::brand::PUBLISHER;
use crate::nav;

const TERMS_VERSION: &str = "1.0";
const TERMS_EFFECTIVE: &str = "20 September 2026";

#[component]
pub fn Terms() -> impl IntoView {
    view! {
        <LegalPage
            title="Terms of Service"
            version=format!("Version {TERMS_VERSION}")
            effective=TERMS_EFFECTIVE
            intro=ViewFn::from(|| {
                view! {
                    <Lead>
                        {format!(
                            "These terms cover your Infinity account, this website, and the Infinity Manager. They are the agreement between you and {PUBLISHER}.",
                        )}
                    </Lead>
                    <P>
                        "Two things they deliberately don't cover, because each has its own document: what you may do with an aircraft you've bought is the "
                        <strong>"End-User License Agreement"</strong>
                        ", linked from every aircraft page, and refunds are the "
                        <Internal href=nav::LEGAL_REFUNDS>"Refund Policy"</Internal>
                        "."
                    </P>
                }
            })
        >
            <Section n="1" anchor="acceptance" title="Accepting these terms">
                <P>
                    "By creating an account, using this website, or installing the Infinity Manager, you agree to these terms. If you don't agree, don't use them."
                </P>
                <P>
                    "You need to be old enough to enter a contract where you live to buy anything here. If you're under 18, you need a parent or guardian's permission."
                </P>
            </Section>

            <Section n="2" anchor="account" title="Your account">
                <P>
                    "You need an account to buy aircraft and to use the Manager. Sign-in is handled by Auth0 — you can use a password or a linked Google, Microsoft, Discord, GitHub, Facebook, Apple or X account."
                </P>
                <List>
                    <ListItem>
                        "Keep your sign-in details to yourself. You're responsible for what happens on your account, unless someone gets in through no fault of yours — tell us as soon as you notice."
                    </ListItem>
                    <ListItem>
                        <strong>"One account per person."</strong>
                        " Your aircraft are tied to your account and are not transferable — don't sell, share or trade the account."
                    </ListItem>
                    <ListItem>
                        "Give us accurate details. If your email address stops working we may not be able to reach you about a purchase or a refund."
                    </ListItem>
                    <ListItem>
                        "You can close your account at any time by asking us. Closing it means the Manager can no longer install or update the aircraft on it."
                    </ListItem>
                </List>
            </Section>

            <Section n="3" anchor="documents" title="Which document wins">
                <P>
                    "If these terms, the EULA and the Refund Policy ever disagree: "
                    <strong>"the Refund Policy governs anything about refunds"</strong>
                    ", the EULA governs what you may do with an aircraft, and these terms govern everything else. Nothing in any of the three takes away rights your local consumer law gives you."
                </P>
            </Section>

            <Section n="4" anchor="buying" title="Buying aircraft">
                <P>
                    "Prices are shown on each aircraft's page and include any applicable tax at checkout. Payment is taken by Lemon Squeezy, our merchant of record — your statement will show a charge beginning "
                    <code>"LEMSQZY*"</code>
                    "."
                </P>
                <P>
                    "A purchase gives you a licence to use that aircraft, not ownership of it. Once your payment clears, the aircraft appears on your account and the Manager can install it. This can take a few minutes."
                </P>
                <P>
                    "We may change prices, run sales, or stop selling an aircraft at any time. That never affects an aircraft you've already bought. Refunds are covered by the "
                    <Internal href=nav::LEGAL_REFUNDS>"Refund Policy"</Internal>
                    "."
                </P>
                <P>
                    "If an aircraft is listed at an obviously wrong price because of a mistake, we may cancel the order and refund you in full rather than honour it. We'll tell you if that happens."
                </P>
            </Section>

            <Section n="5" anchor="manager" title="The Infinity Manager">
                <P>
                    "The Manager is free and is required to install, update and run Infinity aircraft. It updates itself automatically and needs an internet connection to sign in and to fetch updates."
                </P>
                <P>
                    "We aim to keep it working, but we don't promise it will always be available or bug-free. We may change how it works, and we may stop supporting older versions or operating systems. If we ever wind the service down, we'll publish a way for existing owners to keep and reinstall the aircraft they have bought."
                </P>
            </Section>

            <Section n="6" anchor="freeware" title="Community freeware">
                <P>
                    "The Manager also distributes free add-ons made by other people. Those are their creators' work, not ours: they come under their own licence terms, and the creator is responsible for them."
                </P>
                <P>
                    "We don't charge for freeware and we don't promise it works, is safe for your system, or will keep being available. We may remove any freeware from the Manager at any time, including on a rights holder's request."
                </P>
            </Section>

            <Section n="7" anchor="acceptable-use" title="Acceptable use">
                <P>"Don't do any of this with our website, our services or the Manager:"</P>
                <List>
                    <ListItem>
                        "Break into, probe, overload or disrupt them, or try to get at data or accounts that aren't yours."
                    </ListItem>
                    <ListItem>
                        "Work around sign-in, entitlement checks or rate limits, or use an automated tool to buy, scrape or download at scale."
                    </ListItem>
                    <ListItem>
                        "Upload or send anything unlawful, malicious, or infringing someone else's rights."
                    </ListItem>
                    <ListItem>
                        "Resell, redistribute or repackage anything we distribute, except where a freeware creator's own licence allows it."
                    </ListItem>
                    <ListItem>
                        "Impersonate us or anyone else, or misrepresent your connection to us."
                    </ListItem>
                </List>
                <Callout title="Security reports are welcome">
                    <p>
                        "If you find a security problem, tell us on "
                        <External href=nav::DISCORD_URL>"Discord"</External>
                        " before telling anyone else, and give us reasonable time to fix it. Good-faith research reported that way is not a breach of this section."
                    </p>
                </Callout>
            </Section>

            <Section n="8" anchor="community" title="Discord and community spaces">
                <P>
                    "Our Discord server has its own rules, and Discord's own terms apply there too. Being removed from the Discord does not affect the aircraft you own, and losing access to an aircraft is not a moderation tool we use for community conduct."
                </P>
            </Section>

            <Section n="9" anchor="ip" title="Our content, and takedowns">
                <P>
                    {format!(
                        "The Infinity name and logo, this website, the Manager and our aircraft are owned by {PUBLISHER} or our licensors. Microsoft Flight Simulator is a trademark of Microsoft Corporation; we are not affiliated with, endorsed by, or sponsored by Microsoft Corporation. Aircraft names and markings may be trademarks of their manufacturers or operators, used to identify the aircraft depicted.",
                    )}
                </P>
                <P>
                    "If you believe something we distribute infringes your copyright or trademark, contact us on "
                    <External href=nav::DISCORD_URL>"Discord"</External>
                    " or through the "
                    <Internal href=nav::USER>"support form"</Internal>
                    " with enough detail to identify the work, where it appears, and how to reach you. We'll investigate and remove infringing material."
                </P>
            </Section>

            <Section n="10" anchor="suspension" title="Suspension and termination">
                <P>
                    "We may suspend or close an account that breaches these terms or the EULA, or where we reasonably suspect fraud. Where it's proportionate we'll warn you first and give you a chance to put it right, and we'll tell you the reason."
                </P>
                <P>
                    "If we close your account for a breach, aircraft you've already bought are not refunded — see section 13 of the "
                    <Internal href=nav::LEGAL_REFUNDS>"Refund Policy"</Internal>
                    ". If we close it for any other reason, that section still applies."
                </P>
            </Section>

            <Section n="11" anchor="warranty" title="What we do and don't promise">
                <P>
                    "We'll provide the website, the Manager and our aircraft with reasonable care and skill. Beyond that, and beyond what the law requires, they are provided as they are: we don't promise they will be uninterrupted, error-free, or compatible with every system or every version of Microsoft Flight Simulator."
                </P>
                <P>
                    <strong>"This does not affect the rights your consumer law gives you."</strong>
                    " If you're a consumer in the UK, EU/EEA, Australia or anywhere with similar protections, you have rights that can't be excluded — including that what you buy must match its description and be of satisfactory quality — and nothing here limits them."
                </P>
            </Section>

            <Section n="12" anchor="liability" title="Liability">
                <P>
                    "We're liable to you for loss we cause that is a foreseeable result of breaking these terms. We are not liable for loss that isn't foreseeable, for business losses, or for damage caused by something outside our reasonable control."
                </P>
                <P>
                    "Where liability can be limited but not excluded, our total liability to you is limited to what you have paid us in the 12 months before the claim."
                </P>
                <P>
                    <strong>
                        "Nothing here excludes or limits liability for death or personal injury caused by negligence, for fraud, or for anything else that can't lawfully be excluded."
                    </strong>
                </P>
            </Section>

            <Section n="13" anchor="changes" title="Changes to these terms">
                <P>
                    "We may update these terms; the version number and effective date at the top will change. If a change materially affects you, we'll give reasonable notice. Continuing to use your account after a change means you accept it — if you don't, you can close your account, and aircraft you've already bought stay yours under the EULA."
                </P>
            </Section>

            <Section n="14" anchor="law" title="Governing law">
                <P>
                    "These terms are governed by the laws of the State of Iowa, United States of America, without regard to its conflict-of-laws rules."
                </P>
                <P>
                    "If you are a consumer, this does not deprive you of the protection of the mandatory consumer-protection laws of the country where you habitually reside, and you may bring proceedings in the courts of that country where its law gives you that right."
                </P>
                <P>
                    "If any part of these terms turns out to be unenforceable, the rest continues to apply."
                </P>
            </Section>

            <Section n="15" anchor="contact" title="Contact">
                <P>
                    {format!(
                        "{PUBLISHER}, a limited liability company formed in the State of Iowa. Reach us through the ",
                    )}
                    <Internal href=nav::USER>"support form"</Internal>
                    " on your dashboard or on "
                    <External href=nav::DISCORD_URL>"Discord"</External>
                    "."
                </P>
            </Section>
        </LegalPage>
    }
}
