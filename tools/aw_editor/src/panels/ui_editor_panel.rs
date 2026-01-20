//! UI Editor Panel for the editor UI
//!
//! Provides comprehensive runtime UI building:
//! - Canvas and layout management
//! - Widget library (buttons, text, images, sliders, etc.)
//! - Anchoring and responsive design
//! - UI animation and transitions
//! - Style themes and presets
//! - Event binding

use egui::{Color32, RichText, Ui, Vec2};

use crate::panels::Panel;

/// UI widget type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WidgetType {
    #[default]
    Panel,
    Button,
    Label,
    Image,
    Slider,
    ProgressBar,
    Toggle,
    Dropdown,
    TextField,
    ScrollView,
    Grid,
    HorizontalLayout,
    VerticalLayout,
}

impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl WidgetType {
    pub fn name(&self) -> &'static str {
        match self {
            WidgetType::Panel => "Panel",
            WidgetType::Button => "Button",
            WidgetType::Label => "Label",
            WidgetType::Image => "Image",
            WidgetType::Slider => "Slider",
            WidgetType::ProgressBar => "Progress Bar",
            WidgetType::Toggle => "Toggle",
            WidgetType::Dropdown => "Dropdown",
            WidgetType::TextField => "Text Field",
            WidgetType::ScrollView => "Scroll View",
            WidgetType::Grid => "Grid",
            WidgetType::HorizontalLayout => "Horizontal Layout",
            WidgetType::VerticalLayout => "Vertical Layout",
        }
    }

    pub fn all() -> &'static [WidgetType] {
        &[
            WidgetType::Panel,
            WidgetType::Button,
            WidgetType::Label,
            WidgetType::Image,
            WidgetType::Slider,
            WidgetType::ProgressBar,
            WidgetType::Toggle,
            WidgetType::Dropdown,
            WidgetType::TextField,
            WidgetType::ScrollView,
            WidgetType::Grid,
            WidgetType::HorizontalLayout,
            WidgetType::VerticalLayout,
        ]
    }

    pub fn icon(&self) -> &'static str {
        match self {
            WidgetType::Panel => "⬜",
            WidgetType::Button => "🔘",
            WidgetType::Label => "📝",
            WidgetType::Image => "🖼️",
            WidgetType::Slider => "🎚️",
            WidgetType::ProgressBar => "📊",
            WidgetType::Toggle => "☑️",
            WidgetType::Dropdown => "📋",
            WidgetType::TextField => "✏️",
            WidgetType::ScrollView => "📜",
            WidgetType::Grid => "⊞",
            WidgetType::HorizontalLayout => "↔️",
            WidgetType::VerticalLayout => "↕️",
        }
    }
}

/// Anchor preset for UI positioning
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AnchorPreset {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    StretchHorizontal,
    StretchVertical,
    StretchFull,
}

impl std::fmt::Display for AnchorPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl AnchorPreset {
    pub fn all() -> &'static [AnchorPreset] {
        &[
            AnchorPreset::TopLeft,
            AnchorPreset::TopCenter,
            AnchorPreset::TopRight,
            AnchorPreset::MiddleLeft,
            AnchorPreset::MiddleCenter,
            AnchorPreset::MiddleRight,
            AnchorPreset::BottomLeft,
            AnchorPreset::BottomCenter,
            AnchorPreset::BottomRight,
            AnchorPreset::StretchHorizontal,
            AnchorPreset::StretchVertical,
            AnchorPreset::StretchFull,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AnchorPreset::TopLeft => "Top Left",
            AnchorPreset::TopCenter => "Top Center",
            AnchorPreset::TopRight => "Top Right",
            AnchorPreset::MiddleLeft => "Middle Left",
            AnchorPreset::MiddleCenter => "Middle Center",
            AnchorPreset::MiddleRight => "Middle Right",
            AnchorPreset::BottomLeft => "Bottom Left",
            AnchorPreset::BottomCenter => "Bottom Center",
            AnchorPreset::BottomRight => "Bottom Right",
            AnchorPreset::StretchHorizontal => "Stretch Horizontal",
            AnchorPreset::StretchVertical => "Stretch Vertical",
            AnchorPreset::StretchFull => "Stretch Full",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AnchorPreset::TopLeft => "↖️",
            AnchorPreset::TopCenter => "⬆️",
            AnchorPreset::TopRight => "↗️",
            AnchorPreset::MiddleLeft => "⬅️",
            AnchorPreset::MiddleCenter => "⏺️",
            AnchorPreset::MiddleRight => "➡️",
            AnchorPreset::BottomLeft => "↙️",
            AnchorPreset::BottomCenter => "⬇️",
            AnchorPreset::BottomRight => "↘️",
            AnchorPreset::StretchHorizontal => "↔️",
            AnchorPreset::StretchVertical => "↕️",
            AnchorPreset::StretchFull => "⛶",
        }
    }
}

/// UI widget definition
#[derive(Debug, Clone)]
pub struct UiWidget {
    pub id: u32,
    pub name: String,
    pub widget_type: WidgetType,
    pub enabled: bool,
    pub visible: bool,

    // Transform
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub anchor: AnchorPreset,
    pub pivot: [f32; 2],
    pub rotation: f32,

    // Layout
    pub padding: [f32; 4], // top, right, bottom, left
    pub margin: [f32; 4],
    pub spacing: f32,

    // Appearance
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub corner_radius: f32,
    pub opacity: f32,

    // Type-specific
    pub text: String,
    pub font_size: f32,
    pub text_color: [f32; 4],
    pub image_path: String,
    pub value: f32,
    pub min_value: f32,
    pub max_value: f32,

    // Events
    pub on_click: String,
    pub on_value_changed: String,
    pub on_hover_enter: String,
    pub on_hover_exit: String,

    // Hierarchy
    pub parent_id: Option<u32>,
    pub children: Vec<u32>,
}

impl Default for UiWidget {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Widget".to_string(),
            widget_type: WidgetType::Panel,
            enabled: true,
            visible: true,

            position: [0.0, 0.0],
            size: [100.0, 50.0],
            anchor: AnchorPreset::TopLeft,
            pivot: [0.0, 0.0],
            rotation: 0.0,

            padding: [5.0, 5.0, 5.0, 5.0],
            margin: [0.0, 0.0, 0.0, 0.0],
            spacing: 5.0,

            background_color: [0.2, 0.2, 0.2, 1.0],
            border_color: [0.5, 0.5, 0.5, 1.0],
            border_width: 1.0,
            corner_radius: 4.0,
            opacity: 1.0,

            text: String::new(),
            font_size: 14.0,
            text_color: [1.0, 1.0, 1.0, 1.0],
            image_path: String::new(),
            value: 0.0,
            min_value: 0.0,
            max_value: 1.0,

            on_click: String::new(),
            on_value_changed: String::new(),
            on_hover_enter: String::new(),
            on_hover_exit: String::new(),

            parent_id: None,
            children: Vec::new(),
        }
    }
}

