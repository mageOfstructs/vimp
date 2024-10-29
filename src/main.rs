use leptos::mount_to_body;
use leptos::view;
use vimp::components::Canvas;

fn main() {
    mount_to_body(|| {
        view! {
            <Canvas/>
        }
    });
    println!("Hello, world!");
}
