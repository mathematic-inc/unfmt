#![no_std]

extern crate alloc;
extern crate proc_macro;

use alloc::{borrow::ToOwned, format, vec::Vec};

use bstr::ByteSlice;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, parse_str, Expr, ExprLit, Ident, Lit, LitBool, LitByteStr, Token, TypePath,
};

struct Unformat {
    pattern: Vec<u8>,
    text: Expr,
    is_pattern_str: bool,
    full_match: bool,
}

impl Parse for Unformat {
    fn parse(input: ParseStream) -> Result<Self> {
        #[allow(clippy::wildcard_enum_match_arm)]
        let (pattern, is_pattern_str) = match input.parse::<Expr>()? {
            Expr::Lit(ExprLit {
                lit: Lit::Str(str), ..
            }) => (str.value().into_bytes(), true),
            Expr::Lit(ExprLit {
                lit: Lit::ByteStr(byte_str),
                ..
            }) => (byte_str.value(), false),
            _ => return Err(input.error("expected a string literal")),
        };

        input.parse::<Token![,]>()?;

        let text = input.parse::<Expr>()?;

        let full_match = if input.parse::<Token![,]>().is_ok() {
            input.parse::<LitBool>()?.value
        } else {
            false
        };
        Ok(Self {
            pattern,
            text,
            is_pattern_str,
            full_match,
        })
    }
}

enum Assignee {
    Index(u32),
    Variable(Ident),
}

impl Assignee {
    fn new(variable: &str, index: &mut u32) -> Self {
        variable.parse::<u32>().map_or_else(
            |_| {
                if variable.is_empty() {
                    let tuple_index = *index;
                    *index = index.saturating_add(1);
                    Self::Index(tuple_index)
                } else {
                    Self::Variable(parse_str(variable).expect("invalid variable name"))
                }
            },
            Self::Index,
        )
    }
}

enum CaptureTypePath {
    Str,
    Bytes,
    Typed(TypePath),
}

impl CaptureTypePath {
    fn new(type_path: &str, is_pattern_str: bool) -> Self {
        if type_path.is_empty() {
            if is_pattern_str {
                Self::Str
            } else {
                Self::Bytes
            }
        } else if type_path == "&str" {
            Self::Str
        } else if type_path == "&[u8]" {
            Self::Bytes
        } else {
            Self::Typed(parse_str(type_path).expect("invalid type path"))
        }
    }
}

impl ToTokens for CaptureTypePath {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match *self {
            Self::Str => {
                quote! { &str }
            }
            Self::Bytes => {
                quote! { &[u8] }
            }
            Self::Typed(ref type_path) => {
                quote! { #type_path }
            }
        });
    }
}

struct Capture {
    text: Vec<u8>,
    assignee: Assignee,
    r#type: CaptureTypePath,
}

impl Capture {
    fn new(text: &[u8], capture: &str, is_pattern_str: bool, index: &mut u32) -> Self {
        let (variable, type_path) = capture.split_once(':').unwrap_or((capture, ""));
        Self {
            text: text.to_vec(),
            assignee: Assignee::new(variable, index),
            r#type: CaptureTypePath::new(type_path, is_pattern_str),
        }
    }
}

impl ToTokens for Capture {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let rhs = match self.r#type {
            CaptureTypePath::Str => {
                quote! {
                    if let Ok(__unfmt_left) = __unfmt_left.to_str() {
                        __unfmt_left
                    } else {
                        break 'unformat None;
                    }
                }
            }
            CaptureTypePath::Bytes => {
                quote! { __unfmt_left }
            }
            CaptureTypePath::Typed(ref type_path) => {
                quote! {
                    if let Ok(Ok(__unfmt_left)) = __unfmt_left.to_str().map(|value| value.parse::<#type_path>()) {
                        __unfmt_left
                    } else {
                        break 'unformat None;
                    }
                }
            }
        };
        let assignment = match self.assignee {
            Assignee::Index(ref index) => {
                let ident = Ident::new(&format!("__unfmt_capture_{index}"), Span::call_site());
                quote! { let #ident = #rhs }
            }
            Assignee::Variable(ref ident) => {
                quote! { #ident = Some(#rhs) }
            }
        };
        let text = LitByteStr::new(&self.text, Span::call_site());

        // If text is empty, `find` will return `Some(0)` and the capture will
        // be at the end of the pattern, so this capture (`__unfmt_left`) would
        // be empty. Since captures are inherently .*? in regex, this capture
        // should consume the remainder of the text, so we swap `__unfmt_left`
        // and `__unfmt_right` to achieve this.
        tokens.extend(if self.text.is_empty() {
            quote! { let (__unfmt_left, __unfmt_right) = (__unfmt_byte_text, b""); }
        } else {
            quote! {
                let Some((__unfmt_left, __unfmt_right)) = __unfmt_byte_text.split_once_str(#text) else {
                    break 'unformat None;
                };
            }
        });

        tokens.extend(quote! {
            #assignment;
            __unfmt_byte_text = BStr::new(__unfmt_right);
        });
    }
}

