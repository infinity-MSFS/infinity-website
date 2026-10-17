use crate::manual::types::Manual;

mod abnormal_procedures;
mod adverse_weather;
mod emergency_procedures;
mod flight_characteristics;
mod introduction;
mod limitations;
mod normal_procedures;
mod system_description;

// One manual for the DC-10, KC-10 and MD-10. They share most of their content;
// anything that belongs to only some of them carries `applies_to`, and the
// reader shows only what applies to the aircraft picked. The order of
// `chapters` is the reading order and sets the section numbers.
pub static DC10_MANUAL: Manual = Manual {
    id: "dc10",
    title: "DC-10 / KC-10 / MD-10 Flight Manual",
    aircraft: &["DC-10", "KC-10", "MD-10"],
    chapter_label: Some("Section"),
    revision: Some("Draft"),
    summary: Some("Systems, procedures, limitations and handling for the DC-10, KC-10 and MD-10."),
    introduction: introduction::INTRODUCTION,
    chapters: &[
        system_description::SYSTEM_DESCRIPTION,
        emergency_procedures::EMERGENCY_PROCEDURES,
        abnormal_procedures::ABNORMAL_PROCEDURES,
        limitations::LIMITATIONS,
        normal_procedures::NORMAL_PROCEDURES,
        flight_characteristics::FLIGHT_CHARACTERISTICS,
        adverse_weather::ADVERSE_WEATHER,
    ],
};