/// UI Canvas (root container)
#[derive(Debug, Clone)]
pub struct UiCanvas {
    pub id: u32,
    pub name: String,
    pub resolution: [u32; 2],
    pub scale_mode: ScaleMode,
    pub reference_resolution: [u32; 2],
    pub match_width_or_height: f32,
    pub widgets: Vec<UiWidget>,
    pub render_order: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // ScaleMode variants all end with Size by design
pub enum ScaleMode {
    #[default]
    ConstantPixelSize,
    ScaleWithScreenSize,
    ConstantPhysicalSize,
}

impl std::fmt::Display for ScaleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl ScaleMode {
    pub fn all() -> &'static [ScaleMode] {
        &[
            ScaleMode::ConstantPixelSize,
            ScaleMode::ScaleWithScreenSize,
            ScaleMode::ConstantPhysicalSize,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ScaleMode::ConstantPixelSize => "Constant Pixel Size",
            ScaleMode::ScaleWithScreenSize => "Scale With Screen Size",
            ScaleMode::ConstantPhysicalSize => "Constant Physical Size",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ScaleMode::ConstantPixelSize => "📏",
            ScaleMode::ScaleWithScreenSize => "📱",
            ScaleMode::ConstantPhysicalSize => "📐",
        }
    }
}

impl Default for UiCanvas {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Canvas".to_string(),
            resolution: [1920, 1080],
            scale_mode: ScaleMode::ScaleWithScreenSize,
            reference_resolution: [1920, 1080],
            match_width_or_height: 0.5,
            widgets: Vec::new(),
            render_order: 0,
        }
    }
}

/// UI Style/Theme
#[derive(Debug, Clone)]
pub struct UiStyle {
    pub id: u32,
    pub name: String,

    // Colors
    pub primary_color: [f32; 4],
    pub secondary_color: [f32; 4],
    pub accent_color: [f32; 4],
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub disabled_color: [f32; 4],

    // Typography
    pub font_family: String,
    pub font_size_small: f32,
    pub font_size_normal: f32,
    pub font_size_large: f32,
    pub font_size_heading: f32,

    // Spacing
    pub padding_small: f32,
    pub padding_normal: f32,
    pub padding_large: f32,
    pub corner_radius: f32,
    pub border_width: f32,

    // Transitions
    pub transition_duration: f32,
    pub hover_scale: f32,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Default".to_string(),

            primary_color: [0.2, 0.4, 0.8, 1.0],
            secondary_color: [0.3, 0.3, 0.35, 1.0],
            accent_color: [0.9, 0.6, 0.1, 1.0],
            background_color: [0.15, 0.15, 0.18, 1.0],
            text_color: [0.95, 0.95, 0.95, 1.0],
            disabled_color: [0.5, 0.5, 0.5, 0.5],

            font_family: "Default".to_string(),
            font_size_small: 12.0,
            font_size_normal: 14.0,
            font_size_large: 18.0,
            font_size_heading: 24.0,

            padding_small: 4.0,
            padding_normal: 8.0,
            padding_large: 16.0,
            corner_radius: 4.0,
            border_width: 1.0,

            transition_duration: 0.15,
            hover_scale: 1.02,
        }
    }
}

/// UI Animation
#[derive(Debug, Clone)]
pub struct UiAnimation {
    pub id: u32,
    pub name: String,
    pub duration: f32,
    pub delay: f32,
    pub easing: EasingType,
    pub loop_mode: AnimLoopMode,
    pub keyframes: Vec<UiKeyframe>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum EasingType {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Back,
}

impl std::fmt::Display for EasingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl EasingType {
    pub fn all() -> &'static [EasingType] {
        &[
            EasingType::Linear,
            EasingType::EaseIn,
            EasingType::EaseOut,
            EasingType::EaseInOut,
            EasingType::Bounce,
            EasingType::Elastic,
            EasingType::Back,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EasingType::Linear => "Linear",
            EasingType::EaseIn => "Ease In",
            EasingType::EaseOut => "Ease Out",
            EasingType::EaseInOut => "Ease In-Out",
            EasingType::Bounce => "Bounce",
            EasingType::Elastic => "Elastic",
            EasingType::Back => "Back",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EasingType::Linear => "—",
            EasingType::EaseIn => "↗",
            EasingType::EaseOut => "↘",
            EasingType::EaseInOut => "⤴",
            EasingType::Bounce => "⤾",
            EasingType::Elastic => "〰️",
            EasingType::Back => "↶",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AnimLoopMode {
    #[default]
    Once,
    Loop,
    PingPong,
}

impl std::fmt::Display for AnimLoopMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl AnimLoopMode {
    pub fn all() -> &'static [AnimLoopMode] {
        &[
            AnimLoopMode::Once,
            AnimLoopMode::Loop,
            AnimLoopMode::PingPong,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AnimLoopMode::Once => "Once",
            AnimLoopMode::Loop => "Loop",
            AnimLoopMode::PingPong => "Ping Pong",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AnimLoopMode::Once => "1️⃣",
            AnimLoopMode::Loop => "🔄",
            AnimLoopMode::PingPong => "↔",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiKeyframe {
    pub time: f32,
    pub property: String,
    pub value: f32,
}

impl Default for UiAnimation {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Animation".to_string(),
            duration: 1.0,
            delay: 0.0,
            easing: EasingType::EaseInOut,
            loop_mode: AnimLoopMode::Once,
            keyframes: Vec::new(),
        }
    }
}

/// UI Preset template
#[derive(Debug, Clone)]
pub struct UiPreset {
    pub name: String,
    pub category: String,
    pub description: String,
    pub widgets: Vec<UiWidget>,
}

impl UiPreset {
    pub fn builtin_presets() -> Vec<UiPreset> {
        vec![
            UiPreset {
                name: "Health Bar".to_string(),
                category: "HUD".to_string(),
                description: "Player health display with icon".to_string(),
                widgets: Vec::new(),
            },
            UiPreset {
                name: "Minimap".to_string(),
                category: "HUD".to_string(),
                description: "Circular minimap with markers".to_string(),
                widgets: Vec::new(),
            },
            UiPreset {
                name: "Main Menu".to_string(),
                category: "Menus".to_string(),
                description: "Standard main menu layout".to_string(),
                widgets: Vec::new(),
            },
            UiPreset {
                name: "Pause Menu".to_string(),
                category: "Menus".to_string(),
                description: "Overlay pause menu".to_string(),
                widgets: Vec::new(),
            },
            UiPreset {
                name: "Dialog Box".to_string(),
                category: "Dialogs".to_string(),
                description: "Modal dialog with buttons".to_string(),
                widgets: Vec::new(),
            },
            UiPreset {
                name: "Inventory Grid".to_string(),
                category: "Gameplay".to_string(),
                description: "Grid-based inventory system".to_string(),
                widgets: Vec::new(),
            },
            UiPreset {
                name: "Quest Tracker".to_string(),
                category: "HUD".to_string(),
                description: "Active quest objectives".to_string(),
                widgets: Vec::new(),
            },
            UiPreset {
                name: "Loading Screen".to_string(),
                category: "System".to_string(),
                description: "Loading screen with progress".to_string(),
                widgets: Vec::new(),
            },
        ]
    }
}

/// Panel tabs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum UiEditorTab {
    #[default]
    Hierarchy,
    Canvas,
    Widget,
    Style,
    Animation,
    Presets,
    Preview,
}

impl std::fmt::Display for UiEditorTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl UiEditorTab {
    pub fn all() -> &'static [UiEditorTab] {
        &[
            UiEditorTab::Hierarchy,
            UiEditorTab::Canvas,
            UiEditorTab::Widget,
            UiEditorTab::Style,
            UiEditorTab::Animation,
            UiEditorTab::Presets,
            UiEditorTab::Preview,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            UiEditorTab::Hierarchy => "Hierarchy",
            UiEditorTab::Canvas => "Canvas",
            UiEditorTab::Widget => "Widget",
            UiEditorTab::Style => "Style",
            UiEditorTab::Animation => "Animation",
            UiEditorTab::Presets => "Presets",
            UiEditorTab::Preview => "Preview",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            UiEditorTab::Hierarchy => "🌳",
            UiEditorTab::Canvas => "🖼️",
            UiEditorTab::Widget => "🧩",
            UiEditorTab::Style => "🎨",
            UiEditorTab::Animation => "🎬",
            UiEditorTab::Presets => "📋",
            UiEditorTab::Preview => "👁️",
        }
    }
}

/// Main UI Editor Panel
pub struct UiEditorPanel {
    active_tab: UiEditorTab,

