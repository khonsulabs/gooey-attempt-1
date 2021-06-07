use gooey_core::{
    styles::{BackgroundColor, ColorPair, FallbackComponent, Style, StyleComponent},
    Callback, Context, StyledWidget, Widget,
};

#[cfg(feature = "gooey-rasterizer")]
mod rasterizer;

#[cfg(feature = "frontend-browser")]
mod browser;

#[derive(Debug)]
pub struct Button {
    pub label: String,
    pub clicked: Callback,
}

impl Button {
    pub fn new<S: ToString>(label: S, clicked: Callback) -> StyledWidget<Self> {
        StyledWidget::default_for(Self {
            label: label.to_string(),
            clicked,
        })
    }
}

#[derive(Debug)]
pub enum InternalButtonEvent {
    Clicked,
}

#[derive(Debug)]
pub enum ButtonCommand {
    SetLabel(String),
}

impl Widget for Button {
    type Command = ButtonCommand;
    type TransmogrifierCommand = ButtonCommand;
    type TransmogrifierEvent = InternalButtonEvent;

    const CLASS: &'static str = "gooey-button";

    fn receive_event(
        &mut self,
        event: Self::TransmogrifierEvent,
        _context: &gooey_core::Context<Self>,
    ) {
        let InternalButtonEvent::Clicked = event;
        self.clicked.invoke(());
    }

    /// Called when an `event` from the transmogrifier was received.
    #[allow(unused_variables)]
    fn receive_command(&mut self, command: Self::Command, context: &Context<Self>) {
        match &command {
            ButtonCommand::SetLabel(label) => {
                self.label = label.clone();
            }
        }

        context.send_command(command);
    }

    fn background_color(style: &Style) -> Option<&'_ ColorPair> {
        style.get_with_fallback::<ButtonColor>()
    }
}

#[derive(Debug)]
pub struct ButtonTransmogrifier;

/// The button's background color.
#[derive(Debug, Clone)]
pub struct ButtonColor(pub ColorPair);
impl StyleComponent for ButtonColor {}

impl From<ButtonColor> for ColorPair {
    fn from(color: ButtonColor) -> Self {
        color.0
    }
}

impl FallbackComponent for ButtonColor {
    type Fallback = BackgroundColor;
    type Value = ColorPair;

    fn value(&self) -> Option<&ColorPair> {
        Some(&self.0)
    }
}
