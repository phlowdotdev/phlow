use sdk::tracing::field::FieldSet;
use sdk::tracing::{field, Dispatch, Level, Value};
use sdk::tracing::{Metadata, Span};
use sdk::tracing_core::callsite::Identifier;
use sdk::tracing_core::{self, Callsite, Kind};

#[derive(Debug, Clone)]
pub struct MyCallsite;

impl Callsite for MyCallsite {
    fn set_interest(&self, _: tracing_core::Interest) {}
    fn metadata(&self) -> &Metadata<'_> {
        &META
    }
}

pub static CALLSITE: MyCallsite = MyCallsite;

pub static META: Metadata<'static> = Metadata::new(
    "http_request",  // nome do span
    "my_target",     // target
    Level::INFO,     // nível
    Some("file.rs"), // arquivo
    Some(123),       // linha
    Some("1"),       // coluna
    FieldSet::new(
        &[
            "otel.name",
            "http.request.method",
            "http.request.body.size",
            "http.route",
        ],
        Identifier(&CALLSITE),
    ), // field set
    Kind::SPAN,      // tipo: SPAN
);
