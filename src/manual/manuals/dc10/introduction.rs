use crate::manual::types::{CalloutTone, ManualBlock, ManualSection};

// Shown on the manual's main page, below the list of sections.
pub const INTRODUCTION: &[ManualSection] = &[
    ManualSection::at("about", "About this manual").blocks(&[
        ManualBlock::text(
            "This manual covers the DC-10, the KC-10 and the MD-10. The three share most of their systems and procedures, so they share one manual.",
        ),
        ManualBlock::callout(
            CalloutTone::Note,
            "This manual is written for Microsoft Flight Simulator. Don't use it for real-world flight.",
        )
        .titled("Simulation use only"),
    ]),
    ManualSection::at("aircraft", "Choosing your aircraft").blocks(&[
        ManualBlock::text(
            "Pick your aircraft with the DC-10, KC-10 and MD-10 buttons at the top of the page. The manual then shows only what applies to that aircraft. Anything that applies to some of them but not all is marked, for example **DC-10 / KC-10 only**.",
        ),
        ManualBlock::table(
            &["Aircraft", "Flight deck"],
            &[
                &[
                    "DC-10",
                    "Three-crew, with a flight engineer station behind the pilots",
                ],
                &[
                    "KC-10",
                    "Military tanker based on the DC-10-30, with a flight engineer station and an aerial refueling operator station",
                ],
                &["MD-10", "Two-crew glass flight deck, with no flight engineer"],
            ],
        ),
    ]),
    ManualSection::at("conventions", "Warnings, cautions and notes").blocks(&[
        ManualBlock::callout(
            CalloutTone::Warning,
            "A warning marks a step that can cause a serious failure or end the flight if it's done wrong.",
        ),
        ManualBlock::callout(
            CalloutTone::Caution,
            "A caution marks a step that can damage an aircraft system.",
        ),
        ManualBlock::callout(
            CalloutTone::Note,
            "A note adds information that helps but isn't critical.",
        ),
    ]),
    ManualSection::at("using-this-manual", "Finding your way around").blocks(&[
        ManualBlock::list(&[
            "The manual has seven sections, listed above. Open one from the list, or from the section menu at the top of any page.",
            "Inside a section, its contents are listed on the left. On a phone, they're under the **Contents** button.",
            "Search covers the whole manual, for the aircraft you've picked.",
            "Every heading has a link button. Use it to share a link that opens the manual at that spot, for the same aircraft.",
        ]),
    ]),
    ManualSection::at("installing", "Installing and updating").blocks(&[
        ManualBlock::steps(&[
            "Download the Infinity Manager from your account page and install it.",
            "Sign in to the Manager with the account you bought the aircraft with.",
            "Open **My Products**, find the aircraft and install it.",
        ]),
        ManualBlock::text("Updates are delivered through the Infinity Manager."),
    ]),
    ManualSection::at("getting-help", "Getting help").blocks(&[
        ManualBlock::text(
            "If something isn't covered here, ask on our [Discord server](https://discord.gg/HZt4MCEYZk). When you report a problem, include:",
        ),
        ManualBlock::list(&[
            "Which aircraft you were flying.",
            "The aircraft version shown in the Infinity Manager.",
            "Your simulator version.",
            "What you did, what you expected, and what happened instead.",
            "Screenshots or a video, if you have them.",
        ]),
    ]),
];
