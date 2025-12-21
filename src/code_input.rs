use log::error;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement, HtmlTextAreaElement};
use yew::{
    function_component, html, use_effect_with, use_state_eq, Callback, Html, InputEvent,
    MouseEvent, Properties, TargetCast, UseStateHandle,
};

const INPUT_ID: &str = "code-input-content";
const LINE_NUM_ID: &str = "line-numbers";
const INPUT_EVENT_TYPE: &str = "input";

#[derive(Properties, PartialEq)]
pub struct CodeInputProps {
    pub on_submit: Callback<String>,
}

#[derive(Debug, PartialEq)]
enum LineNumber {
    #[allow(dead_code)] // TODO: handle line wrapping
    Blank,
    Present(usize),
}

/// Helper function to run in `use_effect_with` to extract the styles of the
/// textarea and apply them to the line numbers.
fn match_text_style<T>(_: &T) {
    let window = window().expect("window should exist post-rendering");
    let document = window
        .document()
        .expect("document should exist post-rendering");

    let textarea = document
        .get_element_by_id(INPUT_ID)
        .expect("the input text area should be in the document");
    let line_numbers: HtmlElement = document
        .get_element_by_id(LINE_NUM_ID)
        .and_then(|element| element.dyn_into().ok())
        .expect("the line numbers element should be in the document");

    let style = window
        .get_computed_style(&textarea)
        .ok()
        .flatten()
        .expect("the text area should be styled");

    for prop in [
        "font-family",
        "font-size",
        "font-weight",
        "letter-spacing",
        "line-height",
        "padding",
    ] {
        line_numbers
            .style()
            .set_property(prop, &style.get_property_value(prop).unwrap_or_default())
            .expect("style should not be read-only for the line numbers");
    }
}

fn line_numbers_callback(new_lines: UseStateHandle<Vec<LineNumber>>) -> Callback<InputEvent> {
    Callback::from(move |event: InputEvent| {
        let element: HtmlTextAreaElement = event.target_unchecked_into();

        let mut line_nums = vec![];
        for (idx, _line) in element.value().lines().enumerate() {
            line_nums.push(LineNumber::Present(idx + 1));
            // TODO: determine how many wrapped lines are present and append
            // blank line numbers for them
        }
        // `lines` doesn't produce a value for a trailing newline, but the code
        // editor should show a line number in that case since the cursor will be
        // on the new line at that point
        if element.value().ends_with('\n') || element.value().is_empty() {
            line_nums.push(LineNumber::Present(line_nums.len() + 1));
        }

        new_lines.set(line_nums);
    })
}

#[function_component]
pub fn CodeInput(props: &CodeInputProps) -> Html {
    let onclick_eval = &props.on_submit.reform(|event: MouseEvent| {
        let element: HtmlElement = event.target_unchecked_into();
        let code = element
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

    // Passing an empty tuple to `use_effect_with` makes it run only on first render
    use_effect_with((), match_text_style);

    let lines = use_state_eq(|| vec![LineNumber::Present(1)]);
    let oninput = line_numbers_callback(lines.clone());

    let fmt_config = fuusak::config::new_default_config();
    let onclick_fmt = Callback::from(move |event: MouseEvent| {
        let element: HtmlElement = event.target_unchecked_into();
        let code = element
            .owner_document()
            .and_then(|document| document.get_element_by_id(INPUT_ID))
            .and_then(|element| element.dyn_into::<HtmlTextAreaElement>().ok());

        let Some(original) = code else {
            error!("Failed to get input from {INPUT_ID} in the DOM");
            return;
        };

        let formatted = match fuusak::parser::parse_str(&original.value(), &fmt_config) {
            Ok(ast) => fuusak::format::format(&fmt_config, &ast),
            Err(parse_err) => {
                // TODO: this should be surfaced to the user not in the console
                error!("Failed to parse code for formatting: {parse_err}");
                original.value()
            }
        };
        original.set_value(&formatted);

        // Emit an input event to trigger line number recomputation
        if InputEvent::new(INPUT_EVENT_TYPE)
            .and_then(|event| original.dispatch_event(&event))
            .is_err()
        {
            error!("Unable to dispatch input event after formatting");
        }
    });

    html! {
        <div>
            <div class="code-container">
                <div class="code-line-numbers" id={LINE_NUM_ID}>
                {
                    lines.iter().map(|line| html! {
                        <div>
                        {
                            match line {
                                LineNumber::Blank => String::new(),
                                LineNumber::Present(num) => num.to_string(),
                            }
                        }
                        </div>
                    }).collect::<Vec<Html>>()
                }
                </div>
                <textarea {oninput} class="code-lines" id={INPUT_ID}
                    rows=20
                    cols=80
                    wrap="off" // TODO: handle line wrapping
                    autocapitalize="off"
                    autocomplete="off"
                    autocorrect="off"
                    spellcheck="false"/>
                <br/>
            </div>
            <button onclick={onclick_eval}>{ "Evaluate" }</button>
            <button onclick={onclick_fmt}>{ "Format" }</button>
        </div>
    }
}
