use log::error;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::{function_component, html, Callback, Html, MouseEvent, Properties, TargetCast};

const INPUT_ID: &str = "code-input-content";

#[derive(Properties, PartialEq)]
pub struct CodeInputProps {
    pub on_submit: Callback<String>,
}

#[function_component]
pub fn CodeInput(props: &CodeInputProps) -> Html {
    let onclick = &props.on_submit.reform(|event: MouseEvent| {
        let input: HtmlInputElement = event.target_unchecked_into();
        let code = input
            .owner_document()
            .and_then(|document| document.get_element_by_id(INPUT_ID))
            .and_then(|element| {
                element
                    .dyn_into::<HtmlTextAreaElement>()
                    .map(|text_input| text_input.value())
                    .ok()
            });

        if code.is_none() {
            error!("Failed to get input from {INPUT_ID} in the DOM");
        }

        code.unwrap_or_default()
    });

    html! {
        <div>
            <textarea id={INPUT_ID}
                rows=20
                cols=80
                autocapitalize="off"
                autocomplete="off"
                autocorrect="off"
                spellcheck="false"/>
            <br/>
            <button {onclick}>{ "Evaluate" }</button>
        </div>
    }
}
