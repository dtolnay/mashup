#[test]
fn test_attributes() {
    mashup! {
        /// Needs better documentation.
        #[doc(hidden)]
        m["T"] = A a;
    }

    struct Aa;
    let _: m!("T");
}
