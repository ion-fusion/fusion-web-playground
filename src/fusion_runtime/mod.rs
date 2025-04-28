use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(module = "/src/fusion_runtime/mod.js")]
extern "C" {
    pub type Classloader;
    pub type Runtime;

    #[wasm_bindgen(catch)]
    pub async fn create_classloader() -> Result<Classloader, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn create_fusion_runtime(lib: &Classloader) -> Result<Runtime, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn fusion_eval(
        lib: &Classloader,
        runtime: Runtime,
        expr: String,
    ) -> Result<JsValue, JsValue>;
}
