//! The nightly-only [`concat_idents!`] macro in the Rust standard library is
//! notoriously underpowered in that its concatenated identifiers can only refer to
//! existing items, they can never be used to define something new.
//!
//! [`concat_idents!`]: https://doc.rust-lang.org/std/macro.concat_idents.html
//!
//! This crate provides a more flexible spin on concatenating idents.
//!
//! ```toml
//! [dependencies]
//! mashup = "0.1"
//! ```
//!
//! Mashup works with any Rust compiler version >= 1.15.0.
//!
//! # So tell me about concatenating idents
//!
//! This crate provides a `mashup!` macro-generating macro. You provide mashup a
//! mapping of arbitrary key tokens to idents that you want concatenated together,
//! and mashup defines for you a substitution macro that substitutes your key tokens
//! with the single concatenated ident corresponding to each one.
//!
//! ```rust
//! #[macro_use]
//! extern crate mashup;
//!
//! // Use mashup to generate a substitution macro called m. The substitution macro
//! // will replace all occurrences of the key token "method" in its input with the
//! // single concatenated identifier abc.
//! mashup! {
//!     m["method"] = a b c;
//! }
//!
//! struct Struct;
//!
//! m! {
//!     impl Struct {
//!         fn "method"() {}
//!     }
//! }
//!
//! fn main() {
//!     // Our struct now has an abc method.
//!     Struct::abc();
//! }
//! ```
//!
//! # Glossary
//!
//! - **Substitution macro:** A macro produced by the `mashup!` macro. The input to
//!   `mashup!` provides a name for one or more substitution macros to be defined.
//!   The substitution macro in the short example above is called `m!`.
//!
//! - **Key tokens:** Arbitrary tokens that are to be replaced by a single
//!   concatenated ident anywhere in the input of a substitution macro. The token
//!   `"method"` is used as a key token above.
//!
//! - **Ident pieces:** Idents that are concatenated together to form a single
//!   concatenated ident in the final macro-expanded code. The `a` `b` `c` are ident
//!   pieces that come together to form the `abc` method name.
//!
//! # More elaborate example
//!
//! This example demonstrates some trickier uses of mashup.
//!
//! - You may need to bundle a `mashup!` invocation inside of a more convenient
//!   user-facing macro of your own.
//!
//! - The `mashup!` invocation may define multiple substitutions, including via the
//!   `$(...)*` repetition available in macro\_rules.
//!
//! - Key tokens may consist of more than one token.
//!
//! - Substitution macros work equally well in item position and expression
//!   position.
//!
//! ```rust
//! #[macro_use]
//! extern crate mashup;
//!
//! const ROCKET_A: char = 'a';
//! const ROCKET_B: char = 'b';
//!
//! macro_rules! routes {
//!     ($($route:ident),*) => {{
//!         mashup! {
//!             $(
//!                 m["rocket-codegen-route" $route] = ROCKET_ $route;
//!             )*
//!         }
//!
//!         m! {
//!             vec![$("rocket-codegen-route" $route),*]
//!         }
//!     }}
//! }
//!
//! fn main() {
//!     let routes = routes!(A, B);
//!     assert_eq!(routes, vec!['a', 'b']);
//! }
//! ```
//!
//! ## Attributes
//!
//! Attributes for the substitution macro, including doc comments, may be provided
//! inside of the mashup invocation.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate mashup;
//! #
//! mashup! {
//!     /// Needs better documentation.
//!     #[macro_export]
//!     m1["w"] = W w;
//!     m1["x"] = X x;
//!
//!     #[macro_export]
//!     #[doc(hidden)]
//!     m2["y"] = Y y;
//!     m2["z"] = Z z;
//! }
//! #
//! # fn main() {}
//! ```
//!
//! # Limitations
//!
//! - The `mashup!` macro may be invoked *at most once* within a lexical scope. To
//!   provide a way around this, you may use a single mashup invocation to define
//!   more than one substitution macro by using as many different substitution macro
//!   names within one invocation as you want ([#5]).
//!
//! - As a consequence of hygiene, a concatenated identifier may not be used to
//!   refer to a captured local variable ([#6]).
//!
//! [#5]: https://github.com/dtolnay/mashup/issues/5
//! [#6]: https://github.com/dtolnay/mashup/issues/6

#![doc(html_root_url = "https://docs.rs/mashup/0.1.9")]
#![no_std]

#[macro_use]
extern crate proc_macro_hack;

#[allow(unused_imports)]
#[macro_use]
extern crate mashup_impl;
#[doc(hidden)]
pub use mashup_impl::*;

proc_macro_item_decl! {
    #[doc(hidden)]
    mashup_macro! => mashup_macro_impl
}

#[macro_export]
#[doc(hidden)]
macro_rules! mashup_parser {
    (@pieces ($($parse:tt)*) ; $($rest:tt)*) => {
        mashup_parser!(@begin ($($parse)* ;) $($rest)*);
    };
    (@pieces ($($parse:tt)*) $piece:tt $($rest:tt)*) => {
        mashup_parser!(@pieces ($($parse)* $piece) $($rest)*);
    };
    (@begin ($($parse:tt)*) $(#[$attr:meta])* $m:ident[$($n:tt)+] = $i:tt $($rest:tt)*) => {
        mashup_parser!(@pieces ($($parse)* $(#[$attr])* $m[$($n)+] = $i) $($rest)*);
    };
    (@begin ($($parse:tt)*)) => {
        mashup_macro!($($parse)*);
    };
}

/// A working stable concat_idents.
///
/// Refer to the **[crate-level documentation]**.
///
/// [crate-level documentation]: index.html
#[macro_export]
macro_rules! mashup {
    ($($mashup:tt)*) => {
        mashup_parser!(@begin () $($mashup)*);
    }
}
