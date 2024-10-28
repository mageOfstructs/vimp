use leptos::mount_to_body;
use leptos::view;

fn main() {
    mount_to_body(|| view! {test});
    println!("Hello, world!");
}
