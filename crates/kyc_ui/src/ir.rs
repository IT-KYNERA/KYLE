use std::fmt;

// StyleDecl, StyleProp, MediaQuery, AnimDecl are re-exported from ast.rs
// since they are shared across both the parser AST and the UI-IR.
pub use crate::ast::{StyleDecl, StyleProp, MediaQuery, AnimDecl};

/// UI-IR: Platform-agnostic intermediate representation for .kyx UI code.
/// Backends (web, desktop, android, etc.) consume this to generate platform code.

/// A complete .kyx program
#[derive(Clone, Debug)]
pub struct UiProgram {
    pub view_paths: Vec<String>,
    pub code_blocks: Vec<String>,
    pub styles: Vec<StyleDecl>,
    pub animations: Vec<AnimDecl>,
    pub body: Vec<UiNode>,
}

/// Platform-agnostic UI node
#[derive(Clone, Debug)]
pub enum UiNode {
    Element {
        tag: ComponentTag,
        attrs: Vec<UiAttr>,
        children: Vec<UiNode>,
    },
    SelfClosing {
        tag: ComponentTag,
        attrs: Vec<UiAttr>,
    },
    Slot {
        name: String,
        fallback: Vec<UiNode>,
    },
    If {
        condition: String,
        then_branch: Vec<UiNode>,
        else_branch: Vec<UiNode>,
    },
    For {
        item: String,
        list: String,
        body: Vec<UiNode>,
    },
    Match {
        expr: String,
        cases: Vec<MatchCase>,
    },
    Expr(String),
    CodeBlock(String),
    Text(String),
}

/// A match case in @match
#[derive(Clone, Debug)]
pub struct MatchCase {
    pub pattern: String,
    pub body: Vec<UiNode>,
}

/// An XML attribute in UI-IR
#[derive(Clone, Debug)]
pub struct UiAttr {
    pub name: String,
    pub value: AttrValue,
}

/// Attribute value
#[derive(Clone, Debug)]
pub enum AttrValue {
    String(String),
    Expr(String),
    Flag,
}

/// Platform-agnostic component tag.
/// Known tags are enum variants; unknown/unrecognized tags fall to Custom.
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentTag {
    View, Column, Row, Grid, Scroll, Spacer, Card, Surface, Stack, Container, Form, List,
    Text, Label, Button, IconButton,
    TextField, PasswordField, TextArea,
    Checkbox, Radio, Switch, Slider, Select,
    Image, Icon, Canvas, Video, Audio, Link,
    Dialog, Tooltip, Popup, Snackbar, Progress, Spinner, Skeleton,
    Tabs, Tab, Nav, AppBar, BottomBar, Drawer, Menu, MenuItem,
    Router, Outlet, Portal, ErrorBoundary,
    Custom(String),
}

