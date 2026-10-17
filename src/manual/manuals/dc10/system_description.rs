use crate::manual::types::{ManualChapter, ManualSection};

pub const SYSTEM_DESCRIPTION: ManualChapter =
    ManualChapter::new("system-description", "System Description")
        .summary("How the aircraft and each of its systems work.")
        .sections(&[
            ManualSection::at("general", "Aircraft general"),
            ManualSection::at("flight-deck", "Flight deck"),
            ManualSection::at("flight-engineer-station", "Flight engineer station")
                .only(&["DC-10", "KC-10"]),
            ManualSection::at("efb", "Electronic flight bag"),
            ManualSection::at("air-conditioning", "Air conditioning and pressurization"),
            ManualSection::at("autoflight", "Autoflight"),
            ManualSection::at("communications", "Communications"),
            ManualSection::at("electrical", "Electrical"),
            ManualSection::at("fire-protection", "Fire protection"),
            ManualSection::at("flight-controls", "Flight controls"),
            ManualSection::at("fuel", "Fuel"),
            ManualSection::at("hydraulics", "Hydraulics"),
            ManualSection::at("ice-and-rain", "Ice and rain protection"),
            ManualSection::at("landing-gear", "Landing gear and brakes"),
            ManualSection::at("lighting", "Lighting"),
            ManualSection::at("navigation", "Navigation"),
            ManualSection::at("pneumatics", "Pneumatics"),
            ManualSection::at("apu", "APU"),
            ManualSection::at("engines", "Engines"),
            ManualSection::at("aerial-refueling", "Aerial refueling").only(&["KC-10"]),
        ]);
