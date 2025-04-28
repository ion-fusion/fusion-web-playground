mod app;
mod fusion_runtime;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
