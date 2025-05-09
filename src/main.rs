mod code_input;
mod fusion_runtime;
mod fusion_sandbox;

use fusion_sandbox::FusionSandbox;

fn main() {
    // Release builds should only log at info+, dev builds log at debug+
    #[cfg(not(debug_assertions))]
    let config = wasm_logger::Config::new(log::Level::Info);
    #[cfg(debug_assertions)]
    let config = wasm_logger::Config::new(log::Level::Debug);
    wasm_logger::init(config);

    yew::Renderer::<FusionSandbox>::new().render();
}
