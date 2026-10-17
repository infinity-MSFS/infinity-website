use crate::manual::types::{ManualChapter, ManualSection};

pub const ABNORMAL_PROCEDURES: ManualChapter = ManualChapter::new(
    "abnormal-procedures",
    "Abnormal Procedures",
)
.summary(
    "Procedures for failures and other non-normal situations that aren't immediately dangerous.",
)
.sections(&[
    ManualSection::at("general", "General"),
    ManualSection::at("electrical", "Electrical"),
    ManualSection::at("hydraulics", "Hydraulics"),
    ManualSection::at("fuel", "Fuel"),
    ManualSection::at("engines", "Engines and APU"),
    ManualSection::at("flight-controls", "Flight controls"),
    ManualSection::at("landing-gear", "Landing gear and brakes"),
    ManualSection::at("pressurization", "Pressurization and air conditioning"),
    ManualSection::at("aerial-refueling", "Aerial refueling").only(&["KC-10"]),
]);
