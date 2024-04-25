# unfmt

[![crates.io](https://img.shields.io/crates/v/unfmt?style=flat-square)](https://crates.io/crates/unfmt)
[![license](https://img.shields.io/crates/l/unfmt?style=flat-square)](https://github.com/mathematic-inc/unfmt)
[![ci](https://img.shields.io/github/actions/workflow/status/mathematic-inc/unfmt/ci.yaml?label=ci&style=flat-square)](https://github.com/mathematic-inc/unfmt/actions/workflows/ci.yaml)
[![docs](https://img.shields.io/docsrs/unfmt?style=flat-square)](https://docs.rs/unfmt/latest/unfmt/)

`unfmt` is a compile-time pattern matching library that reverses the
interpolation process of `format!`.

You can think of it as an extremely lightweight regular expression engine
without the runtime pattern-compilation cost.

## Installation

```sh
cargo add -D unfmt
```

## Usage

```rs
let value = "My name is Rho.";

// Unnamed captures are returned as tuples.
assert_eq!(
    unformat!("My {} is {}.", value),
    Some(("name", "Rho"))
);

// You can put indices as well; just make sure ALL captures use indices
// otherwise it's not well defined.
assert_eq!(
    unformat!("My {1} is {0}.", value),
    Some(("Rho", "name"))
);

// You can also name captures using variables, but make sure you check the
// return is not None.
let subject;
let object;
assert_eq!(
    unformat!("My {subject} is {object}.", value),
    Some(())
);
assert_eq!((subject, object), (Some("name"), Some("Rho")));

// If a type implements `FromStr`, you can use it as a type argument. This
// is written as `{:Type}`.
assert_eq!(
    unformat!("Listening on {:url::Url}", "Listening on http://localhost:3000"),
    Some((url::Url::from_str("http://localhost:3000").unwrap(),))
);
```

In general, captures are written as `{<index-or-variable>:<type>}`. Multiple
captures in a row (i.e. `{}{}`) are not supported as they aren't well-defined.

## Limitations

- There is no backtracking.
