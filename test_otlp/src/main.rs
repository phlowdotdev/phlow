use libloading::Library;
use sdk::otlp::init_tracing_subscriber;
use sdk::tracing::{dispatcher, span, Dispatch, Level, Span};

type PluginFn = unsafe extern "C" fn(*mut Span, *const Dispatch);

fn main() {
    // Inicia o OpenTelemetry e Tracing no processo principal
    let _guard = init_tracing_subscriber().expect("failed to initialize tracing");

    // Cria o span principal
    let span = span!(Level::INFO, "main", component = "main_binary");
    let _enter = span.enter();

    sdk::tracing::info!("Log dentro do span main");

    // Pega o subscriber (Dispatch) atual
    let dispatch = dispatcher::get_default(|d| Box::into_raw(Box::new(d.clone())));

    // Carrega a biblioteca dinâmica
    unsafe {
        let lib = Library::new("./target/debug/libtracer.so").expect("Failed to load library");

        let func: libloading::Symbol<PluginFn> = lib.get(b"plugin").expect("Failed to get symbol");

        // Chama a função da lib com o Span e o Dispatch
        func(&span as *const _ as *mut _, dispatch);
    }

    // Espera alguns segundos para o exporter enviar tudo (opcional)
    std::thread::sleep(std::time::Duration::from_secs(2));
}
