//! The canonical refund policy. The EULA (§8) and the product page both point
//! here rather than restating any of it — one source of truth, so the two
//! can't drift.
//!
//! Two drafting constraints run through the whole document and should survive
//! any edit:
//!
//!  1. Lemon Squeezy is the merchant of record and holds the money. Every
//!     promise is "we will approve and instruct a refund", never "we will
//!     refund" — promising an outcome we don't control is a misrepresentation
//!     the first time LS declines.
//!  2. The change-of-mind refund (§3) and the fault remedy (§4) are kept
//!     structurally apart. A time limit or an install condition on §4 is void
//!     under UK CRA 2015 s.47, EU DCD Art. 22 and the Australian Consumer Law,
//!     and merging the two paragraphs is what makes a policy unlawful rather
//!     than merely stingy. §4 must never inherit §3's install condition.
//!
//! §3 gates on a STATE ("have you installed it?"), not a clock. An earlier
//! draft ran a 3-day timer from first download, which needed a per-user
//! first-download timestamp the worker does not record — so the limb never
//! bit, every buyer effectively got 14 unconditional days, and the policy said
//! so out loud. A binary condition needs no telemetry and closes the
//! buy-install-fly-refund loop.
//!
//! The words "activation" and "download" are both deliberately avoided for the
//! cut-off: the manager page tells the public that aircraft are "activated
//! automatically", and the purchase page frames the user's deliberate act as
//! "install your new aircraft" from the Library tab — so "installed" is the
//! only term that matches what the buyer actually chose to do.

use infinity_ui::prelude::*;

use super::{External, Internal, LegalPage, Section};
use crate::brand::PUBLISHER;
use crate::nav;

/// Bump whenever the substance changes. §14 promises buyers we apply whichever
/// version is more favourable to them, so this is what identifies a version.
const REFUND_POLICY_VERSION: &str = "1.0";

