use std::collections::HashMap;

use sdk::tracing::field::FieldSet;
use sdk::tracing::{self, field, Dispatch, Level, Value};
use sdk::tracing::{Metadata, Span};
use sdk::tracing_core::callsite::Identifier;
use sdk::tracing_core::{self, Callsite, Kind};

pub fn get_meta<'a>() -> &'a Metadata<'static> {
    #[derive(Debug, Clone)]
    pub struct MyCallsite;

    impl Callsite for MyCallsite {
        fn set_interest(&self, _: tracing_core::Interest) {}
        fn metadata(&self) -> &Metadata<'_> {
            &META
        }
    }

    static CALLSITE: MyCallsite = MyCallsite;

    static META: Metadata<'static> = Metadata::new(
        "http_request",
        "my_target",
        Level::INFO,
        None,
        None,
        None,
        FieldSet::new(
            &[
                "otel.name",
                "http.request.method",
                "http.request.body.size",
                "http.route",
                "user.id", // ← campos extras declarados aqui
            ],
            Identifier(&CALLSITE),
        ),
        Kind::SPAN,
    );

    &META
}
