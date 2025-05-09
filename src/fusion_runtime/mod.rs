use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(module = "/src/fusion_runtime/mod.js")]
extern "C" {
    pub type Classloader;
    pub type Runtime;

    #[wasm_bindgen(catch)]
    async fn js_create_classloader() -> Result<Classloader, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_create_fusion_runtime(lib: &Classloader) -> Result<Runtime, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_fusion_eval(
        lib: &Classloader,
        runtime: &Runtime,
        expr: String,
    ) -> Result<JsValue, JsValue>;
}

fn as_string_or_blank<J: AsRef<JsValue>>(js_value: J) -> String {
    js_value.as_ref().as_string().unwrap_or_default()
}

/// Creates a [CheerpJ Classloader] that has access to fusion-java and ion-java on its
/// classpath. This is needed for the other runtime methods to load and invoke their
/// class methods.
///
/// [CheerpJ Classloader]: https://cheerpj.com/docs/reference/CJ3Library
pub async fn create_classloader() -> Result<Classloader, String> {
    js_create_classloader().await.map_err(as_string_or_blank)
}

/// Creates a [`FusionRuntime`] Java object, which is a re-usable object for creating top levels
/// for invocation in `fusion_eval`.
///
/// [FusionRuntime]: https://docs.ion-fusion.dev/latest/javadoc/dev/ionfusion/fusion/FusionRuntime.html
pub async fn create_fusion_runtime(lib: &Classloader) -> Result<Runtime, String> {
    js_create_fusion_runtime(lib)
        .await
        .map_err(as_string_or_blank)
}

/// Evaluates a Fusion expression using the [CheerpJ Classloader] and Fusion runtime.
/// Each invocation uses a [`SandboxBuilder`] to create a sandboxed [`TopLevel`]
/// with the `/fusion` language and then uses [`FusionIo`] to write the output back out as a string.
///
/// [CheerpJ Classloader]: https://cheerpj.com/docs/reference/CJ3Library
/// [SandboxBuilder]: https://docs.ion-fusion.dev/latest/javadoc/dev/ionfusion/fusion/SandboxBuilder.html
/// [TopLevel]: https://docs.ion-fusion.dev/latest/javadoc/dev/ionfusion/fusion/TopLevel.html
/// [FusionIo]: https://docs.ion-fusion.dev/latest/javadoc/dev/ionfusion/fusion/FusionIo.html
pub async fn fusion_eval(
    lib: &Classloader,
    runtime: &Runtime,
    expr: String,
) -> Result<String, String> {
    js_fusion_eval(lib, runtime, expr)
        .await
        .map(as_string_or_blank)
        .map_err(as_string_or_blank)
}
