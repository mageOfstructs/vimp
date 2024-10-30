use leptos::{
    component, create_signal,
    ev::{Event, KeyboardEvent},
    event_target_value, logging, provide_context, use_context, view, For, IntoView, ReadSignal,
    SignalUpdate, View, WriteSignal,
};
use regex::Regex;

use crate::parser::{
    Command, CommandFSM, CommandType, Coords, Direction, FinishedRelCoord, RelCoordPair,
};

#[derive(Clone)]
struct CursorSetter {
    x: ReadSignal<u32>,
    y: ReadSignal<u32>,
    setx: WriteSignal<u32>,
    sety: WriteSignal<u32>,
}

const REGEX: &str = "[lra]?[0-9]+[hjkl]?(;[0-9]+;)?";
#[component]
pub fn Canvas() -> impl IntoView {
    let (x, setx) = create_signal(50);
    let (y, sety) = create_signal(50);
    provide_context(CursorSetter { x, y, setx, sety });
    view! {
        <Reader/>
        <Cursor x={x} y={y}/>
    }
}

fn update_pos_relative(rcp: RelCoordPair, cs: &CursorSetter) {
    match rcp.1 {
        Direction::Up => cs.sety.update(|y| *y -= rcp.0),
        Direction::Down => cs.sety.update(|y| *y += rcp.0),
        Direction::Left => cs.setx.update(|x| *x -= rcp.0),
        Direction::Right => cs.setx.update(|x| *x += rcp.0),
    }
}

fn get_coords(coords: &Coords, cs: &CursorSetter) -> (u32, u32, u32, u32) {
    let x = (cs.x)();
    let y = (cs.y)();
    match coords {
        Coords::AbsCoord(x2, y2) => (x, y, *x2, *y2),
        Coords::RelCoord(fcp) => match fcp {
            FinishedRelCoord::OneCoord(rcp) => {
                let (x2, y2) = rcp.getCoords(x, y);
                (x, y, x2, y2)
            }
            FinishedRelCoord::TwoCoords(rcp, rcp2) => {
                let (x2, y2) = rcp.getCoords(x, y);
                let (x2, y2) = rcp2.getCoords(x2, y2);
                (x, y, x2, y2)
            }
        },
    }
}

fn parse_command(com: Command) {
    let cs = use_context::<CursorSetter>().unwrap();
    match com.ctype() {
        CommandType::Line => {}
        CommandType::Rectangle => {}
        CommandType::Move => match com.coords() {
            Coords::AbsCoord(x, y) => {
                (cs.setx)(x);
                (cs.sety)(y);
            }
            Coords::RelCoord(rc) => match rc {
                FinishedRelCoord::OneCoord(rcp) => update_pos_relative(rcp, &cs),
                FinishedRelCoord::TwoCoords(rcp1, rcp2) => {
                    update_pos_relative(rcp1, &cs);
                    update_pos_relative(rcp2, &cs)
                }
            },
        },
        CommandType::Text => todo!(),
    }
}

#[component]
fn Reader() -> impl IntoView {
    let (com, set_com) = create_signal(String::new());
    let (fsm, set_fsm) = create_signal(Option::<CommandFSM>::None);
    let (forms, set_forms) = create_signal(Vec::<View>::new());

    let on_keypress = move |evt: KeyboardEvent| {
        let mut next_char = evt.key();
        logging::log!("We got {next_char}!");
        if next_char == "Backspace" {
            set_com.update(|str| {
                str.pop();
            });
            set_fsm(match CommandFSM::from(com()) {
                Ok(fsm) => Some(fsm),
                Err(_) => None,
            });
        }
        if next_char == "Enter" {
            next_char = "\n".to_string();
        }
        if next_char.len() == 1 {
            let next_char = next_char.chars().next().unwrap();
            set_com.update(|str| str.push(next_char));
            match fsm() {
                Some(fsm) => match fsm.advance(next_char) {
                    Ok(com) => {
                        parse_command(com);
                        set_fsm(None);
                        set_com.update(|str| str.clear());
                    }
                    Err(new_fsm) => set_fsm(Some(new_fsm)),
                },
                None => set_fsm(Some(CommandFSM::new(next_char))),
            }
        }
    };

    view! {
        <p>Current command: {com}</p>
        <svg tabindex="1" autofocus on:keydown=on_keypress width="100%" height="100%" style="position: absolute">
            <For each=forms
                key=|el| {1}
                children= move |el| {
                    view! {el}
                }
            />
        </svg>
    }
}

#[component]
fn Cursor(x: ReadSignal<u32>, y: ReadSignal<u32>) -> impl IntoView {
    let style = move || {
        format!(
            "position: relative; top: {}%; left: {}%; color: red",
            y(),
            x()
        )
    };
    view! {
        <div style="width: 100%; height: 100%; z-index: 1; position: absolute">
            <div style={style}>
                UwU
            </div>
        </div>
    }
}

#[component]
fn Line(
    x1: ReadSignal<u32>,
    y1: ReadSignal<u32>,
    x2: ReadSignal<u32>,
    y2: ReadSignal<u32>,
) -> impl IntoView {
    view! {
        <line x1={x1} y1={y1} x2={x2} y2={y2}></line>
    }
}
