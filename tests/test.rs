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
        m["k"] = a b c;
    }

    m! {
        let "k" = 1;
        assert_eq!("k", 1);
    }
}

#[test]
fn test_two_macros() {
    mashup! {
        m[x] = A B C;
        n[x] = D E F;
    }

    const ABC: &str = "abc";
    const DEF: &str = "def";

    assert_eq!(m![x], "abc");
    assert_eq!(n![x], "def");
}

#[test]
fn test_duplicate() {
    mashup! {
        m[K] = A B C;
        m[K] = D E F;
    }

    const ABC: &str = "abc";

    m! {
        assert_eq!(K, "abc");
    }
}

#[test]
fn test_repeat() {
    const ROCKET_A: &str = "/a";
    const ROCKET_B: &str = "/b";

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
    const CONST0: &str = "const0";

    mashup! {
        m["id"] = CONST 0;
    }

    assert_eq!(m!["id"], CONST0);
}
