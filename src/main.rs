mod fusion_runtime;
mod fusion_sandbox;

use fusion_sandbox::FusionSandbox;

fn main() {
    yew::Renderer::<FusionSandbox>::new().render();
}
