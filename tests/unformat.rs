#![cfg(test)]

use std::{net::SocketAddr, str::FromStr};

use unfmt::unformat;

#[test]
fn test_unformat() {
    assert_eq!(unformat!("abc", "abc"), Some(()));
    assert_eq!(unformat!("abc", "abcd"), Some(()));
    assert_eq!(unformat!("abc", "acd"), None);
}

#[test]
fn test_unformat_captures() {
    assert_eq!(unformat!("{}", "abc"), Some("abc"));
    assert_eq!(unformat!("{}bc", "abc"), Some("a"));
    assert_eq!(unformat!("a{}c", "abc"), Some("b"));
    assert_eq!(unformat!("ab{}", "abc"), Some("c"));
    assert_eq!(unformat!("{}{}c", "abc"), None);
    assert_eq!(unformat!("{}b{}", "abc"), Some(("a", "c")));
    assert_eq!(unformat!("a{}c", "acd"), Some(""));
}

#[test]
fn test_unformat_indexed_captures() {
    assert_eq!(unformat!("{1}b{0}", "abc"), Some(("c", "a")));
}

#[test]
fn test_unformat_named_captures() {
    let mut name = None;
    assert_eq!(unformat!("ab{name}", "abc"), Some(()));
    assert_eq!(name, Some("c"));
}

#[test]
fn test_unformat_escaped_captures() {
    let mut name = None;
    assert_eq!(unformat!("a{{{name}}}c", "a{b}c"), Some(()));
    assert_eq!(name, Some("b"));
}

#[test]
fn test_unformat_typed_captures() {
    assert_eq!(unformat!("ab{:usize}", "ab152"), Some(152));
    assert_eq!(
        unformat!("ab{:SocketAddr}a", "ab127.0.0.1:3000a"),
        SocketAddr::from_str("127.0.0.1:3000").ok()
    );
}

#[test]
fn test_unformat_typed_named_captures() {
    let mut name = None;
    assert_eq!(unformat!("ab{name:usize}", "ab152"), Some(()));
    assert_eq!(name, Some(152));

    let mut addr = None;
    assert_eq!(
        unformat!("ab{addr:SocketAddr}a", "ab127.0.0.1:3000a"),
        Some(())
    );
    assert_eq!(addr, SocketAddr::from_str("127.0.0.1:3000").ok());
}

#[test]
fn test_declmacro() {
    macro_rules! test_declmacro {
        ($fmt:literal, $input:expr) => {
            unformat!($fmt, $input)
        };
    }

    test_declmacro!("abc", "abcd");
}