#[component]
pub fn RefundPolicy() -> impl IntoView {
    view! {
        <LegalPage
            title="Refund Policy"
            version=format!("Version {REFUND_POLICY_VERSION}")
            intro=ViewFn::from(|| {
                view! {
                    <Lead>
                        "This policy covers paid Infinity aircraft bought on our website. It sits alongside the End-User License Agreement: the EULA covers your licence to use an aircraft, this covers the purchase. Where refunds are concerned, this document is the one that counts."
                    </Lead>
                    <P>
                        "The terms below are in line with standard practice for downloadable simulator add-ons — most of this market offers no change-of-mind refund at all."
                    </P>
                    <Callout title="The short version">
                        <List>
                            <ListItem>
                                <strong>"Full refund within 14 days of buying"</strong>
                                ", as long as you "
                                <strong>"haven't installed"</strong>
                                " the aircraft yet. Already installed it? Section 4 still covers anything faulty or not as described, with no deadline."
                            </ListItem>
                            <ListItem>
                                <strong>"Faulty, broken, or not as we described it?"</strong>
                                " No deadline. Tell us and we'll fix it or refund you."
                            </ListItem>
                            <ListItem>
                                <strong>
                                    "Bought it twice, bought the wrong one, or charged twice?"
                                </strong>
                                " Refunded whenever you tell us."
                            </ListItem>
                            <ListItem>
                                <strong>"A refund removes the aircraft from your account."</strong>
                            </ListItem>
                            <ListItem>
                                <strong>
                                    "Nothing here takes away rights your local consumer law gives you."
                                </strong>
                            </ListItem>
                        </List>
                    </Callout>
                }
            })
        >
            <Section n="1" anchor="seller" title="Who you are buying from">
                <P>{format!("Infinity aircraft are developed and published by {PUBLISHER}.")}</P>
                <P>
                    "Your payment is taken by our authorised reseller and "
                    <strong>"merchant of record"</strong>
                    ", currently Lemon Squeezy. On the payment side Lemon Squeezy is the seller: it issues your receipt, collects any sales tax or VAT that applies where you live, and is the party that actually moves the money when a refund is made."
                </P>
                <P>
                    <strong>
                        "Your statement will show a charge beginning " <code>"LEMSQZY*"</code>
                    </strong>
                    " — not \"Aero Dynamics\" and not \"Infinity\". If you don't recognise a charge, look for that prefix before you contact your bank."
                </P>
                <P>
                    "When you ask us for a refund and we approve it, we instruct Lemon Squeezy to issue it. Lemon Squeezy also has refund discretion of its own, independent of this policy, and may issue a refund we would not have. This policy describes what we will do. It does not, and cannot, limit what Lemon Squeezy, your card issuer or your bank may do."
                </P>
            </Section>

            <Section n="2" anchor="statutory" title="Rights this policy does not limit">
                <P>
                    "This policy is about what " <strong>"we"</strong>
                    " will do. It is not a list of the only rights you have."
                </P>
                <P>
                    "Depending on where you live you have rights that cannot be excluded or limited by agreement — including under the Consumer Rights Act 2015 in the United Kingdom, Directive 2011/83/EU and Directive (EU) 2019/770 in the European Union and EEA, and the consumer guarantees under the Australian Consumer Law. Consumer law in your US state, Canadian province or other country may give you more as well."
                </P>
                <P>
                    "Where this policy gives you more than the law requires, this policy applies. "
                    <strong>
                        "Where the law gives you more than this policy, the law applies and we will follow it"
                    </strong>
                    " — we will not ask you to meet any condition in this policy before you use a right the law gives you. If part of this policy turns out to be unenforceable where you live, the rest still stands."
                </P>
            </Section>

            <Section n="3" anchor="change-of-mind" title="Refunds before you install">
                <P>
                    "If you have " <strong>"not yet installed"</strong>
                    " the aircraft, you can ask for a full refund within "
                    <strong>"14 days"</strong>
                    " of your order. We may approve it at our discretion — see section 3a — and where we do, we instruct Lemon Squeezy to issue it. Tell us briefly why; a sentence is plenty, and it helps us fix whatever prompted it."
                </P>
                <P>
                    <strong>
                        "Once Infinity Manager has installed the aircraft, the change-of-mind window is over."
                    </strong>
                    " Section 4 is unaffected by this: if the aircraft is faulty, doesn't work, or isn't as we described it, there is no deadline and no install condition."
                </P>
                <P>
                    <strong>"\"Installed\""</strong>
                    " means Infinity Manager has fetched that aircraft's files onto your PC. Buying the aircraft, the aircraft appearing in your account, signing in to Infinity Manager, and installing Infinity Manager itself are "
                    <strong>"not"</strong>
                    " installs of the aircraft."
                </P>
                <P>
                    "All deadlines are calculated in UTC. If your purchase hasn't reached your account yet and you can't install the aircraft at all, tell us — the 14 days don't run against you while that is the case."
                </P>
            </Section>

            <Section n="3a" anchor="discretion" title="Our discretion">
                <P>
                    "The refund described in section 3 is offered voluntarily, in addition to your legal rights, and is granted at our discretion. We may decline a change-of-mind request, including one made inside the refund window — for example where we reasonably consider refunds are being used to obtain aircraft for free, where the request or the underlying order appears fraudulent or was not made by you, or where the account shows repeated buy-and-refund cycles. Where we decline, we will tell you why."
                </P>
                <P>
                    <strong>"This discretion does not apply to sections 2, 4 or 5."</strong>
                    " A faulty aircraft, an aircraft that is not as we described it, a duplicate or mistaken charge, and any right your local consumer law gives you are not discretionary, and we will honour them."
                </P>
            </Section>

            <Section n="4" anchor="faulty" title="If something is wrong with the aircraft">
                <P>
                    "Section 3 is a change-of-mind window. "
                    <strong>
                        "It does not apply to an aircraft that is faulty, that doesn't work, or that isn't as we described it. There is no deadline in this policy for telling us about that."
                    </strong>
                </P>
                <P>
                    "Contact us and we will normally try to put it right first, usually with an update through Infinity Manager — that is usually faster for you than a refund. If we can't fix it in a reasonable time, or the problem is serious enough that a fix isn't a reasonable answer, we will refund you, whether or not the 14 days have passed and whether or not you have flown it."
                </P>
                <P>
                    "This covers defects in the aircraft itself, including an aircraft broken by a Microsoft Flight Simulator update — we treat that as ours to fix, not as your problem. It does not extend to features we have clearly described as planned rather than shipped."
                </P>
                <P>
                    "Your rights under section 2 apply here in full and may give you more than this section does."
                </P>
            </Section>

            <Section n="5" anchor="always" title="Always refunded, window or no window">
                <List>
                    <ListItem>"You were charged more than once for the same order."</ListItem>
                    <ListItem>"You bought an aircraft you already owned."</ListItem>
                    <ListItem>
                        "You bought the wrong aircraft or the wrong variant — including an aircraft for a simulator or platform you can't run. Tell us reasonably promptly."
                    </ListItem>
                    <ListItem>
                        "You paid full price when you qualified for an upgrade or owner discount; we refund the difference."
                    </ListItem>
                    <ListItem>
                        "You paid and the aircraft never reached your account, and we couldn't get it to you."
                    </ListItem>
                    <ListItem>
                        "Someone used your payment method without your permission. Tell us, and tell your bank."
                    </ListItem>
                </List>
            </Section>

            <Section n="6" anchor="effect" title="What a refund does">
                <List>
                    <ListItem>
                        "Lemon Squeezy returns the money to the payment method you used, in the original currency, for the amount you paid. "
                        <strong>"We do not issue store credit."</strong>
                        " Lemon Squeezy's published timescale is up to 10 days to reach your statement; your bank may take longer, and if your card is in a different currency your bank's exchange rates apply on the way back."
                    </ListItem>
                    <ListItem>
                        "Sales tax or VAT is handled by our merchant of record. In most cases it comes back with the purchase; where it has to be claimed separately we'll tell you how."
                    </ListItem>
                    <ListItem>
                        <strong>
                            "Your licence to that aircraft ends and we remove it from your account."
                        </strong>
                        " Infinity Manager will no longer download, reinstall or update it, and you will not receive further updates or support for it."
                    </ListItem>
                    <ListItem>
                        "You should delete any files already installed. We are not going to ask you to prove that you have — removing the entitlement is how we handle this."
                    </ListItem>
                    <ListItem>
                        "Refunding one aircraft does not affect any other aircraft you own, except as set out in section 7."
                    </ListItem>
                    <ListItem>
                        "You can buy the aircraft again later at the price then advertised. A past refund does not block you."
                    </ListItem>
                </List>
            </Section>

            <Section n="7" anchor="upgrades" title="Upgrade discounts and bundles">
                <P>"Some aircraft are cheaper if you already own another one."</P>
                <P>
                    <strong>
                        "If you refund the aircraft that earned you a discount, and you are keeping the discounted aircraft"
                    </strong>
                    ", we deduct the discount you already received from the refund. If you would rather refund both aircraft and the second one is still inside its own window, say so and we'll do that instead. We will tell you the exact amount before processing anything, and we will never charge your payment method again without asking you first."
                </P>
                <P>
                    "If we sell aircraft together as a bundle, the bundle refunds as one purchase: the 14 days run from the bundle order, and the change-of-mind refund is unavailable once any aircraft in the bundle has been installed. We don't refund part of a bundle."
                </P>
            </Section>

            <Section n="8" anchor="declined" title="What we will normally decline">
                <P>
                    "Once you have installed the aircraft, or the 14 days have passed, these are not by themselves grounds for a change-of-mind refund under section 3. Please tell us anyway — several of them turn out to be faults under section 4, and section 2 is unaffected either way."
                </P>
                <List>
                    <ListItem>
                        "Performance on your own PC. We can't predict frame rates on individual systems; if performance is far outside what the product page leads you to expect, that's section 4, not this list."
                    </ListItem>
                    <ListItem>
                        "You don't like a feature that the product page or the manual describes accurately, or the aircraft is more or less complex than you expected."
                    </ListItem>
                    <ListItem>"The aircraft went on sale after you bought it."</ListItem>
                    <ListItem>
                        "A third-party add-on, mod or tool conflicts with ours and a workaround exists."
                    </ListItem>
                    <ListItem>
                        "You've lost access to the account you bought with — we'd rather help you recover the account than refund the aircraft."
                    </ListItem>
                </List>
            </Section>

            <Section n="9" anchor="how-to-ask" title="How to ask">
                <P>
                    "Use the " <Internal href=nav::USER>"support form on your dashboard"</Internal>
                    ", or ask us on " <External href=nav::DISCORD_URL>"Discord"</External>
                    ". Include the order number or receipt email address from your Lemon Squeezy receipt, and the name of the aircraft. For a section 3 request, also tell us briefly why and confirm you haven't installed the aircraft."
                </P>
                <Callout tone=Tone::Warning title="If you can't sign in">
                    <p>
                        "The dashboard form needs you to be signed in. If you can't sign in, if you checked out with a different email address, or if your access has already been removed, "
                        <strong>
                            "you can go straight to Lemon Squeezy instead — you do not have to come to us first."
                        </strong>
                        " Use the contact details on your receipt, or manage your orders at "
                        <External href="https://app.lemonsqueezy.com/my-orders">
                            "app.lemonsqueezy.com/my-orders"
                        </External>
                        ". Our Discord is also open and does not require an Infinity account."
                    </p>
                </Callout>
                <P>
                    "We aim to reply within 2 business days and to decide within 5. If we need longer, we'll tell you why."
                </P>
            </Section>

            <Section n="10" anchor="chargebacks" title="Chargebacks — please talk to us first">
                <P>
                    "If something has gone wrong, contact us or Lemon Squeezy before you contact your bank. A chargeback takes weeks and costs us a fee on top of the refunded amount; a message usually takes a day. Lemon Squeezy's own buyer terms also ask you to contact them first."
                </P>
                <P>
                    "If a chargeback is decided in your favour and the money is returned to you, we remove that aircraft from your account, exactly as we would on a refund. We may decline future sales to an account with a pattern of chargebacks raised without contacting us first."
                </P>
                <P>
                    <strong>
                        "We will not hold a dispute against you if your payment method was used without your permission, or if we didn't get back to you in reasonable time."
                    </strong>
                    " Nothing here affects your right to dispute a charge, or the rights in section 2."
                </P>
            </Section>

            <Section n="11" anchor="abuse" title="Refund abuse">
                <P>
                    "If it looks like refunds are being used to get aircraft for free — repeated buy-and-refund cycles on the same account — we may stop offering discretionary refunds to that account and may decline future purchases, and we'll tell you if we do. "
                    <strong>"Sections 2 and 4 still apply in every case."</strong>
                    " Buying an aircraft again later, including on sale, is completely fine and is not abuse."
                </P>
            </Section>

            <Section n="12" anchor="scope" title="Scope">
                <P>"This policy covers paid Infinity aircraft bought on our website."</P>
                <P>
                    "Infinity Manager and the community freeware distributed through it are free, so there is nothing to refund. Closed beta access is free."
                </P>
                <P>
                    "If you bought an Infinity aircraft anywhere other than our website — the Microsoft Flight Simulator Marketplace, or any other store — that store took your money and that store's refund policy applies. Your receipt will tell you who sold it to you. We'll still help with any technical problem, wherever you bought it."
                </P>
                <P>
                    "If we ever sell pre-orders or early access, we'll publish the refund terms for them before taking any money."
                </P>
            </Section>

            <Section n="13" anchor="withdrawal" title="If an aircraft is withdrawn, or we stop trading">
                <P>
                    "Your licence to an aircraft you have bought is perpetual and we do not take it away because a product is retired. If we stop selling an aircraft, you keep it."
                </P>
                <P>
                    "If we ever lose the right to distribute an aircraft and have to remove it from your account, "
                    <strong>"we will refund you"</strong>
                    ", in full if it happens within a year of your purchase and on a fair pro-rata basis after that."
                </P>
                <P>
                    "Infinity Manager is required to install and update Infinity aircraft. If we ever wind down the Manager service, we will publish a way for existing owners to keep and reinstall the aircraft they have bought."
                </P>
                <P>
                    "If we close your account for breaching the "
                    <Internal href=nav::LEGAL_TERMS>"Terms of Service"</Internal>
                    " or the EULA, aircraft you have already bought are not refunded, and we will tell you why the account was closed."
                </P>
            </Section>

            <Section n="14" anchor="changes" title="Changes to this policy">
                <P>
                    "We may update this policy; every version carries a version number at the top. "
                    <strong>
                        "If it changes after you buy, we apply whichever version is more favourable to you."
                    </strong>
                </P>
            </Section>

            <Section n="15" anchor="who" title="Who we are">
                <P>
                    {format!("{PUBLISHER}, a limited liability company formed in the State of Iowa.")}
                </P>
                <P>
                    "Contact: the " <Internal href=nav::USER>"support form"</Internal>
                    " on your dashboard, or " <External href=nav::DISCORD_URL>"our Discord"</External>
                    "."
                </P>
                <P>
                    "Merchant of record for purchases: currently Lemon Squeezy. For questions about the payment itself, the tax charged, or your receipt, you can contact them using the details on your receipt."
                </P>
                <P>
                    "Licence terms: the End-User License Agreement, linked from every aircraft page."
                </P>
                <P>
                    "We keep your order record and the messages you send us for as long as we need them to handle refunds, support and tax records, including after a refund. See our "
                    <Internal href=nav::LEGAL_PRIVACY>"Privacy Policy"</Internal>
                    "."
                </P>
                <P>
                    {format!(
                        "Microsoft Flight Simulator is a trademark of Microsoft Corporation. {PUBLISHER} is not affiliated with, endorsed by, or sponsored by Microsoft Corporation.",
                    )}
                </P>
            </Section>
        </LegalPage>
    }
}
