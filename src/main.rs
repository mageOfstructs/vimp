use leptos::mount_to_body;
use leptos::view;
use vimp::components::Reader;

fn main() {
    mount_to_body(|| {
        view! {
            <Reader/>
        }
    });
    println!("Hello, world!");
}
