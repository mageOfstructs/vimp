use crate::graphics::{Group, TrueSignalClone};
use js_sys::Array;
use leptos::ev::{self, MouseEvent};
use leptos::web_sys::{Blob, Url};
use leptos::Children;
use leptos::RwSignal;
use leptos::Show;
use leptos::Signal;
use leptos::SignalWith;
use leptos::{
    component, create_signal, ev::KeyboardEvent, logging, on_cleanup, provide_context, use_context,
    view, window, For, IntoView, ReadSignal, SignalUpdate, WriteSignal,
};
use leptos::{window_event_listener, SignalSet};
use std::cell::RefCell;
use std::hash::{DefaultHasher, Hasher};
use wasm_bindgen::JsValue;

use crate::graphics::Circle;
use crate::{
    graphics::{Form, GraphicsItem, Line, Rect, Text},
    parser::{Command, CommandFSM, CommandType, Coords, Direction, FSMResult, RelCoordPair},
};

// TOOD: refactor into separate files

#[derive(Clone)]
struct CursorSetter {
    x: ReadSignal<u32>,
    y: ReadSignal<u32>,
    setx: WriteSignal<u32>,
    sety: WriteSignal<u32>,
}

#[component]
pub fn Canvas() -> impl IntoView {
    let (x, setx) = create_signal(50);
    let (y, sety) = create_signal(50);
    provide_context(CursorSetter { x, y, setx, sety });
    view! {
        <Reader/>
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

    match coords {
        Coords::AbsCoord(x2, y2) => (x, y, *x2, *y2),
        Coords::RelCoord(fcp) => {
            let (x2, y2) = fcp.resolve_fcp();
            (x, y, x2, y2)
        }
    }
}

/// needs CursorSetter to be in context
pub fn get_cursor_pos() -> (u32, u32) {
    let cs = use_context::<CursorSetter>().expect("Will never read this anyways");
    ((cs.x)(), (cs.y)())
}

fn parse_command(
    com: Command,
    set_forms: WriteSignal<Vec<Form>>,
    set_overlays: WriteSignal<Vec<SelectableOverlayData>>,
) {
    let cs = use_context::<CursorSetter>().unwrap();
    let select_mode = use_context::<SelectMode>().unwrap();
    if let SelectState::FormsSelected = select_mode() {
        let buf = use_context::<SelectBuffer>().unwrap().0();
        match com.ctype() {
            CommandType::Move => {
                for form in buf {
                    form.1.move_form(&com.coords());
                }
            }
            _ => todo!(),
        }
    } else {
        let form = match com.ctype() {
            CommandType::Line => {
                logging::log!("Creating a line...");
                Some(Form::Line(Line::try_from(com).unwrap()))
            }
            CommandType::Rectangle => Some(Form::Rect(Rect::try_from(com).unwrap())),
            CommandType::Move => {
                let (x, y) = match com.coords() {
                    Coords::AbsCoord(x, y) => (x, y),
                    Coords::RelCoord(rc) => rc.resolve_fcp(),
                };
                (cs.setx)(x);
                (cs.sety)(y);
                logging::log!("New cursor pos: {}, {}", x, y);
                None
            }
            CommandType::Text => Some(Form::Text(Text::try_from(com).unwrap())),
            CommandType::Circle(_) => Some(Form::Circle(Circle::try_from(com).unwrap())),
        };
        if let Some(form) = form {
            set_overlays.update(|vec| vec.push(form.get_overlay_dims()));
            set_forms.update(|vec| vec.push(form));
        }
    }
}

#[derive(Clone)]
pub struct SelectMode(ReadSignal<SelectState>);

impl FnOnce<()> for SelectMode {
    type Output = SelectState;
    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        (self.0)()
    }
}
impl FnMut<()> for SelectMode {
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output {
        (self.0)()
    }
}
impl Fn<()> for SelectMode {
    extern "rust-call" fn call(&self, _args: ()) -> Self::Output {
        (self.0)()
    }
}

#[derive(Clone)]
struct SelectBuffer(ReadSignal<Vec<(usize, Form)>>);

