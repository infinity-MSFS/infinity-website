use crate::manual::types::{ManualChapter, ManualSection};

pub const EMERGENCY_PROCEDURES: ManualChapter =
    ManualChapter::new("emergency-procedures", "Emergency Procedures")
        .summary("Time-critical procedures for situations that threaten the aircraft.")
        .sections(&[
            ManualSection::at("general", "General"),
            ManualSection::at("engine-fire", "Engine fire or severe damage"),
            ManualSection::at("engine-failure", "Engine failure"),
            ManualSection::at("smoke-fire-fumes", "Smoke, fire or fumes"),
            ManualSection::at("rapid-decompression", "Rapid decompression"),
            ManualSection::at("emergency-descent", "Emergency descent"),
            ManualSection::at("emergency-landing", "Emergency landing and evacuation"),
        ]);
