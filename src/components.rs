use leptos::{
    component, create_signal, ev::Event, event_target_value, logging, provide_context, use_context,
    view, IntoView, ReadSignal, SignalUpdate, WriteSignal,
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
    let (getter, setter) = create_signal(String::new());
    let parser_fn = move |evt: Event| {
        let com = event_target_value(&evt);
        let reg = Regex::new(REGEX).unwrap();
        if reg.is_match(&com) {
            logging::log!("We got a match!");
            let mut it = com.chars();
            let mut fsm = CommandFSM::new(it.next().expect("REGEX let empty string through"));
            logging::log!("FSM: {fsm:?}");
            for char in it {
                match fsm.advance(char) {
                    Ok(command) => {
                        logging::log!("{command:?}");
                        parse_command(command);
                        break;
                    }
                    Err(new_state) => fsm = new_state,
                }
                logging::log!("FSM: {fsm:?}");
            }
        }
        setter(com);
    };

    view! {
        <input prop:value=getter on:input=parser_fn/>
        {getter}
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