#[derive(Clone, Debug, PartialEq)]
pub enum SelectState {
    Off,
    SelectModeOn,
    FormsSelected,
}

#[derive(Clone)]
pub struct FormsWS(pub WriteSignal<Vec<Form>>);

#[derive(Clone)]
pub struct OverlaysWS(pub WriteSignal<Vec<SelectableOverlayData>>);

#[derive(Clone)]
pub struct PreviewWS(pub WriteSignal<Option<Form>>);

#[component]
fn Reader() -> impl IntoView {
    let (com, set_com) = create_signal(String::new());
    let (fsm, set_fsm) = create_signal(Option::<CommandFSM>::None);
    let (forms, set_forms) = create_signal(Vec::<Form>::new());
    let (limbo, set_limbo) = create_signal(Option::<Form>::None);
    let (select_buffer, set_select_buffer) = create_signal(Vec::<(usize, Form)>::new());
    let (select_mode, set_select_mode) = create_signal(SelectState::Off);
    let (overlays, set_overlays) = create_signal(Vec::<SelectableOverlayData>::new());
    let (preview, set_preview) = create_signal(Option::<Form>::None);
    provide_context(overlays);
    provide_context(PreviewWS(set_preview));
    provide_context(SelectMode(select_mode));
    provide_context(SelectBuffer(select_buffer));

    let last_idx: RefCell<Option<usize>> = RefCell::new(None);
    let last_len: RefCell<Option<usize>> = RefCell::new(None);

    // FIXME: in urgent need of a refactor
    // TODO: in urgent need of a refactor
    // BUG: in urgent need of a refactor
    let on_keypress = move |evt: KeyboardEvent| {
        let mut next_char = evt.key();
        logging::log!("We got {next_char}!");
        logging::log!("Select mode: {:?}", select_mode());
        match select_mode() {
            SelectState::SelectModeOn => {
                if next_char.len() == 1 {
                    set_com.update(|com| com.push_str(&next_char));
                } else if next_char == "Backspace" && !com().is_empty() {
                    set_com.update(|com| {
                        com.pop();
                    });
                }

                let idxs: Vec<_> = com()
                    .split(',')
                    .filter(|str| !str.is_empty())
                    .map(|str| Namer::get_index(str))
                    .collect();

                let last_len_ref = *last_len.borrow();
                if let None = last_len_ref {
                    *last_len.borrow_mut() = Some(idxs.len());
                }
                logging::log!("idxs: {idxs:?}");

                let last_idx_ref = *last_idx.borrow();
                let new_last = if com().is_empty() {
                    None
                } else {
                    idxs.last().copied()
                };

                logging::log!("last: {last_idx_ref:?}, new_last: {new_last:?}");
                if new_last != last_idx_ref {
                    if let Some(len) = last_len_ref
                        && len >= idxs.len()
                        && let Some(i) = last_idx_ref
                    {
                        overlays.with(|vec| {
                            vec[i].set_selected(false);
                        });
                    }
                    if let Some(new_last) = new_last
                        && new_last < forms().len()
                    {
                        overlays.with(|vec| {
                            vec[new_last].set_selected(true);
                        });
                    }
                    *last_idx.borrow_mut() = new_last;
                } else if last_idx_ref.is_none() {
                    for i in &idxs {
                        let i = *i;
                        logging::log!("This should be index '{}'", i);
                        if i >= forms().len() || i >= overlays().len() {
                            logging::error!("But this index is out of bounce!");
                        } else {
                            logging::log!("Updating the selected prop");
                            overlays.with(|vec| vec[i].selected.set(true));
                            logging::log!("Successfully updated the selected prop(s)");
                        }
                    }
                }

                if next_char == "Enter" {
                    logging::log!("We got da '{}'", com());
                    idxs.iter().for_each(|i| {
                        set_select_buffer.update(|buf| buf.push((*i, forms()[*i].clone())));
                    });
                    set_select_mode(SelectState::FormsSelected);
                    set_com.update(|str| str.clear());
                }
                return;
            }
            SelectState::FormsSelected => match &*next_char {
                "d" | "y" => {
                    if next_char == "d" {
                        for form in select_buffer() {
                            set_overlays.update(|vec| {
                                vec.remove(form.0);
                            });
                            set_forms.update(|vec| {
                                vec.remove(form.0);
                            });
                        }
                        set_select_buffer.update(|vec| vec.clear());
                    }
                    logging::log!("made it here");
                    set_select_mode(SelectState::Off);
                    set_overlays.update(|vec| {
                        select_buffer()
                            .iter()
                            .map(|el| el.0)
                            .for_each(|i| vec[i].selected.set(false));
                    });
                    set_com.update(|str| str.clear());
                    return;
                }
                "g" => {
                    set_forms.update(|vec| {
                        let group =
                            Group::from_iter(select_buffer().iter().map(|tuple| tuple.1.clone()));
                        set_overlays.update(|vec| vec.push(group.get_overlay_dims()));
                        vec.push(Form::Group(group));
                    });
                    clear_select(
                        set_com,
                        set_fsm,
                        set_select_mode,
                        set_overlays,
                        select_buffer,
                        set_select_buffer,
                    );
                    return;
                }
                _ => {}
            },
            _ => {}
        }
        match &*next_char {
            "Escape" => {
                clear_select(
                    set_com,
                    set_fsm,
                    set_select_mode,
                    set_overlays,
                    select_buffer,
                    set_select_buffer,
                );
            }
            "e" if fsm().is_none() => {
                set_select_mode(SelectState::SelectModeOn);
                return;
            }
            "p" => {
                set_select_buffer.update(|select_buffer| {
                    for form in select_buffer.iter().map(|(_, form)| form) {
                        if let Form::Group(_) = form {
                            provide_context(FormsWS(set_forms));
                            provide_context(OverlaysWS(set_overlays));
                        }
                        let form = form.deep_clone();
                        let (x, y) = get_cursor_pos();
                        form.move_form(&Coords::AbsCoord(x, y));
                        set_overlays.update(|vec| vec.push(form.get_overlay_dims()));
                        set_forms.update(|vec| {
                            vec.push(form);
                        });
                    }
                });
                clear_select(
                    set_com,
                    set_fsm,
                    set_select_mode,
                    set_overlays,
                    select_buffer,
                    set_select_buffer,
                );
                return;
            }
            "Backspace" if !com().is_empty() => {
                set_com.update(|str| {
                    str.pop();
                });
                set_fsm(match CommandFSM::from(com()) {
                    FSMResult::OkCommand(com) => {
                        // this is technically unreachable
                        parse_command(com, set_forms, set_overlays);
                        return;
                    }
                    FSMResult::OkFSM(fsm) => Some(fsm),
                    FSMResult::Err(_) => None,
                });
                update_preview(&fsm);
                return;
            }
            "u" if fsm().is_none() => {
                set_forms.update(|vec| {
                    set_limbo(vec.pop());
                });
                set_overlays.update(|vec| {
                    vec.pop();
                });
                return;
            }
            "U" if fsm().is_none() => {
                set_forms.update(|vec| {
                    match limbo() {
                        Some(form) => {
                            set_overlays.update(|vec| vec.push(form.get_overlay_dims()));
                            vec.push(form)
                        }
                        None => logging::warn!("The void cannot be shaped"),
                    };
                });
                return;
            }
            "Enter" => next_char = "\n".to_string(),

            _ => {}
        }

        if next_char.len() == 1 {
            let next_char = next_char.chars().next().unwrap();
            set_com.update(|str| str.push(next_char));
            match fsm() {
                Some(fsm) => match fsm.advance(next_char) {
                    Ok(com) => {
                        parse_command(com, set_forms, set_overlays);
                        logging::log!("Finished Command parsing");
                        set_fsm(None);
                        logging::log!("Updated State 1");
                        set_com.update(|str| str.clear());
                        logging::log!("Updated State 2");
                    }
                    Err(new_fsm) => set_fsm(Some(new_fsm)),
                },
                None => {
                    set_fsm(Some(match CommandFSM::new(next_char) {
                        Ok(fsm) => fsm,
                        Err(err) => {
                            logging::error!("Couldn't create CommandFSM, because this stoopid char snuck in: {err}");
                            return;
                        }
                    }))
                }
            }
            update_preview(&fsm);
        }
    };

    let handle = window_event_listener(ev::keydown, on_keypress);
    on_cleanup(move || handle.remove());

    view! {
        <ExportBtn/>
        <div class="box">
            <p>Current command: {com}</p>
            <div class="container">
            <svg id="svg_canvas" style="width: 100%; height: 100%; position: absolute">
                {move ||
                    if let Some(form) = preview() {
                        form.into_view()
                    } else {
                        view! {}.into_view()
                    }
                }
                <For each=forms
                    key=|el| {el.key()}
                    children= move |el| {
                        view! {{el.into_view()}}
                    }
                />
            </svg>
            <Cursor/>
            </div>
        </div>
    }
}

