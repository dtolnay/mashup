#![doc(html_root_url = "https://docs.rs/mashup-impl/0.1.9")]

#[macro_use]
extern crate proc_macro_hack;

extern crate proc_macro2;
use proc_macro2::{Delimiter, Ident, Literal, TokenStream, TokenTree};

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

type Input = Map<String, SubstitutionMacro>;

struct SubstitutionMacro {
    attrs: Vec<TokenStream>,
    patterns: Vec<Concat>,
}

struct Concat {
    tag: TokenStream,
    pieces: Vec<TokenTree>,
}

impl Concat {
    fn mashup(&self) -> String {
        self.pieces
            .iter()
            .map(|tt| match *tt {
                TokenTree::Literal(ref lit) => identify(lit),
                _ => tt.to_string(),
            }).collect()
    }
}

/// Returns a `to_string()` impl for `Literals`
/// which is safe for use in idents
fn identify(lit: &Literal) -> String {
    let stringified = lit.to_string();
    match stringified.chars().next() {
        Some(ch) if ch == '"' || ch == '\'' => stringified.trim_matches(ch).replace('-', "_"),
        _ => stringified,
    }
}

fn parse(tts: TokenStream) -> Input {
    let mut tts = tts.into_iter().peekable();
    let mut map = Map::new();
    let mut attrs = Vec::new();

    while let Some(next) = tts.next() {
        match next {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '#' {
                    if let Some(TokenTree::Group(group)) = tts.next() {
                        if group.delimiter() == Delimiter::Bracket {
                            attrs.push(group.stream());
                            continue;
                        }
                    }
                }
                panic!("unexpected mashup input");
            }
            TokenTree::Ident(ident) => {
                let name = ident.to_string();

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
                        TokenTree::Ident(mut ident) => {
                            let fragment = ident.to_string();
                            if fragment.starts_with("r#") {
                                ident = Ident::new(&fragment[2..], ident.span());
                            }
                            if fragment == "env" {
                                let resolve_env = match tts.peek() {
                                    Some(&TokenTree::Punct(ref p)) => p.as_char() == '!',
                                    _ => false,
                                };
                                if resolve_env {
                                    match tts.next().and_then(|_| tts.next()) {
                                        Some(TokenTree::Group(ref grp))
                                            if grp.delimiter() == Delimiter::Parenthesis =>
                                        {
                                            match grp.stream().into_iter().next() {
                                                Some(TokenTree::Literal(ref varname)) => {
                                                    let resolved = std::env::var(identify(varname))
                                                        .unwrap_or_else(|_| {
                                                            panic!(
                                                                "unresolvable mashup env var {}",
                                                                varname
                                                            )
                                                        });
                                                    pieces.push(TokenTree::Literal(
                                                        Literal::string(&resolved),
                                                    ));
                                                    continue;
                                                }
                                                _ => panic!("unexpected mashup input"),
                                            }
                                        }
                                        _ => panic!("unexpected mashup input"),
                                    }
                                }
                            }
                            pieces.push(TokenTree::Ident(ident));
                        }
                        TokenTree::Literal(_) => {
                            pieces.push(tt);
                        }
                        TokenTree::Punct(tt) => match tt.as_char() {
                            '_' | '\'' => pieces.push(TokenTree::Punct(tt)),
                            ';' => break,
                            other => panic!("unexpected op {:?}", other),
                        },
                        _ => panic!("unexpected mashup input"),
                    }
                }

                let substitution_macro = map.entry(name.to_string()).or_insert_with(|| {
                    SubstitutionMacro {
                        attrs: Vec::new(),
                        patterns: Vec::new(),
                    }
                });

                substitution_macro.attrs.append(&mut attrs);
                substitution_macro.patterns.push(Concat {
                    tag: tag,
                    pieces: pieces,
                });
            }
            _ => panic!("unexpected mashup input"),
        }
    }

    map
}

fn make_macro(name: String, substitution_macro: SubstitutionMacro) -> String {
    let mut attrs = String::new();
    let mut rules = String::new();

    for attr in substitution_macro.attrs {
        attrs += &format!("#[{}]", attr);
    }

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
    for (i, p) in substitution_macro.patterns.iter().enumerate() {
        all += " ";
        all += &p.mashup();

        let mut quadratic = String::new();
        for j in 0..substitution_macro.patterns.len() {
            quadratic += &format!(" $v{}:tt", j);
        }

        rules += &"
            // Replace target tokens with concatenated ident.
            (@(__mashup_all) (($($top:tt)*) $($stack:tt)*) __mashup_pattern $($rest:tt)*) => {
                __mashup_replace! {
                    @(__mashup_continue) (($($top)* __mashup_current) $($stack)*) $($rest)*
                }
            };
            "
            .replace("__mashup_replace", &name)
            .replace("__mashup_pattern", &p.tag.to_string())
            .replace("__mashup_all", &quadratic)
            .replace("__mashup_current", &format!("$v{}", i))
            .replace("__mashup_continue", &quadratic.replace(":tt", ""));
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

    format!("{} macro_rules! {} {{ {} }}", attrs, name, rules)
}
