use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, MouseEvent, Properties, TargetCast};

use crate::fusion_runtime::{self, Classloader, Runtime};

/// The component's internal state is maintained by values here.
#[derive(Default)]
pub struct FusionSandbox {
    classloader: Option<Rc<Classloader>>,
    runtime: Option<Rc<Runtime>>,
    fusion_result: Option<Result<String, String>>,
    runtime_error: Option<String>,
}

/// The messages that our sandbox can react to. These are what represent changes in state for the component.
pub enum SandboxMsg {
    ClassloaderReady(Classloader),
    RuntimeReady(Runtime),
    Invoke(String),
    FusionResult(Result<String, String>),
    RuntimeError(String),
}

/// These would be our HTML properties that would define some behavior if we wanted any on this component.
/// If we eventually want to support running the sandbox at an arbitrary Fusion/Ion version, this would probably
/// be where those values would live.
#[derive(Default, PartialEq, Properties)]
pub struct EmptyProps;

impl Component for FusionSandbox {
    type Message = SandboxMsg;

    type Properties = EmptyProps;

    /// Called when an instance of this component is added to the DOM.
    fn create(ctx: &Context<Self>) -> Self {
        // On create, spawn the initial process to initialize the CheerpJ runtime and create a classloader
        ctx.link().send_future(async {
            match fusion_runtime::create_classloader().await {
                Ok(classloader) => SandboxMsg::ClassloaderReady(classloader),
                Err(err) => SandboxMsg::RuntimeError(err),
            }
        });

        FusionSandbox::default()
    }

    /// Called every time this component recieves a message for some state update.
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SandboxMsg::ClassloaderReady(classloader) => {
                let refcounted = Rc::new(classloader);
                self.classloader = Some(refcounted.clone());
                ctx.link().send_future(async move {
                    match fusion_runtime::create_fusion_runtime(&refcounted).await {
                        Ok(runtime) => SandboxMsg::RuntimeReady(runtime),
                        Err(err) => SandboxMsg::RuntimeError(err),
                    }
                });
                false // No need to re-render at this point
            }
            SandboxMsg::RuntimeReady(runtime) => {
                self.runtime = Some(Rc::new(runtime));
                true // Render the input now that we're ready
            }
            SandboxMsg::Invoke(expr) => {
                let classloader = self.classloader.clone().expect("A classloader must exist");
                let runtime = self.runtime.clone().expect("A runtime must exist");
                ctx.link().send_future(async move {
                    SandboxMsg::FusionResult(
                        fusion_runtime::fusion_eval(&classloader, &runtime, expr).await,
                    )
                });
                false // No spinner or anything for waiting on evaluation for now, but if we had one this would be true
            }
            SandboxMsg::FusionResult(result) => {
                self.fusion_result = Some(result);
                true // Re-render our new result
            }
            SandboxMsg::RuntimeError(err) => {
                self.runtime_error = Some(err);
                true // Re-render our new error
            }
        }
    }

    /// Called on initial `create` and any time `update` returns true for re-rendering
    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(error) = &self.runtime_error {
            log::error!("{error}");
            html! {
                <main>
                    <h1>{ "Unexpected Runtime Error" }</h1>
                    <span class="subtitle">{ error }</span>
                </main>
            }
        } else if self.runtime.is_some() {
            let onclick = ctx.link().callback(|event: MouseEvent| {
                let input: HtmlInputElement = event.target_unchecked_into();

                input
                    .owner_document()
                    .and_then(|document| document.get_element_by_id("fusion-script"))
                    .and_then(|element| {
                        element
                            .dyn_into::<HtmlInputElement>()
                            .map(|text_input| text_input.value())
                            .ok()
                    })
                    .map_or(
                        SandboxMsg::RuntimeError("Could not parse input".to_owned()),
                        SandboxMsg::Invoke,
                    )
            });

            html! {
                <main>
                    <h1>{ "Fusion is ready!" }</h1>
                    <input id="fusion-script"/>
                    <button {onclick}>{ "Evaluate" }</button>
                    {
                        if let Some(result) = &self.fusion_result {
                            match result {
                                Ok(output) => html! {
                                        <span class="subtitle">{ "Your result is: " }{ output }</span>
                                    },
                                    Err(err) => html! {
                                        <span class="subtitle">{ "Your expression produced an error: "}{ err }</span>
                                    },
                                }
                        } else {
                            html! {
                                <span/>
                            }
                        }
                    }
                </main>
            }
        } else {
            html! {
                <main>
                    <h1>{ "Fusion is loading..." }</h1>
                </main>
            }
        }
    }
}
