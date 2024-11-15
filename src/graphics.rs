use crate::components::get_cursor_pos;
use crate::components::SelectMode;
use crate::components::Selectable;
use crate::components::SelectableOverlayData;
use leptos::use_context;
use leptos::ReadSignal;
use leptos::Signal;
use leptos::SignalUpdate;
use leptos::WriteSignal;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use leptos::logging;
use leptos::{view, RwSignal};
use leptos::{window, IntoView};

use crate::parser::Command;
use crate::parser::CommandType;

macro_rules! gen_form {
    ($($type:ident),+) => {
        #[derive(Clone, Debug)]
        pub enum Form {
            $($type($type)),+
        }

        impl GraphicsItem for Form {
            fn key(&self) -> u128 {
                match self {
                    $(Self::$type(form) => form.key()),+
                }
            }
            fn get_overlay_dims(&self) -> SelectableOverlayData {
                match self {
                    $(Self::$type(form) => form.get_overlay_dims()),+
                }
            }
        }

        impl IntoView for Form {
            fn into_view(self) -> leptos::View {
                match self {
                    $(Self::$type(form) => form.into_view()),+
                }
            }
        }
    };
}

pub fn key_from_four(n1: u32, n2: u32, n3: u32, n4: u32) -> u128 {
    ((n1 as u128) << 96u128) + ((n2 as u128) << 64u128) + ((n3 as u128) << 32u128) + n4 as u128
}
fn format_css<T: Display>(c: T) -> String {
    format!("{}%", c)
}
pub trait GraphicsItem: Clone {
    fn key(&self) -> u128;
    fn get_overlay_dims(&self) -> SelectableOverlayData;
}

gen_form!(Line, Rect, Text, Circle);

#[derive(Clone, Debug)]
pub struct Line {
    x1: RwSignal<u32>,
    y1: RwSignal<u32>,
    x2: RwSignal<u32>,
    y2: RwSignal<u32>,
    color: RwSignal<String>,
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Line: x1={}, y1={}, x2={}, y2={}",
            (self.x1.read_only())(),
            (self.y1.read_only())(),
            (self.x2.read_only())(),
            (self.y2.read_only())()
        )
    }
}

impl Line {
    pub fn from(pair: (u32, u32, u32, u32)) -> Self {
        Line {
            x1: RwSignal::new(pair.0),
            y1: RwSignal::new(pair.1),
            x2: RwSignal::new(pair.2),
            y2: RwSignal::new(pair.3),
            color: RwSignal::new("red".to_string()),
        }
    }

    fn css_coords_reactive(
        &self,
    ) -> (
        impl Fn() -> String,
        impl Fn() -> String,
        impl Fn() -> String,
        impl Fn() -> String,
    ) {
        let x1 = self.x1;
        let y1 = self.y1;
        let x2 = self.x2;
        let y2 = self.y2;
        (
            move || format_css(x1()),
            move || format_css((y1)()),
            move || format_css((x2)()),
            move || format_css((y2)()),
        )
    }
    fn css_coords(&self) -> (String, String, String, String) {
        (
            format_css((self.x1)()),
            format_css((self.y1)()),
            format_css((self.x2)()),
            format_css((self.y2)()),
        )
    }
}

impl GraphicsItem for Line {
    fn key(&self) -> u128 {
        logging::log!("called key() on Line");
        let ret = key_from_four(
            (self.x1.read_only())(),
            (self.y1.read_only())(),
            (self.x2.read_only())(),
            (self.y2.read_only())(),
        );
        logging::log!("finish keygen");
        ret
    }
    fn get_overlay_dims(&self) -> SelectableOverlayData {
        let mut x2 = self.x2.read_only();
        let mut x1 = self.x1.read_only();
        let mut y2 = self.y2.read_only();
        let mut y1 = self.y1.read_only();
        if y1() > y2() {
            y1 = y2;
            y2 = self.y1.read_only();
        }
        if x1() > x2() {
            x1 = x2;
            x2 = self.x1.read_only();
        }
        let width = Signal::derive(move || (x2)() - (x1)());
        let height = Signal::derive(move || (y2)() - (y1)());

        SelectableOverlayData::new(y1.into(), x1.into(), width, height)
    }
}

#[derive(Clone, Debug)]
pub struct Rect {
    x: RwSignal<u32>,
    y: RwSignal<u32>,
    width: RwSignal<u32>,
    height: RwSignal<u32>,
    rx: RwSignal<u32>,
    ry: RwSignal<u32>,
    border_color: RwSignal<String>,
    inner_color: RwSignal<String>,
}