    // Canvases
    canvases: Vec<UiCanvas>,
    selected_canvas: Option<u32>,
    current_canvas: UiCanvas,

    // Widgets
    selected_widget: Option<u32>,
    current_widget: UiWidget,
    widget_clipboard: Option<UiWidget>,

    // Styles
    styles: Vec<UiStyle>,
    selected_style: Option<u32>,
    current_style: UiStyle,

    // Animations
    animations: Vec<UiAnimation>,
    selected_animation: Option<u32>,

    // Presets
    presets: Vec<UiPreset>,

    // Editor state
    show_guides: bool,
    snap_to_grid: bool,
    grid_size: f32,
    zoom: f32,

    // ID counters
    next_canvas_id: u32,
    next_widget_id: u32,
    next_style_id: u32,
    next_animation_id: u32,
}

impl Default for UiEditorPanel {
    fn default() -> Self {
        let mut panel = Self {
            active_tab: UiEditorTab::Hierarchy,

            canvases: Vec::new(),
            selected_canvas: None,
            current_canvas: UiCanvas::default(),

            selected_widget: None,
            current_widget: UiWidget::default(),
            widget_clipboard: None,

            styles: Vec::new(),
            selected_style: None,
            current_style: UiStyle::default(),

            animations: Vec::new(),
            selected_animation: None,

            presets: UiPreset::builtin_presets(),

            show_guides: true,
            snap_to_grid: true,
            grid_size: 10.0,
            zoom: 1.0,

            next_canvas_id: 1,
            next_widget_id: 1,
            next_style_id: 1,
            next_animation_id: 1,
        };

        panel.create_sample_data();
        panel
    }
}

impl UiEditorPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_sample_data(&mut self) {
        // Create main HUD canvas
        let canvas_id = self.next_canvas_id();
        let mut hud_canvas = UiCanvas {
            id: canvas_id,
            name: "HUD Canvas".to_string(),
            render_order: 10,
            ..Default::default()
        };

        // Add health bar
        let widget_id = self.next_widget_id();
        hud_canvas.widgets.push(UiWidget {
            id: widget_id,
            name: "Health Bar Background".to_string(),
            widget_type: WidgetType::Panel,
            position: [20.0, 20.0],
            size: [200.0, 30.0],
            background_color: [0.2, 0.2, 0.2, 0.8],
            corner_radius: 4.0,
            ..Default::default()
        });

        let widget_id = self.next_widget_id();
        hud_canvas.widgets.push(UiWidget {
            id: widget_id,
            name: "Health Bar Fill".to_string(),
            widget_type: WidgetType::ProgressBar,
            position: [22.0, 22.0],
            size: [196.0, 26.0],
            background_color: [0.8, 0.2, 0.2, 1.0],
            value: 0.75,
            ..Default::default()
        });

        // Add minimap
        let widget_id = self.next_widget_id();
        hud_canvas.widgets.push(UiWidget {
            id: widget_id,
            name: "Minimap".to_string(),
            widget_type: WidgetType::Image,
            position: [1720.0, 20.0],
            size: [180.0, 180.0],
            anchor: AnchorPreset::TopRight,
            corner_radius: 90.0,
            border_width: 2.0,
            border_color: [0.6, 0.6, 0.6, 1.0],
            ..Default::default()
        });

        self.canvases.push(hud_canvas.clone());
        self.current_canvas = hud_canvas;
        self.selected_canvas = Some(canvas_id);

        // Create menu canvas
        let canvas_id = self.next_canvas_id();
        let mut menu_canvas = UiCanvas {
            id: canvas_id,
            name: "Main Menu".to_string(),
            render_order: 100,
            ..Default::default()
        };

        // Add title
        let widget_id = self.next_widget_id();
        menu_canvas.widgets.push(UiWidget {
            id: widget_id,
            name: "Title".to_string(),
            widget_type: WidgetType::Label,
            position: [960.0, 200.0],
            size: [400.0, 80.0],
            anchor: AnchorPreset::TopCenter,
            text: "GAME TITLE".to_string(),
            font_size: 48.0,
            ..Default::default()
        });

        // Add menu buttons
        let buttons = ["New Game", "Continue", "Options", "Quit"];
        for (i, text) in buttons.iter().enumerate() {
            let widget_id = self.next_widget_id();
            menu_canvas.widgets.push(UiWidget {
                id: widget_id,
                name: format!("{} Button", text),
                widget_type: WidgetType::Button,
                position: [960.0, 400.0 + i as f32 * 70.0],
                size: [300.0, 50.0],
                anchor: AnchorPreset::TopCenter,
                text: text.to_string(),
                font_size: 18.0,
                background_color: [0.3, 0.3, 0.35, 1.0],
                corner_radius: 8.0,
                on_click: format!("On{}Click", text.replace(" ", "")),
                ..Default::default()
            });
        }

        self.canvases.push(menu_canvas);

        // Create default style
        let style_id = self.next_style_id();
        self.styles.push(UiStyle {
            id: style_id,
            name: "Default Theme".to_string(),
            ..Default::default()
        });

        let style_id = self.next_style_id();
        self.styles.push(UiStyle {
            id: style_id,
            name: "Dark Theme".to_string(),
            background_color: [0.1, 0.1, 0.12, 1.0],
            primary_color: [0.3, 0.5, 0.9, 1.0],
            ..Default::default()
        });

