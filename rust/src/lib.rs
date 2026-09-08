//! Version-pinned NTA 8800 calculation-kernel foundation.
//!
//! The browser `engine.js` remains the screening calculator. This crate is the
//! controlled boundary for the eventual NTA 8800 conformance kernel. The
//! upstream OpenAEC crates are pinned to one immutable commit so that a kernel
//! release cannot silently change because of a moving git dependency.

pub mod demand;
pub mod input;
pub mod kernel;
pub mod ventilation;

/// Exact upstream revision used by this kernel boundary.
pub const OPENAEC_REV: &str = "6e8738c075719e2fc8fcf918d969406f98927b07";

/// Current NTA target for this project.
pub const NTA_VERSION: &str = "NTA 8800:2025+C1:2026";

/// Current project input schema revision.
pub const INPUT_SCHEMA_VERSION: &str = "0.2.0";

/// Dependency manifest used for traceability and QMS review.
#[must_use]
pub const fn kernel_manifest() -> (&'static str, &'static str) { (NTA_VERSION, OPENAEC_REV) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_version_is_pinned() {
        let (nta, rev) = kernel_manifest();
        assert_eq!(nta, "NTA 8800:2025+C1:2026");
        assert_eq!(rev.len(), 40);
        assert!(rev.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn input_schema_version_is_explicit() { assert_eq!(INPUT_SCHEMA_VERSION, "0.2.0"); }

    #[test]
    fn upstream_natural_units_are_available() {
        use nta8800_model::time::Month;
        use nta8800_tables::climate::de_bilt_climate_data;
        use nta8800_transmission::MONTH_HOURS;
        let climate = de_bilt_climate_data();
        assert_eq!(Month::all().len(), 12);
        assert_eq!(MONTH_HOURS.iter().sum::<f64>(), 8760.0);
        assert!((climate.outdoor_temperature[Month::Januari] - 2.61).abs() < 1e-9);
    }
}