impl Rect {
    // pub fn new(x: u32, y: u32, x2: u32, y2: u32) -> Self {
    //     Self {
    //         x: RwSignal::new(x),
    //         y: RwSignal::new(y),
    //         width: RwSignal::new(x2 as i32 - x as i32), // if this underflows, we're cooked
    //         height: RwSignal::new(y2 as i32 - y as i32), // if this underflows, we're cooked
    //         rx: RwSignal::new(Default::default()),
    //         ry: RwSignal::new(Default::default()),
    //         border_color: RwSignal::new(Default::default()),
    //         inner_color: RwSignal::new("red".to_string()),
    //     }
    // }
    // pub fn from(tuple: (u32, u32, u32, u32)) -> Self {
    //     logging::log!("Creating new rect with {tuple:?}");
    //     let (x, y, x2, y2) = tuple;
    //     Self::new(x, y, x2, y2)
    // }

    fn css_coords_reactive(
        &self,
    ) -> (
        impl Fn() -> String,
        impl Fn() -> String,
        impl Fn() -> String,
        impl Fn() -> String,
    ) {
        let mut x1: Signal<u32> = Signal::from(self.x);
        let mut y1: Signal<u32> = self.y.into();
        let mut width: Signal<u32> = self.width.into();
        let mut height: Signal<u32> = self.height.into();
        // if width() < 0 {
        //     x1 = Signal::derive(move || (x1() as i32 + width()) as u32);
        //     width = Signal::derive(move || -width());
        // }
        // if height() < 0 {
        //     y1 = Signal::derive(move || (y1() as i32 + height()) as u32);
        //     height = Signal::derive(move || -height());
        // }
        (
            move || format_css(x1()),
            move || format_css((y1)()),
            move || format_css((width)()),
            move || format_css((height)()),
        )
    }
}

