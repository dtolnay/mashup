#[test]
fn test_raw_identifier() {
    mashup! {
        m["x"] = F r#move;
    }

    m! {
        struct "x";
    }

    let _ = Fmove;
}