/// Basic implementation of reversing the `format!` process. Matches a given
/// text against a given pattern, returning any captures.
///
/// Rules:
///
///  - Patterns are substring matched.
///  - Captures are written as `{<index-or-var>?(:<type>)?}` in the pattern.
///  - Captures are similar to `(.*?)` in regex, but without backtracking.
///  - Sequential captures (e.g. `{}{}`) are not supported and will return
///    `None`.
///
/// # Panics
///
/// This function panics if the pattern is invalid. This includes:
///
///  - Consecutive captures.
///  - Unmatched `}` in the pattern.
///  - Invalid UTF-8 in capture names.
///
#[proc_macro]
pub fn unformat(input: TokenStream) -> TokenStream {
    let Unformat {
        pattern,
        text,
        is_pattern_str,
        full_match,
    } = parse_macro_input!(input as Unformat);

    let (initial_part, captures) = compile(&pattern, is_pattern_str);
    let initial_part = Lit::ByteStr(LitByteStr::new(&initial_part, Span::call_site()));

    let capture_idents = {
        let mut capture_indices = captures
            .iter()
            .filter_map(|capture| match capture.assignee {
                Assignee::Index(capture_index) => Some(capture_index),
                Assignee::Variable(..) => None,
            })
            .collect::<Vec<_>>();

        capture_indices.sort_by(|&index_a, &index_b| index_a.cmp(&index_b));

        capture_indices
            .into_iter()
            .map(|index| Ident::new(&format!("__unfmt_capture_{index}"), Span::call_site()))
            .collect::<Vec<_>>()
    };

    let capture_block = if full_match {
        quote! {
            if !__unfmt_left.is_empty() {
                break 'unformat None;
            }
            #(#captures)*
            if !__unfmt_byte_text.is_empty() {
                break 'unformat None;
            }
        }
    } else {
        quote! { #(#captures)* }
    };

    TokenStream::from(quote! {
        'unformat: {
            use ::std::str::FromStr;
            use ::unfmt::bstr::{ByteSlice, BStr};
            let Some((__unfmt_left, mut __unfmt_byte_text)) = BStr::new(#text).split_once_str(#initial_part) else {
                break 'unformat None;
            };
            #capture_block
            Some((#(#capture_idents,)*))
        }
    })
}

fn compile(pattern: &[u8], is_pattern_str: bool) -> (Vec<u8>, Vec<Capture>) {
    let pattern = pattern
        .replace(b"{{", "\u{f8fd}")
        .replace(b"}}", "\u{f8fe}");
    let mut pattern_parts = pattern.split_str("{");

    // SAFETY: The first part is always present.
    let initial_part = unsafe {
        pattern_parts
            .next()
            .unwrap_unchecked()
            .replace("\u{f8ff}", "{{")
    };

    let mut current_index: u32 = 0;
    let mut compiled_pattern = Vec::new();
    for pattern_part in pattern_parts {
        let (capture, text) = pattern_part
            .split_once_str("}")
            .expect("unmatched } in pattern");
        let capture = capture
            .to_str()
            .expect("invalid UTF-8 in capture names")
            .to_owned();
        let text = text.replace("\u{f8fd}", "{").replace("\u{f8fe}", "}");
        compiled_pattern.push(Capture::new(
            &text,
            &capture,
            is_pattern_str,
            &mut current_index,
        ));
    }

    assert!(
        compiled_pattern.windows(2).all(|parts| parts
            .iter()
            .any(|&Capture { ref text, .. }| !text.is_empty())),
        "consecutive captures are not allowed"
    );

    (initial_part, compiled_pattern)
}