impl GraphicsItem for Rect {
    fn key(&self) -> u128 {
        key_from_four(
            (self.x)(),
            (self.y)(),
            (self.width)() as u32,
            (self.height)() as u32,
        )
    }
    fn get_overlay_dims(&self) -> SelectableOverlayData {
        SelectableOverlayData::new(
            self.y.into(),
            self.x.into(),
            self.width.into(),
            self.height.into(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    x: RwSignal<u32>,
    y: RwSignal<u32>,
    text: RwSignal<String>,
    font_size: RwSignal<u32>,
    color: RwSignal<String>,
}

impl Text {
    fn css_coords_reactive(&self) -> (impl Fn() -> String, impl Fn() -> String) {
        let x = self.x;
        let y = self.y;
        (move || format_css((x)()), move || format_css((y)()))
    }
}

#[derive(Clone, Debug)]
pub struct Circle {
    radius: RwSignal<u32>,
    x: RwSignal<u32>,
    y: RwSignal<u32>,
    color: RwSignal<String>,
}

impl Circle {
    pub fn new(radius: u32, x: u32, y: u32) -> Self {
        Self {
            radius: RwSignal::new(radius),
            x: RwSignal::new(x),
            y: RwSignal::new(y),
            color: RwSignal::new("red".to_string()),
        }
    }
}

impl GraphicsItem for Circle {
    fn key(&self) -> u128 {
        (self.radius)() as u128
    }
    fn get_overlay_dims(&self) -> SelectableOverlayData {
        let x = self.x.read_only();
        let y = self.y.read_only();
        let radius = self.radius.read_only();

        let x = Signal::derive(move || x().checked_sub(radius()).unwrap_or(0));
        let y = Signal::derive(move || y().checked_sub(radius()).unwrap_or(0));
        SelectableOverlayData::new(
            x,
            y,
            Signal::derive(move || radius() * 2),
            Signal::derive(move || radius() * 2),
        )
    }
}

impl GraphicsItem for Text {
    fn key(&self) -> u128 {
        let mut hasher = DefaultHasher::new();
        ((((self.x)() as u128) << 32u128) + (self.y)() as u128).hash(&mut hasher);
        (self.text)().hash(&mut hasher);
        hasher.finish() as u128
    }
    fn get_overlay_dims(&self) -> SelectableOverlayData {
        let font_size = self.font_size.read_only();
        let text = self.text.read_only();
        SelectableOverlayData::new(
            self.x.into(),
            self.y.into(),
            Signal::derive(move || font_size() * text().len() as u32),
            self.font_size.into(),
        )
    }
}

const DEFAULT_STYLE: &str = ";stroke-width=2";

impl IntoView for Line {
    fn into_view(self) -> leptos::View {
        logging::log!("called into_view() on Line");
        let (x1, y1, x2, y2) = self.css_coords_reactive();
        let style = move || format!("stroke:{}{}", (self.color)(), DEFAULT_STYLE);
        view! {
            <line x1={x1} y1={y1} x2={x2} y2={y2} style={style}/>
        }
        .into_view()
    }
}

impl IntoView for Rect {
    fn into_view(self) -> leptos::View {
        let (x, y, width, height) = self.css_coords_reactive();
        logging::log!(
            "Rendering new rect with {},{},{},{}",
            x(),
            y(),
            width(),
            height()
        );
        view! {
            <rect x={x} y={y} rx={self.rx} ry={self.ry} width={width} height={height} fill={self.inner_color}/>
        }
        .into_view()
    }
}

impl IntoView for Text {
    fn into_view(self) -> leptos::View {
        let (x, y) = self.css_coords_reactive();
        view! {
            <text x={x} y={y} fill={self.color}>{self.text}</text>
        }
        .into_view()
    }
}

impl IntoView for Circle {
    fn into_view(self) -> leptos::View {
        view! {
            <circle r={move || format_css((self.radius)())} cx={move || format_css((self.x)())} cy={move || format_css((self.y)())} fill={self.color}/>
        }
        .into_view()
    }
}

impl TryFrom<Command> for Line {
    type Error = CommandType;
    fn try_from(value: Command) -> Result<Self, Self::Error> {
        if let CommandType::Line = value.ctype() {
            let ((x, y), (x2, y2)) = (get_cursor_pos(), value.coords().resolve());
            let color = value.color().unwrap_or("red".to_string());
            Ok(Line {
                x1: RwSignal::new(x),
                y1: RwSignal::new(y),
                x2: RwSignal::new(x2),
                y2: RwSignal::new(y2),
                color: RwSignal::new(color),
            })
        } else {
            Err(value.ctype())
        }
    }
}

impl TryFrom<Command> for Rect {
    type Error = CommandType;
    fn try_from(command: Command) -> Result<Self, Self::Error> {
        if let CommandType::Rectangle = command.ctype() {
            let color = command.color().unwrap_or("red".to_string());
            let ((mut x, mut y), (x2, y2)) = (get_cursor_pos(), command.coords().resolve());
            let mut width: i32 = x2 as i32 - x as i32;
            let mut height = y2 as i32 - y as i32;
            if width < 0 {
                x = (x as i32 + width) as u32;
                width = -width;
            }
            if height < 0 {
                y = (y as i32 + height) as u32;
                height = -height;
            }
            Ok(Self {
                x: RwSignal::new(x),
                y: RwSignal::new(y),
                width: RwSignal::new(width as u32), // if this underflows, we're cooked
                height: RwSignal::new(height as u32), // if this underflows, we're cooked
                rx: RwSignal::new(Default::default()),
                ry: RwSignal::new(Default::default()),
                border_color: RwSignal::new(Default::default()),
                inner_color: RwSignal::new(color),
            })
        } else {
            Err(command.ctype())
        }
    }
}

impl TryFrom<Command> for Text {
    type Error = CommandType;
    fn try_from(command: Command) -> Result<Self, Self::Error> {
        match command.ctype() {
            CommandType::Text => {
                let text = loop {
                    match window()
                        .prompt_with_message_and_default("Text:", "I'm such a silly boykisser")
                    {
                        Ok(text) => match text {
                            Some(text) => break text,
                            None => {
                                window()
                                    .alert_with_message("You gotta put something in there!")
                                    .unwrap();
                            }
                        },
                        Err(jsval) => logging::warn!("User's fault: {jsval:?} (should be null)"),
                    }
                };
                let (x, y) = command.coords().resolve();
                let color = command.color().unwrap_or("red".to_string());
                Ok(Self {
                    x: x.into(),
                    y: y.into(),
                    text: text.into(),
                    font_size: Default::default(),
                    color: color.into(),
                })
            }
            other => Err(other),
        }
    }
}

impl TryFrom<Command> for Circle {
    type Error = CommandType;
    fn try_from(com: Command) -> Result<Self, Self::Error> {
        match com.ctype() {
            CommandType::Circle(rad) => {
                let (x, y) = com.coords().resolve();
                let color = com.color().unwrap_or("red".to_string());
                Ok(Self {
                    radius: RwSignal::new(rad),
                    x: RwSignal::new(x),
                    y: RwSignal::new(y),
                    color: RwSignal::new(color),
                })
            }
            other => Err(other),
        }
    }
}
