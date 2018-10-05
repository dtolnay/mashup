#[macro_use]
extern crate mashup;

#[test]
fn test_basic() {
    mashup! {
        m["k"] = a b c;
    }

    m! {
        impl Struct {
            fn "k"() {}
        }
    }

    struct Struct;
    Struct::abc();
}

#[test]
fn test_shared_hygiene() {
    mashup! {
        m["a"] = a a;
        m["b"] = b b;
    }

    m! {
        let "a" = 1;
        let "b" = 2;
        assert_eq!("a" + 1, "b");
    }
}

#[test]
fn test_two_macros() {
    mashup! {
        m[x] = A B C;
        n[x] = D E F;
    }

    const ABC: &'static str = "abc";
    const DEF: &'static str = "def";

    assert_eq!(m![x], "abc");
    assert_eq!(n![x], "def");
}

#[test]
fn test_duplicate() {
    mashup! {
        m[K] = A B C;
        m[K] = D E F;
    }

    const ABC: &'static str = "abc";

    m! {
        assert_eq!(K, "abc");
    }
}

#[test]
fn test_repeat() {
    const ROCKET_A: &'static str = "/a";
    const ROCKET_B: &'static str = "/b";

    macro_rules! routes {
        ($($route:ident),*) => {{
            mashup! {
                $(
                    m["rocket" $route] = ROCKET_ $route;
                )*
            }

            m! {
                vec![$("rocket" $route),*]
            }
        }}
    }

    let routes = routes!(A, B);
    assert_eq!(routes, vec!["/a", "/b"]);
}

#[test]
fn test_integer() {
    const CONST0: &'static str = "const0";

    mashup! {
        m["id"] = CONST 0;
    }

    assert_eq!(m!["id"], CONST0);
}

#[test]
fn test_empty() {
    mashup!{}
}

#[test]
fn test_underscore() {
    mashup! {
        m[X] = A _ B;
    }

    m! {
        const A_B: usize = 0;
        assert_eq!(X, 0);
    }
}

#[test]
fn test_lifetime() {
    mashup! {
        m['life time] = 'd e;
    }

    m! {
        #[allow(dead_code)]
        struct S<'life time> {
            q: &'life time str,
        }
    }
}

#[test]
fn test_type_macro() {
    mashup! {
        m["T"] = A a;
    }

    struct Aa;
    type Foo = m!["T"];
    let _: Foo = Aa;
}

#[test]
fn test_pattern_macro() {
    mashup! {
        m["T"] = A a;
    }

    struct Aa(usize);
    let m!["T"(i)] = Aa(1);
    assert_eq!(i, 1);
}

#[test]
fn test_keyword() {
    mashup! {
        m["x"] = F move;
    }

    m! {
        struct "x";
    }

    let _ = Fmove;
}

#[test]
fn test_literal_str() {
    mashup! {
        m["x"] = Foo "Bar-Baz";
    }

    m! {
        struct "x";
    }

    let _ = FooBar_Baz;
}

#[test]
fn test_env_literal() {
    mashup! {
        m["x"] = Lib env bar;
    }

    m! {
        struct "x";
    }

    let _ = Libenvbar;
}

#[test]
fn test_env_present() {
    mashup! {
        m["x"] = Lib env!("CARGO_PKG_NAME");
    }

    m! {
        struct "x";
    }

    let _ = Libmashup;
}

macro_rules! conditionally_ignore {
    {
        #[cfg(not($cfg:ident))]
        mod $name:ident;
    } => {
        #[cfg(not($cfg))]
        include!(concat!("conditional/", stringify!($name), ".rs"));

        #[cfg($cfg)]
        #[test]
        #[ignore]
        fn $name() {
            panic!("not tested");
        }
    };
}

conditionally_ignore! {
    #[cfg(not(no_attributes))]
    mod test_attributes;
}

conditionally_ignore! {
    #[cfg(not(no_raw_identifiers))]
    mod test_raw_identifiers;
}
