#[macro_use]
extern crate proc_macro_hack;

extern crate proc_macro2;
use proc_macro2::{Delimiter, TokenStream, TokenTree};

use std::collections::BTreeMap as Map;
use std::str::FromStr;

proc_macro_item_impl! {
    pub fn mashup_macro_impl(s: &str) -> String {
        let tts = TokenStream::from_str(s).unwrap();
        let input = parse(tts);

        let mut macros = String::new();
        for (name, concat) in input {
            macros += &make_macro(name, concat);
        }
        macros
    }
}

type Input = Map<String, Patterns>;
type Patterns = Vec<Concat>;

struct Concat {
    tag: TokenStream,
    pieces: Vec<TokenTree>,
}

impl Concat {
    fn mashup(&self) -> String {
        self.pieces.iter().map(ToString::to_string).collect()
    }
}

fn parse(tts: TokenStream) -> Input {
    let mut tts = tts.into_iter();
    let mut map = Map::new();

    while let Some(name) = tts.next() {
        let tag = match tts.next() {
            Some(TokenTree::Group(group)) => {
                assert_eq!(group.delimiter(), Delimiter::Bracket);
                group.stream()
            }
            _ => panic!("unexpected mashup input"),
        };

        assert_eq!(tts.next().unwrap().to_string(), "=");

        let mut pieces = Vec::new();
        while let Some(tt) = tts.next() {
            match tt {
                tt @ TokenTree::Term(_) | tt @ TokenTree::Literal(_) => {
                    pieces.push(tt);
                }
                TokenTree::Op(tt) => {
                    match tt.op() {
                        '_' => pieces.push(TokenTree::Op(tt)),
                        ';' => break,
                        other => panic!("unexpected op {:?}", other),
                    }
                }
                _ => panic!("unexpected mashup input"),
            }
        }

        map.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(Concat { tag: tag, pieces: pieces });
    }

    map
}

fn make_macro(name: String, patterns: Patterns) -> String {
    let mut rules = String::new();

    rules += &"
        // Open parenthesis.
        (@($($v:tt)*) ($($stack:tt)*) ($($first:tt)*) $($rest:tt)*) => {
            __mashup_replace! {
                @($($v)*) (() $($stack)*) $($first)* __mashup_close_paren $($rest)*
            }
        };

        // Open square bracket.
        (@($($v:tt)*) ($($stack:tt)*) [$($first:tt)*] $($rest:tt)*) => {
            __mashup_replace! {
                @($($v)*) (() $($stack)*) $($first)* __mashup_close_bracket $($rest)*
            }
        };

        // Open curly brace.
        (@($($v:tt)*) ($($stack:tt)*) {$($first:tt)*} $($rest:tt)*) => {
            __mashup_replace! {
                @($($v)*) (() $($stack)*) $($first)* __mashup_close_brace $($rest)*
            }
        };

        // Close parenthesis.
        (@($($v:tt)*) (($($close:tt)*) ($($top:tt)*) $($stack:tt)*) __mashup_close_paren $($rest:tt)*) => {
            __mashup_replace! {
                @($($v)*) (($($top)* ($($close)*)) $($stack)*) $($rest)*
            }
        };

        // Close square bracket.
        (@($($v:tt)*) (($($close:tt)*) ($($top:tt)*) $($stack:tt)*) __mashup_close_bracket $($rest:tt)*) => {
            __mashup_replace! {
                @($($v)*) (($($top)* [$($close)*]) $($stack)*) $($rest)*
            }
        };

        // Close curly brace.
        (@($($v:tt)*) (($($close:tt)*) ($($top:tt)*) $($stack:tt)*) __mashup_close_brace $($rest:tt)*) => {
            __mashup_replace! {
                @($($v)*) (($($top)* {$($close)*}) $($stack)*) $($rest)*
            }
        };
        "
        .replace("__mashup_replace", &name);

    let mut all = String::new();
    for (i, p) in patterns.iter().enumerate() {
        all += " ";
        all += &p.mashup();

        let mut quadratic = String::new();
        for (j, q) in patterns.iter().enumerate() {
            if i == j {
                quadratic += " $v:tt";
            } else {
                quadratic += " ";
                quadratic += &q.mashup();
            }
        }

        rules += &"
            // Replace target tokens with concatenated ident.
            (@(__mashup_all) (($($top:tt)*) $($stack:tt)*) __mashup_pattern $($rest:tt)*) => {
                __mashup_replace! {
                    @(__mashup_continue) (($($top)* $v) $($stack)*) $($rest)*
                }
            };
            "
            .replace("__mashup_replace", &name)
            .replace("__mashup_pattern", &p.tag.to_string())
            .replace("__mashup_all", &quadratic)
            .replace("__mashup_continue", &quadratic.replace("$v:tt", "$v"));
    }

    rules += &"
        // Munch a token that is not one of the targets.
        (@($($v:tt)*) (($($top:tt)*) $($stack:tt)*) $first:tt $($rest:tt)*) => {
            __mashup_replace! {
                @($($v)*) (($($top)* $first) $($stack)*) $($rest)*
            }
        };

        // Done.
        (@($($v:tt)*) (($($top:tt)+))) => {
            $($top)+
        };

        // Launch.
        ($($tt:tt)*) => {
            __mashup_replace! {
                @(__mashup_all) (()) $($tt)*
            }
        }
        "
        .replace("__mashup_replace", &name)
        .replace("__mashup_all", &all);

    format!("macro_rules! {} {{ {} }}", name, rules)
}