fn update_preview(fsm: &ReadSignal<Option<CommandFSM>>) {
    let set_preview = use_context::<PreviewWS>().unwrap().0;
    if let Some(fsm) = fsm() {
        set_preview(match fsm.try_into() {
            Ok(form) => Some(form),
            Err(_) => None,
        });
    }
}

#[component]
fn ExportBtn() -> impl IntoView {
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
            None => panic!("BUG: svg canvas has wrong id!"),
        }
        .inner_html();
        let svg = format!(
            "<?xml version=\"1.0\" standalone=\"no\"?>
<svg height=\"100%\" width=\"100%\" version=\"1.1\"
     xmlns=\"http://www.w3.org/2000/svg\">
            {}
</svg>
",
            svg
        );
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
    view! {
        <div style="position: absolute; top: 0%; right: 10%" min-width="20%">
            <button on:click={export}>Export</button>
            {move || {
                if let Some(url) = download_link() {
                    view! {
                        <a href={url} download="image.svg">Click to Download</a>
                    }.into_view()
                } else {view! { }.into_view()}
            }}
        </div>
    }
}

fn mouseclick(evt: MouseEvent) {
    if evt.button() == 0 {
        logging::log!("Mouse click: {evt:?}");
    }
}

#[derive(Clone, Debug)]
pub struct SelectableOverlayData {
    top: Signal<u32>,
    left: Signal<u32>,
    end_x: Signal<u32>,
    end_y: Signal<u32>,
    selected: RwSignal<bool>,
}

