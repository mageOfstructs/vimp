use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use leptos::logging;
use leptos::IntoView;
use leptos::ReadSignal;
use leptos::{view, RwSignal};

use crate::parser::Command;
use crate::parser::CommandFSM;
use crate::parser::CommandType;
use crate::parser::Coords;

type KeyType = u128;

fn key_from_four(n1: u32, n2: u32, n3: u32, n4: u32) -> u128 {
    ((n1 as u128) << 96u128) + ((n2 as u128) << 64u128) + ((n3 as u128) << 32u128) + n4 as u128
}
fn format_css(c: u32) -> String {
    format!("{}%", c)
}
pub trait GraphicsItem: Clone {
    fn key(&self) -> u128;
}

#[derive(Clone)]
pub enum Form {
    Line(Line),
    Rect(Rect),
    Text(Text),
}

impl GraphicsItem for Form {
    fn key(&self) -> u128 {
        match self {
            Self::Line(l) => l.key(),
            Self::Rect(r) => r.key(),
            Self::Text(t) => t.key(),
        }
    }
}

impl IntoView for Form {
    fn into_view(self) -> leptos::View {
        match self {
            Self::Line(l) => l.into_view(),
            Self::Rect(r) => r.into_view(),
            Self::Text(t) => t.into_view(),
        }
    }
}

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
        let x1 = self.x1.clone();
        let y1 = self.y1.clone();
        let x2 = self.x2.clone();
        let y2 = self.y2.clone();
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
        return ret;
    }
}

#[derive(Clone)]
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
    pub fn new(x: u32, y: u32, x2: u32, y2: u32) -> Self {
        Self {
            x: RwSignal::new(x),
            y: RwSignal::new(y),
            width: RwSignal::new(x2 - x),
            height: RwSignal::new(y2 - y),
            rx: RwSignal::new(Default::default()),
            ry: RwSignal::new(Default::default()),
            border_color: RwSignal::new(Default::default()),
            inner_color: RwSignal::new("red".to_string()),
        }
    }
    pub fn from(tuple: (u32, u32, u32, u32)) -> Self {
        let (x, y, width, height) = tuple;
        Self::new(x, y, width, height)
    }

    fn css_coords_reactive(
        &self,
    ) -> (
        impl Fn() -> String,
        impl Fn() -> String,
        impl Fn() -> String,
        impl Fn() -> String,
    ) {
        let x1 = self.x.clone();
        let y1 = self.y.clone();
        let width = self.width.clone();
        let height = self.height.clone();
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
        key_from_four((self.x)(), (self.y)(), (self.width)(), (self.height)())
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
            // TODO: has some repetition, CLEAN THAT UP
            CommandType::Text(text) => match command.coords() {
                Coords::AbsCoord(x, y) => Ok(Self {
                    x: x.into(),
                    y: y.into(),
                    text: text.into(),
                }),
                Coords::RelCoord(fcp) => {
                    let (x, y) = fcp.resolve_fcp();
                    Ok(Self {
                        x: x.into(),
                        y: y.into(),
                        text: text.into(),
                    })
                }
            },
            other => Err(other),
        }
    }

    fn css_coords_reactive(&self) -> (impl Fn() -> String, impl Fn() -> String) {
        let x = self.x.clone();
        let y = self.y.clone();
        (move || format_css((x)()), move || format_css((y)()))
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
