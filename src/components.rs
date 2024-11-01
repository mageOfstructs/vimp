use js_sys::Array;
use leptos::web_sys::{Blob, Document, Url};
use leptos::{
    component, create_signal, ev::KeyboardEvent, logging, provide_context, use_context, view,
    window, For, IntoView, ReadSignal, SignalUpdate, View, WriteSignal,
};
use regex::Regex;
use wasm_bindgen::JsValue;

use crate::{
    graphics::{Form, GraphicsItem, Line, Rect, Text},
    parser::{
        Command, CommandFSM, CommandType, Coords, Direction, FSMResult, FinishedRelCoord,
        RelCoordPair,
    },
};

#[derive(Clone)]
struct CursorSetter {
    x: ReadSignal<u32>,
    y: ReadSignal<u32>,
    setx: WriteSignal<u32>,
    sety: WriteSignal<u32>,
}

const REGEX: &str = "[lra]?[0-9]*[hjkl]?(;[0-9]*[hjkl]?;)?";
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

fn calc_coords(coords: &Coords, cs: &CursorSetter) -> (u32, u32, u32, u32) {
    let x = (cs.x)();
    let y = (cs.y)();
    logging::log!("Got cursor pos...");
    match coords {
        Coords::AbsCoord(x2, y2) => (x, y, *x2, *y2),
        Coords::RelCoord(fcp) => match fcp {
            FinishedRelCoord::OneCoord(rcp) => {
                let (x2, y2) = rcp.get_coords(x, y);
                (x, y, x2, y2)
            }
            FinishedRelCoord::TwoCoords(rcp, rcp2) => {
                let (x2, y2) = rcp.get_coords(x, y);
                let (x2, y2) = rcp2.get_coords(x2, y2);
                (x, y, x2, y2)
            }
        },
    }
}

/// needs CursorSetter to be in context
pub fn get_cursor_pos() -> (u32, u32) {
    let cs = use_context::<CursorSetter>().expect("Will never read this anyways");
    ((cs.x)(), (cs.y)())
}

fn parse_command(com: Command, set_forms: WriteSignal<Vec<Form>>) {
    let cs = use_context::<CursorSetter>().unwrap();
    let com = com.clone();
    match com.ctype() {
        CommandType::Line => {
            logging::log!("Creating a line...");
            set_forms.update(|vec| {
                let line = Line::from(calc_coords(&com.coords(), &cs));
                logging::log!("Created a line: {:?}", line.clone().into_view());
                vec.push(Form::Line(line));
                logging::log!("Updated vec");
            });
        }
        CommandType::Rectangle => {
            set_forms
                .update(|vec| vec.push(Form::Rect(Rect::from(calc_coords(&com.coords(), &cs)))));
        }
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
        CommandType::Text => {
            set_forms.update(|vec| vec.push(Form::Text(Text::from(com).unwrap())));
        }
    }
}

#[component]
fn Reader() -> impl IntoView {
    let (com, set_com) = create_signal(String::new());
    let (fsm, set_fsm) = create_signal(Option::<CommandFSM>::None);
    let (forms, set_forms) = create_signal(Vec::<Form>::new());
    let (limbo, set_limbo) = create_signal(Option::<Form>::None);

    let (download_link, set_download_link) = create_signal(Option::<String>::None);
    let export = move |_| {
        let doc = match window().document() {
            Some(doc) => doc,
            None => {
                logging::error!("Document property not found (this is a major invalid state)!");
                panic!("Document no work");
            }
        };
        let svg = match doc.get_element_by_id("svg_canvas") {
            Some(el) => el,
            None => {
                logging::error!("BUG: svg canvas has wrong id!");
                panic!();
            }
        }
        .inner_html();
        logging::log!("Svg Data: {svg}");
        let blob_parts = Array::new_with_length(1);
        blob_parts.set(0, JsValue::from_str(&svg));
        let blob = match Blob::new_with_str_sequence(&blob_parts) {
            Ok(blob) => blob,
            Err(err) => {
                logging::error!("Failed to create blob: {err:?}");
                panic!()
            }
        };
        match Url::create_object_url_with_blob(&blob) {
            Ok(url) => set_download_link(Some(url)),
            Err(err) => {
                logging::error!("Failed to create URL: {err:?}")
            }
        }
    };

    let on_keypress = move |evt: KeyboardEvent| {
        let mut next_char = evt.key();
        logging::log!("We got {next_char}!");
        if next_char == "Backspace" {
            set_com.update(|str| {
                str.pop();
            });
            set_fsm(match CommandFSM::from(com()) {
                FSMResult::OkCommand(com) => {
                    parse_command(com, set_forms);
                    return;
                }
                FSMResult::OkFSM(fsm) => Some(fsm),
                FSMResult::Err(_) => None,
            });
        }
        if next_char == "Enter" {
            next_char = "\n".to_string();
        } else if next_char == "r" {
            set_forms.update(|vec| {
                set_limbo(vec.pop());
            });
            return;
        } else if next_char == "r" && evt.ctrl_key() {
            set_forms.update(|vec| {
                match limbo() {
                    Some(form) => vec.push(form),
                    None => logging::warn!("The void cannot be shaped"),
                };
            });
            return;
        }

        if next_char.len() == 1 {
            let next_char = next_char.chars().next().unwrap();
            set_com.update(|str| str.push(next_char));
            if Regex::new(REGEX).unwrap().is_match(&com()) {
                match fsm() {
                    Some(fsm) => match fsm.advance(next_char) {
                        Ok(com) => {
                            parse_command(com, set_forms);
                            logging::log!("Finished Command parsing");
                            set_fsm(None);
                            logging::log!("Updated State 1");
                            set_com.update(|str| str.clear());
                            logging::log!("Updated State 2");
                        }
                        Err(new_fsm) => set_fsm(Some(new_fsm)),
                    },
                    None => set_fsm(Some(match CommandFSM::new(next_char) {
                        Ok(fsm) => fsm,
                        Err(err) => {
                            logging::error!("Couldn't create CommandFSM, because this stoopid char snuck in: {err}");
                            return;
                        }
                    })),
                }
            } else {
                set_com.update(|str| {
                    str.pop();
                })
            };
        }
    };

    view! {
        <p>Current command: {com}</p>
        <button on:click={export}>Export</button>
        {move || {
            if let Some(url) = download_link() {
                view! {
                    <a href={url} download="image.svg">Click to Download</a>
                }.into_view()
            } else {view! { }.into_view()}
        }}
        <svg id="svg_canvas" tabindex="1" autofocus on:keydown=on_keypress width="100%" height="100%" style="position: absolute">
            <For each=forms
                key=|el| {el.key()}
                children= move |el| {
                    view! {{el.into_view()}}
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