impl IntoView for SelectableOverlayData {
    fn into_view(self) -> leptos::View {
        logging::log!("auf bessere Zeiten warten");
        let namer = use_context::<RwSignal<Namer>>().unwrap();
        let name = namer().next_name();
        namer.update(|namer| namer.inc());
        view! {
            <SelectableOverlay top={self.top} left={self.left} end_x={self.end_x} end_y={self.end_y} selected={self.selected.read_only()} name={name}/>
        }
    }
}

impl SelectableOverlayData {
    pub fn new(
        top: Signal<u32>,
        left: Signal<u32>,
        end_x: Signal<u32>,
        end_y: Signal<u32>,
    ) -> Self {
        Self {
            top,
            left,
            end_x,
            end_y,
            selected: RwSignal::new(false),
        }
    }
    fn key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write_u32(self.top());
        hasher.write_u32(self.left());
        hasher.write_u32(self.end_x());
        hasher.write_u32(self.end_y());
        hasher.finish()
    }

    pub fn top(&self) -> u32 {
        (self.top)()
    }
    pub fn left(&self) -> u32 {
        (self.left)()
    }
    pub fn end_x(&self) -> u32 {
        (self.end_x)()
    }
    pub fn end_y(&self) -> u32 {
        (self.end_y)()
    }
    pub fn set_selected(&self, selected: bool) {
        self.selected.set(selected);
    }
}