impl ComponentTag {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "view" => Self::View,
            "column" => Self::Column,
            "row" => Self::Row,
            "grid" => Self::Grid,
            "scroll" => Self::Scroll,
            "spacer" => Self::Spacer,
            "card" => Self::Card,
            "surface" => Self::Surface,
            "stack" => Self::Stack,
            "container" => Self::Container,
            "form" => Self::Form,
            "list" => Self::List,
            "text" => Self::Text,
            "label" => Self::Label,
            "button" => Self::Button,
            "iconbutton" | "icon_button" => Self::IconButton,
            "textfield" | "text_field" => Self::TextField,
            "passwordfield" | "password_field" => Self::PasswordField,
            "textarea" | "text_area" => Self::TextArea,
            "checkbox" => Self::Checkbox,
            "radio" => Self::Radio,
            "switch" => Self::Switch,
            "slider" => Self::Slider,
            "select" => Self::Select,
            "image" => Self::Image,
            "icon" => Self::Icon,
            "canvas" => Self::Canvas,
            "video" => Self::Video,
            "audio" => Self::Audio,
            "link" => Self::Link,
            "dialog" => Self::Dialog,
            "tooltip" => Self::Tooltip,
            "popup" => Self::Popup,
            "snackbar" => Self::Snackbar,
            "progress" => Self::Progress,
            "spinner" => Self::Spinner,
            "skeleton" => Self::Skeleton,
            "tabs" => Self::Tabs,
            "tab" => Self::Tab,
            "nav" => Self::Nav,
            "appbar" | "app_bar" => Self::AppBar,
            "bottombar" | "bottom_bar" => Self::BottomBar,
            "drawer" => Self::Drawer,
            "menu" => Self::Menu,
            "menuitem" | "menu_item" => Self::MenuItem,
            "router" => Self::Router,
            "outlet" => Self::Outlet,
            "portal" => Self::Portal,
            "error_boundary" | "errorboundary" => Self::ErrorBoundary,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::View => "view",
            Self::Column => "column",
            Self::Row => "row",
            Self::Grid => "grid",
            Self::Scroll => "scroll",
            Self::Spacer => "spacer",
            Self::Card => "card",
            Self::Surface => "surface",
            Self::Stack => "stack",
            Self::Container => "container",
            Self::Form => "form",
            Self::List => "list",
            Self::Text => "text",
            Self::Label => "label",
            Self::Button => "button",
            Self::IconButton => "iconbutton",
            Self::TextField => "textfield",
            Self::PasswordField => "passwordfield",
            Self::TextArea => "textarea",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Switch => "switch",
            Self::Slider => "slider",
            Self::Select => "select",
            Self::Image => "image",
            Self::Icon => "icon",
            Self::Canvas => "canvas",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Link => "link",
            Self::Dialog => "dialog",
            Self::Tooltip => "tooltip",
            Self::Popup => "popup",
            Self::Snackbar => "snackbar",
            Self::Progress => "progress",
            Self::Spinner => "spinner",
            Self::Skeleton => "skeleton",
            Self::Tabs => "tabs",
            Self::Tab => "tab",
            Self::Nav => "nav",
            Self::AppBar => "appbar",
            Self::BottomBar => "bottombar",
            Self::Drawer => "drawer",
            Self::Menu => "menu",
            Self::MenuItem => "menuitem",
            Self::Router => "router",
            Self::Outlet => "outlet",
            Self::Portal => "portal",
            Self::ErrorBoundary => "errorboundary",
            Self::Custom(s) => s,
        }
    }
}

impl fmt::Display for UiProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for path in &self.view_paths {
            writeln!(f, "view(\"{}\")", path)?;
        }
        for block in &self.code_blocks {
            writeln!(f, "@({})", block)?;
        }
        for anim in &self.animations {
            writeln!(f, "animation<{}> {}:", anim.component, anim.name)?;
            for p in &anim.props { writeln!(f, "    {} = {}", p.name, p.value)?; }
        }
        for node in &self.body {
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

impl fmt::Display for UiNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiNode::Element { tag, attrs, children } => {
                write!(f, "<{}", tag.as_str())?;
                for a in attrs { write!(f, " {}", a)?; }
                if children.is_empty() {
                    writeln!(f, " />")?;
                } else {
                    writeln!(f, ">")?;
                    for c in children { write!(f, "  {}", c)?; }
                    writeln!(f, "</{}>", tag.as_str())?;
                }
                Ok(())
            }
            UiNode::SelfClosing { tag, attrs } => {
                write!(f, "<{}", tag.as_str())?;
                for a in attrs { write!(f, " {}", a)?; }
                writeln!(f, " />")
            }
            UiNode::Slot { name, .. } => writeln!(f, "<slot name=\"{}\" />", name),
            UiNode::If { condition, .. } => writeln!(f, "@if({}): ...", condition),
            UiNode::For { item, list, .. } => writeln!(f, "@for({} in {}): ...", item, list),
            UiNode::Match { expr, .. } => writeln!(f, "@match({}): ...", expr),
            UiNode::Expr(e) => writeln!(f, "@{}", e),
            UiNode::CodeBlock(b) => writeln!(f, "@({})", b),
            UiNode::Text(t) => write!(f, "{}", t),
        }
    }
}

impl fmt::Display for UiAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            AttrValue::String(s) => write!(f, "{}=\"{}\"", self.name, s),
            AttrValue::Expr(e) => write!(f, "{}={}", self.name, e),
            AttrValue::Flag => write!(f, "{}", self.name),
        }
    }
}
