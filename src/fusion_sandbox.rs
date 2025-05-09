use std::rc::Rc;

use yew::{html, Component, Context, Html, Properties};

use crate::{
    code_input::CodeInput,
    fusion_runtime::{self, Classloader, Runtime},
};

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
                Err(js) => SandboxMsg::RuntimeError(js.as_string().unwrap_or_default()),
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
                        Err(js) => SandboxMsg::RuntimeError(js.as_string().unwrap_or_default()),
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
                    let result = fusion_runtime::fusion_eval(&classloader, &runtime, expr)
                        .await
                        .map(|output| output.as_string().unwrap_or_default())
                        .map_err(|e| e.as_string().unwrap_or_default());

                    SandboxMsg::FusionResult(result)
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
            let on_submit = ctx.link().callback(|code: String| SandboxMsg::Invoke(code));

            html! {
                <main>
                    <h1>{ "Fusion is ready!" }</h1>
                    <CodeInput {on_submit}/>
                    <br/>
                    {
                        if let Some(result) = &self.fusion_result {
                            let (message, content) = match result {
                                Ok(output) => ("Your result is: ", output),
                                Err(err) => ("Your expression produced an error: ", err),
                            };
                            html! {
                                <div>
                                    <span class="subtitle">{message}</span>
                                    <br/>
                                    <span class="monospaced">{content}</span>
                                </div>
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