        if !self.styles.is_empty() {
            self.current_style = self.styles[0].clone();
            self.selected_style = Some(self.styles[0].id);
        }

        if !self.current_canvas.widgets.is_empty() {
            self.current_widget = self.current_canvas.widgets[0].clone();
            self.selected_widget = Some(self.current_canvas.widgets[0].id);
        }
    }

    fn next_canvas_id(&mut self) -> u32 {
        let id = self.next_canvas_id;
        self.next_canvas_id += 1;
        id
    }

    fn next_widget_id(&mut self) -> u32 {
        let id = self.next_widget_id;
        self.next_widget_id += 1;
        id
    }

    fn next_style_id(&mut self) -> u32 {
        let id = self.next_style_id;
        self.next_style_id += 1;
        id
    }

    #[allow(dead_code)]
    fn next_animation_id(&mut self) -> u32 {
        let id = self.next_animation_id;
        self.next_animation_id += 1;
        id
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (UiEditorTab::Hierarchy, "📋 Hierarchy"),
                (UiEditorTab::Canvas, "🖼️ Canvas"),
                (UiEditorTab::Widget, "🔲 Widget"),
                (UiEditorTab::Style, "🎨 Style"),
                (UiEditorTab::Animation, "🎬 Animation"),
                (UiEditorTab::Presets, "📦 Presets"),
                (UiEditorTab::Preview, "▶️ Preview"),
            ];

