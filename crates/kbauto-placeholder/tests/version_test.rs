//! Tests for PlaybookVersion parsing, display, and comparison.

use kbauto_placeholder::PlaybookVersion;

#[test]
fn parse_standard_version() {
    let v = PlaybookVersion::parse("1.2.3").expect("should parse 1.2.3");
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
}

#[test]
fn parse_version_with_v_prefix() {
    let v = PlaybookVersion::parse("v1.0.0").expect("should parse v1.0.0");
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 0);
    assert_eq!(v.patch, 0);
}

#[test]
fn display_version() {
    let v = PlaybookVersion {
        major: 1,
        minor: 2,
        patch: 3,
    };
    assert_eq!(format!("{v}"), "1.2.3");
}

#[test]
fn ordering_versions() {
    let v1 = PlaybookVersion::parse("1.0.0").unwrap();
    let v2 = PlaybookVersion::parse("1.1.0").unwrap();
    let v3 = PlaybookVersion::parse("1.1.1").unwrap();
    let v4 = PlaybookVersion::parse("2.0.0").unwrap();

    assert!(v1 < v2, "1.0.0 < 1.1.0");
    assert!(v2 < v3, "1.1.0 < 1.1.1");
    assert!(v3 < v4, "1.1.1 < 2.0.0");
}

#[test]
fn equality_versions() {
    let v1 = PlaybookVersion::parse("1.0.0").unwrap();
    let v2 = PlaybookVersion::parse("v1.0.0").unwrap();
    assert_eq!(v1, v2, "1.0.0 == v1.0.0");
}

#[test]
fn reject_invalid_version_too_few_parts() {
    let result = PlaybookVersion::parse("1.0");
    assert!(result.is_err(), "should reject version with too few parts");
}

#[test]
fn reject_invalid_version_too_many_parts() {
    let result = PlaybookVersion::parse("1.0.0.0");
    assert!(result.is_err(), "should reject version with too many parts");
}

#[test]
fn reject_invalid_version_non_numeric() {
    let result = PlaybookVersion::parse("a.b.c");
    assert!(result.is_err(), "should reject non-numeric version");
}

#[test]
fn from_str_trait() {
    use std::str::FromStr;
    let v = PlaybookVersion::from_str("3.2.1").expect("should parse via FromStr");
    assert_eq!(v.major, 3);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 1);
}