#[component]
fn SelectableOverlay(
    top: Signal<u32>,
    left: Signal<u32>,
    end_x: Signal<u32>,
    end_y: Signal<u32>,
    selected: ReadSignal<bool>,
    name: String,
) -> impl IntoView {
    let style = move || {
        format!(
            "position: absolute; top: {}%; left: {}%; min-width: 5%; min-height: 5%; border: 2px inset; border-radius: 10px; font-size: 1em; text-align: center",
            top() + ((end_y().checked_sub(top()).unwrap_or(0))/2) as u32,
            left() + ((end_x().checked_sub(left()).unwrap_or(0))/2) as u32,
        )
    };
    let class = move || format!("selectable {}", if selected() { "selected" } else { "" });
    view! {
        <div class={class} style={style}>{name}</div>
    }
}

#[derive(Clone)]
pub struct Namer {
    cur_name: Vec<char>,
}
impl Namer {
    pub fn get_index(name: &str) -> usize {
        name.chars()
            .enumerate()
            .map(|(i, c)| (c as usize - 'a' as usize) * 26_usize.pow(i as u32))
            .sum()
    }
    pub fn new() -> Self {
        Self {
            cur_name: vec!['a'],
        }
    }
    // TODO: refactor into separate file, having internal methods accessible from the outside feels
    // wrong
    fn inc_internal(&mut self, idx: usize) {
        if self.cur_name.len() > idx {
            self.cur_name[idx] = (self.cur_name[idx] as u8 + 1) as char;
        } else {
            self.cur_name.push('a');
        }
        if self.cur_name[idx] as u8 == 123 {
            self.cur_name[idx] = 'a';
            self.inc_internal(idx + 1);
        }
    }

    fn inc(&mut self) {
        self.inc_internal(0);
    }
    pub fn next_name(&mut self) -> String {
        let mut ret = String::with_capacity(self.cur_name.len());
        // i know, there is an iterator for that
        for i in 0..self.cur_name.len() {
            ret.push(self.cur_name[i])
        }
        logging::log!("Namer: returning {ret}!");
        self.inc();
        ret
    }

    pub fn clear(&mut self) {
        self.cur_name.clear();
        self.cur_name.push('a');
    }
}

#[component]
fn Cursor() -> impl IntoView {
    let cs = use_context::<CursorSetter>().unwrap();
    let (x, y) = (cs.x, cs.y);
    let style = move || {
        format!(
            "position: absolute; top: {}%; left: {}%; color: red; display: inline; z-index: 2",
            y(),
            x()
        )
    };
    let select_mode = use_context::<SelectMode>().unwrap();
    let overlays = use_context::<ReadSignal<Vec<SelectableOverlayData>>>().unwrap();
    let namer = RwSignal::new(Namer::new());
    provide_context(namer);

    view! {
        <div id="overlay" on:mousedown={mouseclick} style="width: 100%; height: 100%; z-index: 1; position: absolute; box-sizing: border-box"> //  padding-right: 5%; padding-bottom: 2%;
        <Show
            when=move || select_mode() == SelectState::Off
            fallback=move || {
                view! {
                    {overlays}
                }
            }
        >
            {move || {
                namer.update(|namer| namer.clear()); // not the most
                // efficient place to put this in
                view! {}.into_view()
            }}
        </Show>
        <div style={style}>
            UwU
        </div>
        </div>
    }
}

#[component]
pub fn Selectable(edit: ReadSignal<bool>, children: Children) -> impl IntoView {
    view! {
        <div style="display: inline; margin: 0" class={if edit() {"selectable"} else {""}}>
            {children()}
        </div>
    }
}

fn clear_select(
    set_com: WriteSignal<String>,
    set_fsm: WriteSignal<Option<CommandFSM>>,
    set_select_mode: WriteSignal<SelectState>,
    set_overlays: WriteSignal<Vec<SelectableOverlayData>>,
    select_buffer: ReadSignal<Vec<(usize, Form)>>,
    set_select_buffer: WriteSignal<Vec<(usize, Form)>>,
) {
    set_com.update(|str| str.clear());
    set_fsm(None);
    set_select_mode(SelectState::Off);
    set_overlays.update(|vec| {
        select_buffer()
            .iter()
            .map(|el| el.0)
            .for_each(|i| vec[i].selected.set(false));
    });
    set_select_buffer.update(|vec| vec.clear());
}