            for (tab, label) in tabs {
                let is_selected = self.active_tab == tab;
                let button = egui::Button::new(label).fill(if is_selected {
                    Color32::from_rgb(60, 100, 160)
                } else {
                    Color32::from_rgb(50, 50, 55)
                });

                if ui.add(button).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!("🖼️ {} canvases", self.canvases.len()));
            ui.separator();
            ui.label(format!("🔲 {} widgets", self.current_canvas.widgets.len()));
            ui.separator();
            ui.label(format!("🎨 {} styles", self.styles.len()));
        });

        ui.separator();
    }

    fn show_hierarchy_tab(&mut self, ui: &mut Ui) {
        ui.heading("📋 UI Hierarchy");
        ui.add_space(10.0);

        // Canvas selection
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("canvas_select")
                .selected_text(&self.current_canvas.name)
                .show_ui(ui, |ui| {
                    for canvas in &self.canvases.clone() {
                        if ui.selectable_value(&mut self.selected_canvas, Some(canvas.id), &canvas.name).clicked() {
                            self.current_canvas = canvas.clone();
                        }
                    }
                });

            if ui.button("+ Canvas").clicked() {
                let id = self.next_canvas_id();
                let new_canvas = UiCanvas {
                    id,
                    name: format!("Canvas {}", id),
                    ..Default::default()
                };
                self.canvases.push(new_canvas.clone());
                self.current_canvas = new_canvas;
                self.selected_canvas = Some(id);
            }
        });

        ui.add_space(10.0);

        // Widget tree
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Widgets").strong());
                if ui.button("+ Add").clicked() {
                    let id = self.next_widget_id();
                    let new_widget = UiWidget {
                        id,
                        name: format!("Widget {}", id),
                        ..Default::default()
                    };
                    self.current_canvas.widgets.push(new_widget.clone());
                    self.current_widget = new_widget;
                    self.selected_widget = Some(id);
                }
            });

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for widget in &self.current_canvas.widgets.clone() {
                        let is_selected = self.selected_widget == Some(widget.id);
                        let icon = widget.widget_type.icon();
                        let label = format!("{} {}", icon, widget.name);

                        let response = ui.selectable_label(is_selected, label);
                        if response.clicked() {
                            self.selected_widget = Some(widget.id);
                            self.current_widget = widget.clone();
                        }
                    }
                });
        });

        ui.add_space(10.0);

        // Widget type palette
        ui.group(|ui| {
            ui.label(RichText::new("Widget Palette").strong());

            egui::Grid::new("widget_palette")
                .num_columns(4)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    for (i, wt) in WidgetType::all().iter().enumerate() {
                        if ui.button(format!("{}\n{:?}", wt.icon(), wt)).clicked() {
                            let id = self.next_widget_id();
                            let new_widget = UiWidget {
                                id,
                                name: format!("{:?} {}", wt, id),
                                widget_type: *wt,
                                ..Default::default()
                            };
                            self.current_canvas.widgets.push(new_widget.clone());
                            self.current_widget = new_widget;
                            self.selected_widget = Some(id);
                        }
                        if (i + 1) % 4 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn show_canvas_tab(&mut self, ui: &mut Ui) {
        ui.heading("🖼️ Canvas Settings");
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label(RichText::new("📐 Properties").strong());

            egui::Grid::new("canvas_props")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.current_canvas.name);
                    ui.end_row();

                    ui.label("Resolution:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut self.current_canvas.resolution[0]).speed(1).prefix("W:").range(1..=7680));
                        ui.add(egui::DragValue::new(&mut self.current_canvas.resolution[1]).speed(1).prefix("H:").range(1..=4320));
                    });
                    ui.end_row();

                    ui.label("Scale Mode:");
                    egui::ComboBox::from_id_salt("scale_mode")
                        .selected_text(format!("{:?}", self.current_canvas.scale_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.current_canvas.scale_mode, ScaleMode::ConstantPixelSize, "Constant Pixel Size");
                            ui.selectable_value(&mut self.current_canvas.scale_mode, ScaleMode::ScaleWithScreenSize, "Scale With Screen Size");
                            ui.selectable_value(&mut self.current_canvas.scale_mode, ScaleMode::ConstantPhysicalSize, "Constant Physical Size");
                        });
                    ui.end_row();

                    ui.label("Render Order:");
                    ui.add(egui::DragValue::new(&mut self.current_canvas.render_order).speed(1));
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // Editor settings
        ui.group(|ui| {
            ui.label(RichText::new("🔧 Editor").strong());

            ui.checkbox(&mut self.show_guides, "Show Guides");
            ui.checkbox(&mut self.snap_to_grid, "Snap to Grid");

            ui.horizontal(|ui| {
                ui.label("Grid Size:");
                ui.add(egui::Slider::new(&mut self.grid_size, 1.0..=50.0));
            });

            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom, 0.25..=4.0).logarithmic(true));
            });
        });
    }

    fn show_widget_tab(&mut self, ui: &mut Ui) {
        ui.heading("🔲 Widget Properties");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                // Basic
                ui.group(|ui| {
                    ui.label(RichText::new("📝 Basic").strong());

                    egui::Grid::new("widget_basic")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.current_widget.name);
                            ui.end_row();

                            ui.label("Type:");
                            ui.label(format!("{} {:?}", self.current_widget.widget_type.icon(), self.current_widget.widget_type));
                            ui.end_row();

                            ui.label("Enabled:");
                            ui.checkbox(&mut self.current_widget.enabled, "");
                            ui.end_row();

                            ui.label("Visible:");
                            ui.checkbox(&mut self.current_widget.visible, "");
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Transform
                ui.group(|ui| {
                    ui.label(RichText::new("📐 Transform").strong());

                    egui::Grid::new("widget_transform")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Position:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_widget.position[0]).speed(1.0).prefix("X:"));
                                ui.add(egui::DragValue::new(&mut self.current_widget.position[1]).speed(1.0).prefix("Y:"));
                            });
                            ui.end_row();

                            ui.label("Size:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_widget.size[0]).speed(1.0).prefix("W:"));
                                ui.add(egui::DragValue::new(&mut self.current_widget.size[1]).speed(1.0).prefix("H:"));
                            });
                            ui.end_row();

                            ui.label("Anchor:");
                            egui::ComboBox::from_id_salt("anchor")
                                .selected_text(format!("{:?}", self.current_widget.anchor))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_widget.anchor, AnchorPreset::TopLeft, "Top Left");
                                    ui.selectable_value(&mut self.current_widget.anchor, AnchorPreset::TopCenter, "Top Center");
                                    ui.selectable_value(&mut self.current_widget.anchor, AnchorPreset::TopRight, "Top Right");
                                    ui.selectable_value(&mut self.current_widget.anchor, AnchorPreset::MiddleCenter, "Middle Center");
                                    ui.selectable_value(&mut self.current_widget.anchor, AnchorPreset::BottomCenter, "Bottom Center");
                                    ui.selectable_value(&mut self.current_widget.anchor, AnchorPreset::StretchFull, "Stretch Full");
                                });
                            ui.end_row();

                            ui.label("Rotation:");
                            ui.add(egui::Slider::new(&mut self.current_widget.rotation, 0.0..=360.0).suffix("°"));
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Appearance
                ui.group(|ui| {
                    ui.label(RichText::new("🎨 Appearance").strong());

                    egui::Grid::new("widget_appearance")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Background:");
                            let mut color = Color32::from_rgba_unmultiplied(
                                (self.current_widget.background_color[0] * 255.0) as u8,
                                (self.current_widget.background_color[1] * 255.0) as u8,
                                (self.current_widget.background_color[2] * 255.0) as u8,
                                (self.current_widget.background_color[3] * 255.0) as u8,
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                self.current_widget.background_color = [
                                    color.r() as f32 / 255.0,
                                    color.g() as f32 / 255.0,
                                    color.b() as f32 / 255.0,
                                    color.a() as f32 / 255.0,
                                ];
                            }
                            ui.end_row();

                            ui.label("Corner Radius:");
                            ui.add(egui::Slider::new(&mut self.current_widget.corner_radius, 0.0..=50.0));
                            ui.end_row();

                            ui.label("Border Width:");
                            ui.add(egui::Slider::new(&mut self.current_widget.border_width, 0.0..=10.0));
                            ui.end_row();

                            ui.label("Opacity:");
                            ui.add(egui::Slider::new(&mut self.current_widget.opacity, 0.0..=1.0));
                            ui.end_row();
                        });
                });

                // Type-specific content
                ui.add_space(10.0);
                match self.current_widget.widget_type {
                    WidgetType::Label | WidgetType::Button => {
                        ui.group(|ui| {
                            ui.label(RichText::new("📝 Text").strong());

                            egui::Grid::new("widget_text")
                                .num_columns(2)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Text:");
                                    ui.text_edit_singleline(&mut self.current_widget.text);
                                    ui.end_row();

                                    ui.label("Font Size:");
                                    ui.add(egui::Slider::new(&mut self.current_widget.font_size, 8.0..=72.0));
                                    ui.end_row();
                                });
                        });
                    }
                    WidgetType::Slider | WidgetType::ProgressBar => {
                        ui.group(|ui| {
                            ui.label(RichText::new("🎚️ Value").strong());

                            egui::Grid::new("widget_value")
                                .num_columns(2)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Value:");
                                    ui.add(egui::Slider::new(&mut self.current_widget.value, self.current_widget.min_value..=self.current_widget.max_value));
                                    ui.end_row();

                                    ui.label("Min:");
                                    ui.add(egui::DragValue::new(&mut self.current_widget.min_value).speed(0.1));
                                    ui.end_row();

                                    ui.label("Max:");
                                    ui.add(egui::DragValue::new(&mut self.current_widget.max_value).speed(0.1));
                                    ui.end_row();
                                });
                        });
                    }
                    WidgetType::Image => {
                        ui.group(|ui| {
                            ui.label(RichText::new("🖼️ Image").strong());

                            ui.horizontal(|ui| {
                                ui.label("Path:");
                                ui.text_edit_singleline(&mut self.current_widget.image_path);
                                if ui.button("📂").clicked() {
                                    // Open file dialog
                                }
                            });
                        });
                    }
                    _ => {}
                }
            });
    }

    fn show_style_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎨 UI Styles");
        ui.add_space(10.0);

        // Style selector
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("style_select")
                .selected_text(&self.current_style.name)
                .show_ui(ui, |ui| {
                    for style in &self.styles.clone() {
                        if ui.selectable_value(&mut self.selected_style, Some(style.id), &style.name).clicked() {
                            self.current_style = style.clone();
                        }
                    }
                });

            if ui.button("+ New").clicked() {
                let id = self.next_style_id();
                let new_style = UiStyle {
                    id,
                    name: format!("Style {}", id),
                    ..Default::default()
                };
                self.styles.push(new_style.clone());
                self.current_style = new_style;
                self.selected_style = Some(id);
            }
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                ui.group(|ui| {
                    ui.label(RichText::new("🎨 Colors").strong());

                    egui::Grid::new("style_colors")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            let color_fields = [
                                ("Primary:", &mut self.current_style.primary_color),
                                ("Secondary:", &mut self.current_style.secondary_color),
                                ("Accent:", &mut self.current_style.accent_color),
                                ("Background:", &mut self.current_style.background_color),
                                ("Text:", &mut self.current_style.text_color),
                            ];

                            for (label, color_arr) in color_fields {
                                ui.label(label);
                                let mut color = Color32::from_rgba_unmultiplied(
                                    (color_arr[0] * 255.0) as u8,
                                    (color_arr[1] * 255.0) as u8,
                                    (color_arr[2] * 255.0) as u8,
                                    (color_arr[3] * 255.0) as u8,
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    *color_arr = [
                                        color.r() as f32 / 255.0,
                                        color.g() as f32 / 255.0,
                                        color.b() as f32 / 255.0,
                                        color.a() as f32 / 255.0,
                                    ];
                                }
                                ui.end_row();
                            }
                        });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.label(RichText::new("📝 Typography").strong());

                    egui::Grid::new("style_typography")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Font Family:");
                            ui.text_edit_singleline(&mut self.current_style.font_family);
                            ui.end_row();

                            ui.label("Font Normal:");
                            ui.add(egui::DragValue::new(&mut self.current_style.font_size_normal).speed(0.5));
                            ui.end_row();

                            ui.label("Font Large:");
                            ui.add(egui::DragValue::new(&mut self.current_style.font_size_large).speed(0.5));
                            ui.end_row();

                            ui.label("Font Heading:");
                            ui.add(egui::DragValue::new(&mut self.current_style.font_size_heading).speed(0.5));
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.label(RichText::new("📐 Spacing").strong());

                    egui::Grid::new("style_spacing")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Corner Radius:");
                            ui.add(egui::Slider::new(&mut self.current_style.corner_radius, 0.0..=20.0));
                            ui.end_row();

                            ui.label("Border Width:");
                            ui.add(egui::Slider::new(&mut self.current_style.border_width, 0.0..=5.0));
                            ui.end_row();

                            ui.label("Padding Normal:");
                            ui.add(egui::DragValue::new(&mut self.current_style.padding_normal).speed(0.5));
                            ui.end_row();
                        });
                });
            });
    }

    fn show_animation_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎬 UI Animations");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("+ New Animation").clicked() {
                let id = self.next_animation_id;
                self.next_animation_id += 1;
                self.animations.push(UiAnimation {
                    id,
                    name: format!("Animation {}", id),
                    ..Default::default()
                });
            }
        });

        ui.add_space(10.0);

        if self.animations.is_empty() {
            ui.label("No animations. Click '+ New Animation' to create one.");
        } else {
            for anim in &mut self.animations {
                ui.group(|ui| {
                    ui.label(RichText::new(&anim.name).strong());

                    egui::Grid::new(format!("anim_{}", anim.id))
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Duration:");
                            ui.add(egui::DragValue::new(&mut anim.duration).speed(0.1).suffix("s"));
                            ui.end_row();

                            ui.label("Delay:");
                            ui.add(egui::DragValue::new(&mut anim.delay).speed(0.1).suffix("s"));
                            ui.end_row();

                            ui.label("Easing:");
                            egui::ComboBox::from_id_salt(format!("easing_{}", anim.id))
                                .selected_text(format!("{:?}", anim.easing))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut anim.easing, EasingType::Linear, "Linear");
                                    ui.selectable_value(&mut anim.easing, EasingType::EaseIn, "Ease In");
                                    ui.selectable_value(&mut anim.easing, EasingType::EaseOut, "Ease Out");
                                    ui.selectable_value(&mut anim.easing, EasingType::EaseInOut, "Ease In Out");
                                    ui.selectable_value(&mut anim.easing, EasingType::Bounce, "Bounce");
                                    ui.selectable_value(&mut anim.easing, EasingType::Elastic, "Elastic");
                                });
                            ui.end_row();

                            ui.label("Loop:");
                            egui::ComboBox::from_id_salt(format!("loop_{}", anim.id))
                                .selected_text(format!("{:?}", anim.loop_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut anim.loop_mode, AnimLoopMode::Once, "Once");
                                    ui.selectable_value(&mut anim.loop_mode, AnimLoopMode::Loop, "Loop");
                                    ui.selectable_value(&mut anim.loop_mode, AnimLoopMode::PingPong, "Ping Pong");
                                });
                            ui.end_row();
                        });
                });
            }
        }
    }

    fn show_presets_tab(&mut self, ui: &mut Ui) {
        ui.heading("📦 UI Presets");
        ui.add_space(10.0);

        let categories: Vec<_> = self.presets.iter().map(|p| p.category.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();

        for category in categories {
            ui.group(|ui| {
                ui.label(RichText::new(&category).strong());

                for preset in self.presets.iter().filter(|p| p.category == category) {
                    ui.horizontal(|ui| {
                        if ui.button("➕").clicked() {
                            // Add preset to canvas
                        }
                        ui.label(&preset.name);
                        ui.label(RichText::new(&preset.description).small().color(Color32::GRAY));
                    });
                }
            });
            ui.add_space(5.0);
        }
    }

    fn show_preview_tab(&mut self, ui: &mut Ui) {
        ui.heading("▶️ Preview");
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label(RichText::new("🖥️ Preview Settings").strong());

            ui.horizontal(|ui| {
                if ui.button("📱 Mobile").clicked() {
                    self.current_canvas.resolution = [375, 812];
                }
                if ui.button("📟 Tablet").clicked() {
                    self.current_canvas.resolution = [1024, 768];
                }
                if ui.button("🖥️ Desktop").clicked() {
                    self.current_canvas.resolution = [1920, 1080];
                }
                if ui.button("📺 4K").clicked() {
                    self.current_canvas.resolution = [3840, 2160];
                }
            });
        });

        ui.add_space(10.0);

        // Preview area
        let preview_height = 250.0;
        let preview_width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(preview_width, preview_height), egui::Sense::hover());

        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 35));

        // Draw canvas outline
        let aspect = self.current_canvas.resolution[0] as f32 / self.current_canvas.resolution[1] as f32;
        let canvas_height = preview_height - 20.0;
        let canvas_width = (canvas_height * aspect).min(preview_width - 20.0);
        let canvas_rect = egui::Rect::from_center_size(
            rect.center(),
            Vec2::new(canvas_width, canvas_height),
        );

        painter.rect_stroke(canvas_rect, 2.0, egui::Stroke::new(1.0, Color32::GRAY), egui::StrokeKind::Outside);

        // Draw resolution label
        painter.text(
            egui::Pos2::new(rect.center().x, rect.max.y - 10.0),
            egui::Align2::CENTER_CENTER,
            format!("{}×{}", self.current_canvas.resolution[0], self.current_canvas.resolution[1]),
            egui::FontId::default(),
            Color32::GRAY,
        );
    }

    // Getters for testing
    pub fn canvas_count(&self) -> usize {
        self.canvases.len()
    }

    pub fn widget_count(&self) -> usize {
        self.current_canvas.widgets.len()
    }

    pub fn style_count(&self) -> usize {
        self.styles.len()
    }

    pub fn preset_count(&self) -> usize {
        self.presets.len()
    }

    pub fn add_canvas(&mut self, name: &str) -> u32 {
        let id = self.next_canvas_id();
        self.canvases.push(UiCanvas {
            id,
            name: name.to_string(),
            ..Default::default()
        });
        id
    }

    pub fn add_widget(&mut self, name: &str, widget_type: WidgetType) -> u32 {
        let id = self.next_widget_id();
        self.current_canvas.widgets.push(UiWidget {
            id,
            name: name.to_string(),
            widget_type,
            ..Default::default()
        });
        id
    }
}

