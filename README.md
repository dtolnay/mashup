Mashup: a working stable concat\_idents
=======================================

[![Build Status](https://api.travis-ci.org/dtolnay/mashup.svg?branch=master)](https://travis-ci.org/dtolnay/mashup)
[![Latest Version](https://img.shields.io/crates/v/mashup.svg)](https://crates.io/crates/mashup)

The nightly-only [`concat_idents!`] macro in the Rust standard library is
notoriously underpowered in that its concatenated identifiers can only refer to
existing items, they can never be used to define something new.

[`concat_idents!`]: https://doc.rust-lang.org/std/macro.concat_idents.html

This crate provides a more flexible spin on concatenating idents.

```toml
[dependencies]
mashup = "0.1"
```

Mashup works with any Rust compiler version >= 1.15.0, although versions older
than 1.20.0 will need to use an ever so slightly less convenient syntax.

## So tell me about concatenating idents

This crate provides a `mashup!` macro-generating macro. You provide mashup a
mapping of arbitrary key tokens to idents that you want concatenated together,
and mashup defines for you a substitution macro that substitutes your key tokens
with the single concatenated ident corresponding to each one.

```rust
#[macro_use]
extern crate mashup;

// Use mashup to generate a substitution macro called m. The substitution macro
// will replace all occurrences of the key token "method" in its input with the
// single concatenated identifier abc.
mashup! {
    m["method"] = a b c;
}

struct Struct;

m! {
    impl Struct {
        fn "method"() {}
    }
}

fn main() {
    // Our struct now has an abc method.
    Struct::abc();
}
```

## Glossary

- **Substitution macro:** A macro produced by the `mashup!` macro. The input to
  `mashup!` provides a name for one or more substitution macros to be defined.
  The substitution macro in the short example above is called `m!`.

- **Key tokens:** Arbitrary tokens that are to be replaced by a single
  concatenated ident anywhere in the input of a substitution macro. The token
  `"method"` is used as a key token above.

- **Ident pieces:** Idents that are concatenated together to form a single
  concatenated ident in the final macro-expanded code. The `a` `b` `c` are ident
  pieces that come together to form the `abc` method name.

## More elaborate example

This example demonstrates some trickier uses of mashup.

- You may need to bundle a `mashup!` invocation inside of a more convenient
  user-facing macro of your own.

- The `mashup!` invocation may define multiple substitutions, including via the
  `$(...)*` repetition available in macro\_rules.

- Key tokens may consist of more than one token.

- Substitution macros work equally well in item position and expression
  position.

```rust
#[macro_use]
extern crate mashup;

const ROCKET_A: &str = "/a";
const ROCKET_B: &str = "/b";

macro_rules! routes {
    ($($route:ident),*) => {{
        mashup! {
            $(
                m["rocket-codegen-route" $route] = ROCKET_ $route;
            )*
        }

        m! {
            vec![$("rocket-codegen-route" $route),*]
        }
    }}
}

fn main() {
    let routes = routes!(A, B);
    assert_eq!(routes, vec!["/a", "/b"]);
}
```

## Limitations

- The `mashup!` macro may be invoked *at most once* within a lexical scope. To
  provide a way around this, you may use a single mashup invocation to define
  more than one substitution macro by using as many different substitution macro
  names within one invocation as you want.

- If any ident pieces are integer literals, the entire set of ident pieces must
  be parenthesized: `m[K] = (my_ $route 0);`.

- If targeting a Rust compiler older than 1.20.0, ident pieces must always be
  parenthesized.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
