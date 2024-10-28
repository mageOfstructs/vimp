use leptos::{component, create_signal, ev::Event, event_target_value, view, IntoView};

use crate::parser::CommandFSM;

const REGEX: &str = "[lra]?[0-9]+[hjkl]?(;[0-9]+;)?";

#[component]
pub fn Reader() -> impl IntoView {
    let (getter, setter) = create_signal(String::new());
    let mut commandFSM: Option<CommandFSM> = None;
    let parser_fn = move |evt: Event| {
        let com = event_target_value(&evt);
        if let None = commandFSM {
            // commandFSM = Some(CommandFSM::new(c);
        }
    };

    view! {
        <input prop:value=getter on:input=parser_fn/>
        {getter}
    }
}
