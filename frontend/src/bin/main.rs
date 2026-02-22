// frontend/src/bin/main.rs

// dependencies
use rusty_quote_generator_client::app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