impl Panel for UiEditorPanel {
    fn name(&self) -> &'static str {
        "UI Editor"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            UiEditorTab::Hierarchy => self.show_hierarchy_tab(ui),
            UiEditorTab::Canvas => self.show_canvas_tab(ui),
            UiEditorTab::Widget => self.show_widget_tab(ui),
            UiEditorTab::Style => self.show_style_tab(ui),
            UiEditorTab::Animation => self.show_animation_tab(ui),
            UiEditorTab::Presets => self.show_presets_tab(ui),
            UiEditorTab::Preview => self.show_preview_tab(ui),
        }
    }

    fn update(&mut self) {
        // Sync current widget back to canvas
        if let Some(widget_id) = self.selected_widget {
            if let Some(widget) = self.current_canvas.widgets.iter_mut().find(|w| w.id == widget_id) {
                *widget = self.current_widget.clone();
            }
        }

        // Sync current canvas back to list
        if let Some(canvas_id) = self.selected_canvas {
            if let Some(canvas) = self.canvases.iter_mut().find(|c| c.id == canvas_id) {
                *canvas = self.current_canvas.clone();
            }
        }

        // Sync current style back to list
        if let Some(style_id) = self.selected_style {
            if let Some(style) = self.styles.iter_mut().find(|s| s.id == style_id) {
                *style = self.current_style.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_editor_panel_creation() {
        let panel = UiEditorPanel::new();
        assert!(panel.canvas_count() >= 2);
    }

    #[test]
    fn test_default_sample_data() {
        let panel = UiEditorPanel::new();
        assert!(panel.widget_count() >= 3);
        assert!(panel.style_count() >= 2);
    }

    #[test]
    fn test_add_canvas() {
        let mut panel = UiEditorPanel::new();
        let initial = panel.canvas_count();
        let id = panel.add_canvas("Test Canvas");
        assert!(id > 0);
        assert_eq!(panel.canvas_count(), initial + 1);
    }

    #[test]
    fn test_add_widget() {
        let mut panel = UiEditorPanel::new();
        let initial = panel.widget_count();
        let id = panel.add_widget("Test Button", WidgetType::Button);
        assert!(id > 0);
        assert_eq!(panel.widget_count(), initial + 1);
    }

    #[test]
    fn test_widget_type_icons() {
        assert_eq!(WidgetType::Button.icon(), "🔘");
        assert_eq!(WidgetType::Label.icon(), "📝");
        assert_eq!(WidgetType::Image.icon(), "🖼️");
    }

    #[test]
    fn test_widget_type_all() {
        let all = WidgetType::all();
        assert!(all.len() >= 13);
    }

    #[test]
    fn test_builtin_presets() {
        let presets = UiPreset::builtin_presets();
        assert!(presets.len() >= 8);
    }

    // ============================================================
    // Session 5: Enum Enhancement Tests
    // ============================================================

    // WidgetType tests (8 tests)
    #[test]
    fn test_widget_type_display() {
        assert!(format!("{}", WidgetType::Panel).contains("Panel"));
        assert!(format!("{}", WidgetType::Button).contains("Button"));
        assert!(format!("{}", WidgetType::Label).contains("Label"));
        assert!(format!("{}", WidgetType::Slider).contains("Slider"));
    }

    #[test]
    fn test_widget_type_name() {
        assert_eq!(WidgetType::Panel.name(), "Panel");
        assert_eq!(WidgetType::Button.name(), "Button");
        assert_eq!(WidgetType::ProgressBar.name(), "Progress Bar");
        assert_eq!(WidgetType::TextField.name(), "Text Field");
    }

    #[test]
    fn test_widget_type_icon_present() {
        assert!(!WidgetType::Panel.icon().is_empty());
        assert!(!WidgetType::Button.icon().is_empty());
        assert!(!WidgetType::Image.icon().is_empty());
        assert!(!WidgetType::Grid.icon().is_empty());
    }

    #[test]
    fn test_widget_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for widget in WidgetType::all() {
            assert!(set.insert(*widget));
        }
        assert_eq!(set.len(), 13);
    }

    #[test]
    fn test_widget_type_default_value() {
        assert_eq!(WidgetType::default(), WidgetType::Panel);
    }

    #[test]
    fn test_widget_type_all_unique() {
        let all = WidgetType::all();
        for (i, widget1) in all.iter().enumerate() {
            for (j, widget2) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(widget1, widget2);
                }
            }
        }
    }

    #[test]
    fn test_widget_type_all_have_names() {
        for widget in WidgetType::all() {
            assert!(!widget.name().is_empty());
        }
    }

    #[test]
    fn test_widget_type_all_have_icons() {
        for widget in WidgetType::all() {
            assert!(!widget.icon().is_empty());
        }
    }

    // AnchorPreset tests (8 tests)
    #[test]
    fn test_anchor_preset_display() {
        assert!(format!("{}", AnchorPreset::TopLeft).contains("Top Left"));
        assert!(format!("{}", AnchorPreset::MiddleCenter).contains("Middle Center"));
        assert!(format!("{}", AnchorPreset::BottomRight).contains("Bottom Right"));
        assert!(format!("{}", AnchorPreset::StretchFull).contains("Stretch Full"));
    }

    #[test]
    fn test_anchor_preset_name() {
        assert_eq!(AnchorPreset::TopLeft.name(), "Top Left");
        assert_eq!(AnchorPreset::MiddleCenter.name(), "Middle Center");
        assert_eq!(AnchorPreset::BottomRight.name(), "Bottom Right");
        assert_eq!(AnchorPreset::StretchHorizontal.name(), "Stretch Horizontal");
    }

    #[test]
    fn test_anchor_preset_icon_present() {
        assert!(!AnchorPreset::TopLeft.icon().is_empty());
        assert!(!AnchorPreset::MiddleCenter.icon().is_empty());
        assert!(!AnchorPreset::BottomRight.icon().is_empty());
        assert!(!AnchorPreset::StretchFull.icon().is_empty());
    }

    #[test]
    fn test_anchor_preset_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for anchor in AnchorPreset::all() {
            assert!(set.insert(*anchor));
        }
        assert_eq!(set.len(), 12);
    }

    #[test]
    fn test_anchor_preset_default_value() {
        assert_eq!(AnchorPreset::default(), AnchorPreset::TopLeft);
    }

    #[test]
    fn test_anchor_preset_all_count() {
        assert_eq!(AnchorPreset::all().len(), 12);
    }

    #[test]
    fn test_anchor_preset_all_have_names() {
        for anchor in AnchorPreset::all() {
            assert!(!anchor.name().is_empty());
        }
    }

    #[test]
    fn test_anchor_preset_all_have_icons() {
        for anchor in AnchorPreset::all() {
            assert!(!anchor.icon().is_empty());
        }
    }

    // ScaleMode tests (7 tests)
    #[test]
    fn test_scale_mode_display() {
        assert!(format!("{}", ScaleMode::ConstantPixelSize).contains("Constant Pixel Size"));
        assert!(format!("{}", ScaleMode::ScaleWithScreenSize).contains("Scale With Screen Size"));
        assert!(format!("{}", ScaleMode::ConstantPhysicalSize).contains("Constant Physical Size"));
    }

    #[test]
    fn test_scale_mode_name() {
        assert_eq!(ScaleMode::ConstantPixelSize.name(), "Constant Pixel Size");
        assert_eq!(ScaleMode::ScaleWithScreenSize.name(), "Scale With Screen Size");
        assert_eq!(ScaleMode::ConstantPhysicalSize.name(), "Constant Physical Size");
    }

    #[test]
    fn test_scale_mode_icon_present() {
        assert!(!ScaleMode::ConstantPixelSize.icon().is_empty());
        assert!(!ScaleMode::ScaleWithScreenSize.icon().is_empty());
        assert!(!ScaleMode::ConstantPhysicalSize.icon().is_empty());
    }

    #[test]
    fn test_scale_mode_default_value() {
        assert_eq!(ScaleMode::default(), ScaleMode::ConstantPixelSize);
    }

    #[test]
    fn test_scale_mode_all_count() {
        assert_eq!(ScaleMode::all().len(), 3);
    }

    #[test]
    fn test_scale_mode_all_have_names() {
        for mode in ScaleMode::all() {
            assert!(!mode.name().is_empty());
        }
    }

    #[test]
    fn test_scale_mode_all_have_icons() {
        for mode in ScaleMode::all() {
            assert!(!mode.icon().is_empty());
        }
    }

    // EasingType tests (8 tests)
    #[test]
    fn test_easing_type_display() {
        assert!(format!("{}", EasingType::Linear).contains("Linear"));
        assert!(format!("{}", EasingType::EaseIn).contains("Ease In"));
        assert!(format!("{}", EasingType::EaseOut).contains("Ease Out"));
        assert!(format!("{}", EasingType::Bounce).contains("Bounce"));
    }

    #[test]
    fn test_easing_type_name() {
        assert_eq!(EasingType::Linear.name(), "Linear");
        assert_eq!(EasingType::EaseInOut.name(), "Ease In-Out");
        assert_eq!(EasingType::Elastic.name(), "Elastic");
        assert_eq!(EasingType::Back.name(), "Back");
    }

    #[test]
    fn test_easing_type_icon_present() {
        assert!(!EasingType::Linear.icon().is_empty());
        assert!(!EasingType::EaseIn.icon().is_empty());
        assert!(!EasingType::Bounce.icon().is_empty());
        assert!(!EasingType::Elastic.icon().is_empty());
    }

    #[test]
    fn test_easing_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for easing in EasingType::all() {
            assert!(set.insert(*easing));
        }
        assert_eq!(set.len(), 7);
    }

    #[test]
    fn test_easing_type_default_value() {
        assert_eq!(EasingType::default(), EasingType::Linear);
    }

    #[test]
    fn test_easing_type_all_count() {
        assert_eq!(EasingType::all().len(), 7);
    }

    #[test]
    fn test_easing_type_all_have_names() {
        for easing in EasingType::all() {
            assert!(!easing.name().is_empty());
        }
    }

    #[test]
    fn test_easing_type_all_have_icons() {
        for easing in EasingType::all() {
            assert!(!easing.icon().is_empty());
        }
    }

    // AnimLoopMode tests (7 tests)
    #[test]
    fn test_anim_loop_mode_display() {
        assert!(format!("{}", AnimLoopMode::Once).contains("Once"));
        assert!(format!("{}", AnimLoopMode::Loop).contains("Loop"));
        assert!(format!("{}", AnimLoopMode::PingPong).contains("Ping Pong"));
    }

    #[test]
    fn test_anim_loop_mode_name() {
        assert_eq!(AnimLoopMode::Once.name(), "Once");
        assert_eq!(AnimLoopMode::Loop.name(), "Loop");
        assert_eq!(AnimLoopMode::PingPong.name(), "Ping Pong");
    }

    #[test]
    fn test_anim_loop_mode_icon_present() {
        assert!(!AnimLoopMode::Once.icon().is_empty());
        assert!(!AnimLoopMode::Loop.icon().is_empty());
        assert!(!AnimLoopMode::PingPong.icon().is_empty());
    }

    #[test]
    fn test_anim_loop_mode_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for mode in AnimLoopMode::all() {
            assert!(set.insert(*mode));
        }
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_anim_loop_mode_default_value() {
        assert_eq!(AnimLoopMode::default(), AnimLoopMode::Once);
    }

    #[test]
    fn test_anim_loop_mode_all_have_names() {
        for mode in AnimLoopMode::all() {
            assert!(!mode.name().is_empty());
        }
    }

    #[test]
    fn test_anim_loop_mode_all_have_icons() {
        for mode in AnimLoopMode::all() {
            assert!(!mode.icon().is_empty());
        }
    }

    // UiEditorTab tests (8 tests)
    #[test]
    fn test_ui_editor_tab_display() {
        assert!(format!("{}", UiEditorTab::Hierarchy).contains("Hierarchy"));
        assert!(format!("{}", UiEditorTab::Canvas).contains("Canvas"));
        assert!(format!("{}", UiEditorTab::Widget).contains("Widget"));
        assert!(format!("{}", UiEditorTab::Preview).contains("Preview"));
    }

    #[test]
    fn test_ui_editor_tab_name() {
        assert_eq!(UiEditorTab::Hierarchy.name(), "Hierarchy");
        assert_eq!(UiEditorTab::Canvas.name(), "Canvas");
        assert_eq!(UiEditorTab::Animation.name(), "Animation");
        assert_eq!(UiEditorTab::Presets.name(), "Presets");
    }

    #[test]
    fn test_ui_editor_tab_icon_present() {
        assert!(!UiEditorTab::Hierarchy.icon().is_empty());
        assert!(!UiEditorTab::Widget.icon().is_empty());
        assert!(!UiEditorTab::Style.icon().is_empty());
        assert!(!UiEditorTab::Preview.icon().is_empty());
    }

    #[test]
    fn test_ui_editor_tab_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for tab in UiEditorTab::all() {
            assert!(set.insert(*tab));
        }
        assert_eq!(set.len(), 7);
    }

    #[test]
    fn test_ui_editor_tab_default_value() {
        assert_eq!(UiEditorTab::default(), UiEditorTab::Hierarchy);
    }

    #[test]
    fn test_ui_editor_tab_all_count() {
        assert_eq!(UiEditorTab::all().len(), 7);
    }

    #[test]
    fn test_ui_editor_tab_all_have_names() {
        for tab in UiEditorTab::all() {
            assert!(!tab.name().is_empty());
        }
    }

    #[test]
    fn test_ui_editor_tab_all_have_icons() {
        for tab in UiEditorTab::all() {
            assert!(!tab.icon().is_empty());
        }
    }

    #[test]
    fn test_default_widget() {
        let widget = UiWidget::default();
        assert!(widget.enabled);
        assert!(widget.visible);
        assert!((widget.opacity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_default_style() {
        let style = UiStyle::default();
        assert!((style.font_size_normal - 14.0).abs() < 0.001);
    }

    #[test]
    fn test_panel_trait_implementation() {
        let panel = UiEditorPanel::new();
        assert_eq!(panel.name(), "UI Editor");
    }
}
