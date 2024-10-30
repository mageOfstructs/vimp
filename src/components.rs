use leptos::{
    component, create_signal,
    ev::{Event, KeyboardEvent},
    event_target_value, logging, provide_context, use_context, view, IntoView, ReadSignal,
    SignalUpdate, WriteSignal,
};
use regex::Regex;

use crate::parser::{
    Command, CommandFSM, CommandType, Coords, Direction, FinishedRelCoord, RelCoordPair,
};

#[derive(Clone)]
struct CursorSetter(WriteSignal<u32>, WriteSignal<u32>);

const REGEX: &str = "[lra]?[0-9]+[hjkl]?(;[0-9]+;)?";
#[component]
pub fn Canvas() -> impl IntoView {
    let (x, setx) = create_signal(50);
    let (y, sety) = create_signal(50);
    provide_context(CursorSetter(setx, sety));
    view! {
        <Reader/>
        <Cursor x={x} y={y}/>
    }
}

fn update_pos_relative(rcp: RelCoordPair, cs: &CursorSetter) {
    match rcp.1 {
        Direction::Up => cs.1.update(|y| *y -= rcp.0),
        Direction::Down => cs.1.update(|y| *y += rcp.0),
        Direction::Left => cs.0.update(|x| *x -= rcp.0),
        Direction::Right => cs.0.update(|x| *x += rcp.0),
    }
}
fn parse_command(com: Command) {
    match com.ctype() {
        CommandType::Line => {}
        CommandType::Rectangle => {}
        CommandType::Move => {
            let cs = use_context::<CursorSetter>().unwrap();
            match com.coords() {
                Coords::AbsCoord(x, y) => {
                    cs.0(x);
                    cs.1(y);
                }
                Coords::RelCoord(rc) => match rc {
                    FinishedRelCoord::OneCoord(rcp) => update_pos_relative(rcp, &cs),
                    FinishedRelCoord::TwoCoords(rcp1, rcp2) => {
                        update_pos_relative(rcp1, &cs);
                        update_pos_relative(rcp2, &cs)
                    }
                },
            }
        }
        CommandType::Text => todo!(),
    }
}

#[component]
fn Reader() -> impl IntoView {
    let (fsm, set_fsm) = create_signal(Option::<CommandFSM>::None);
    let on_keypress = move |evt: KeyboardEvent| {
        let next_char = evt.key();
        logging::log!("We got {next_char}!");
        if next_char.len() == 1 {
            let next_char = next_char.chars().next().unwrap();
            match fsm() {
                Some(fsm) => match fsm.advance(next_char) {
                    Ok(com) => {
                        parse_command(com);
                        set_fsm(None);
                    }
                    Err(new_fsm) => set_fsm(Some(new_fsm)),
                },
                None => set_fsm(Some(CommandFSM::new(next_char))),
            }
        }
    };
    view! {
        <p>Current command: {fsm}</p>
        <svg tabindex="1" autofocus on:keydown=on_keypress width="100%" height="100%">
        </svg>
        <br/>
    }
}

#[component]
fn Cursor(x: ReadSignal<u32>, y: ReadSignal<u32>) -> impl IntoView {
    let style = move || {
        format!(
            "position: absolute; top: {}%; left: {}%; min-width: 4vw; min-height: 4vh; color: red",
            y(),
            x()
        )
    };
    view! {
        <div style={style}>
            Here
        </div>
    }
}
