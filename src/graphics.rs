use crate::components::Selectable;
use leptos::Signal;
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
use crate::parser::Coords;

macro_rules! gen_form {
    ($($type:ident),+) => {
        #[derive(Clone)]
        pub enum Form {
            $($type($type)),+
        }

        impl GraphicsItem for Form {
            fn key(&self) -> u128 {
                match self {
                    $(Self::$type(form) => form.key()),+
                }
            }
        }

        impl IntoView for Form {
            fn into_view(self) -> leptos::View {
                match self {
                    $(Self::$type(form) => form.into_view()),+
                }

                // let (edit, set_edit) = create_signal(true);
                // view! {
                //     <Selectable edit={edit}>
                //     {
                //         match self {
                //             $(Self::$type(form) => form.into_view()),+
                //         }
                //     }
                //     </Selectable>
                // }
            }
        }
    };
}

fn key_from_four(n1: u32, n2: u32, n3: u32, n4: u32) -> u128 {
    ((n1 as u128) << 96u128) + ((n2 as u128) << 64u128) + ((n3 as u128) << 32u128) + n4 as u128
}
fn format_css<T: Display>(c: T) -> String {
    format!("{}%", c)
}
pub trait GraphicsItem: Clone {
    fn key(&self) -> u128;
}

gen_form!(Line, Rect, Text, Circle);

#[derive(Clone)]
pub struct Line {
    x1: RwSignal<u32>,
    y1: RwSignal<u32>,
    x2: RwSignal<u32>,
    y2: RwSignal<u32>,
    color: String,
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
            color: Default::default(),
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
}

#[derive(Clone)]
pub struct Rect {
    x: RwSignal<u32>,
    y: RwSignal<u32>,
    width: RwSignal<i32>,
    height: RwSignal<i32>,
    rx: RwSignal<u32>,
    ry: RwSignal<u32>,
    border_color: RwSignal<String>,
    inner_color: RwSignal<String>,
}

impl Rect {
    pub fn new(x: u32, y: u32, x2: u32, y2: u32) -> Self {
        Self {
            x: RwSignal::new(x),
            y: RwSignal::new(y),
            width: RwSignal::new(x2 as i32 - x as i32), // if this underflows, we're cooked
            height: RwSignal::new(y2 as i32 - y as i32), // if this underflows, we're cooked
            rx: RwSignal::new(Default::default()),
            ry: RwSignal::new(Default::default()),
            border_color: RwSignal::new(Default::default()),
            inner_color: RwSignal::new("red".to_string()),
        }
    }
    pub fn from(tuple: (u32, u32, u32, u32)) -> Self {
        logging::log!("Creating new rect with {tuple:?}");
        let (x, y, x2, y2) = tuple;
        Self::new(x, y, x2, y2)
    }

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
        let mut width: Signal<i32> = self.width.into();
        let mut height: Signal<i32> = self.height.into();
        if width() < 0 {
            x1 = Signal::derive(move || (x1() as i32 + width()) as u32);
            width = Signal::derive(move || -width());
        }
        if height() < 0 {
            y1 = Signal::derive(move || (y1() as i32 + height()) as u32);
            height = Signal::derive(move || -height());
        }
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
}

#[derive(Clone)]
pub struct Text {
    x: RwSignal<u32>,
    y: RwSignal<u32>,
    text: RwSignal<String>,
}

impl Text {
    pub fn from(command: Command) -> Result<Self, CommandType> {
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
                Ok(Self {
                    x: x.into(),
                    y: y.into(),
                    text: text.into(),
                })
            }
            other => Err(other),
        }
    }

    fn css_coords_reactive(&self) -> (impl Fn() -> String, impl Fn() -> String) {
        let x = self.x;
        let y = self.y;
        (move || format_css((x)()), move || format_css((y)()))
    }
}

#[derive(Clone)]
pub struct Circle {
    radius: RwSignal<u32>,
    x: RwSignal<u32>,
    y: RwSignal<u32>,
}

impl Circle {
    pub fn new(radius: u32, x: u32, y: u32) -> Self {
        Self {
            radius: RwSignal::new(radius),
            x: RwSignal::new(x),
            y: RwSignal::new(y),
        }
    }

    pub fn from(com: Command) -> Result<Self, ()> {
        match com.ctype() {
            CommandType::Circle(rad) => {
                let (x, y) = com.coords().resolve();
                Ok(Self::new(rad, x, y))
            }
            _ => Err(()),
        }
    }
}

impl GraphicsItem for Circle {
    fn key(&self) -> u128 {
        (self.radius)() as u128
    }
}

impl GraphicsItem for Text {
    fn key(&self) -> u128 {
        let mut hasher = DefaultHasher::new();
        ((((self.x)() as u128) << 32u128) + (self.y)() as u128).hash(&mut hasher);
        (self.text)().hash(&mut hasher);
        hasher.finish() as u128
    }
}

const DEFAULT_STYLE: &str = "stroke:red;stroke-width=2";

impl IntoView for Line {
    fn into_view(self) -> leptos::View {
        logging::log!("called into_view() on Line");
        let (x1, y1, x2, y2) = self.css_coords_reactive();
        view! {
            <line x1={x1} y1={y1} x2={x2} y2={y2} style={DEFAULT_STYLE}/>
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
            <rect x={x} y={y} width={width} height={height} fill={self.inner_color}/>
        }
        .into_view()
    }
}

impl IntoView for Text {
    fn into_view(self) -> leptos::View {
        let (x, y) = self.css_coords_reactive();
        view! {
            <text x={x} y={y}>{self.text}</text>
        }
        .into_view()
    }
}

impl IntoView for Circle {
    fn into_view(self) -> leptos::View {
        view! {
            <circle r={move || format_css((self.radius)())} cx={move || format_css((self.x)())} cy={move || format_css((self.y)())}/>
        }
        .into_view()
    }
}
