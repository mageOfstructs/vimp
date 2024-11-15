use leptos::mount_to_body;
use leptos::view;
use vimp::components::Canvas;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <Canvas/>
        }
    });
    println!("Hello, world!");
}
