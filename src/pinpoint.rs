use iced_core::{event, mouse, overlay, Clipboard, Event, Layout, Length, Point, Rectangle, Shell};
use iced_core::{widget::Tree, Element, Widget};

use crate::overlay::Anchor;
use crate::overlay::FloatingElementOverlay;
use crate::overlay::Offset;

#[allow(missing_debug_implementations)]
pub struct Pinpoint<'a, B, Message, Renderer>
where
    B: Fn(usize) -> Element<'a, Message, Renderer>,
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    /// The anchor of the element.
    anchor: Anchor,
    /// The offset of the element.
    offset: Vec<Offset>,
    /// The visibility of the element.
    hidden: bool,
    /// The underlying element.
    underlay: Element<'a, Message, Renderer>,
    /// The floating element of the [`FloatingElementOverlay`](FloatingElementOverlay).
    element: Vec<B>,
}

impl<'a, B, Message, Renderer> Pinpoint<'a, B, Message, Renderer>
where
    B: Fn(usize) -> Element<'a, Message, Renderer>,
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    pub fn new<U>(underlay: U, element: Vec<B>, offset: Vec<Offset>) -> Self
    where
        U: Into<Element<'a, Message, Renderer>>,
    {
        Pinpoint {
            anchor: Anchor::Center,
            offset,
            hidden: false,
            underlay: underlay.into(),
            element,
        }
    }

    /// Sets the [`Anchor`](Anchor) of the [`FloatingElement`](FloatingElement).
    #[must_use]
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Hide or unhide the [`Element`](iced_core::Element) on the
    /// [`FloatingElement`](FloatingElement).
    #[must_use]
    pub fn hide(mut self, hide: bool) -> Self {
        self.hidden = hide;
        self
    }
}

impl<'a, B, Message, Renderer> Widget<Message, Renderer> for Pinpoint<'a, B, Message, Renderer>
where
    B: Fn(usize) -> Element<'a, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer,
{
    fn children(&self) -> Vec<iced_core::widget::Tree> {
        let mut elements = vec![Tree::new(&self.underlay)];
        for (index, value) in self.element.iter().enumerate() {
            elements.push(Tree::new(&(value)(index)));
        }
        elements
    }

    fn diff(&self, tree: &mut Tree) {
        let mut elements = vec![&self.underlay];
        let mut store = vec![];
        for (index, value) in self.element.iter().enumerate() {
            store.push((value)(index));
        }
        for i in store.iter() {
            elements.push(&i);
        }
        tree.diff_children(&elements.as_slice());
    }

    fn width(&self) -> Length {
        self.underlay.as_widget().width()
    }

    fn height(&self) -> Length {
        self.underlay.as_widget().height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        self.underlay.as_widget().layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.underlay.as_widget_mut().on_event(
            &mut state.children[0],
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.underlay.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        state: &iced_core::widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &iced_core::renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.underlay.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        if self.hidden {
            return self
                .underlay
                .as_widget_mut()
                .overlay(&mut state.children[0], layout, renderer);
        }

        let bounds = layout.bounds();

        let position = match self.anchor {
            Anchor::NorthWest => Point::new(0.0, 0.0),
            Anchor::NorthEast => Point::new(bounds.width, 0.0),
            Anchor::SouthWest => Point::new(0.0, bounds.height),
            Anchor::SouthEast => Point::new(bounds.width, bounds.height),
            Anchor::North => Point::new(bounds.center_x(), 0.0),
            Anchor::East => Point::new(bounds.width, bounds.center_y()),
            Anchor::South => Point::new(bounds.center_x(), bounds.height),
            Anchor::West => Point::new(0.0, bounds.center_y()),
            Anchor::Center => Point::new(bounds.center_x(), bounds.center_y()),
        };

        let position = Point::new(bounds.x + position.x, bounds.y + position.y);

        let mut group = iced_core::overlay::Group::new();

        for (index, value) in state.children.iter_mut().enumerate() {
            if index != 0 {
                group = group.push(
                    FloatingElementOverlay::new(
                        value,
                        (self.element[index - 1])(index - 1),
                        &self.anchor,
                        &self.offset[index - 1],
                    )
                    .overlay(position),
                );
            }
        }
        Some(group.into())
    }
}

impl<'a, B, Message, Renderer> From<Pinpoint<'a, B, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    B: 'a + Fn(usize) -> Element<'a, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer,
{
    fn from(floating_element: Pinpoint<'a, B, Message, Renderer>) -> Self {
        Element::new(floating_element)
    }
}

// pub fn pack_closure<'a>(
//     ret: Element<'a, crate::Message, iced::Renderer>,
// ) -> Element<'a, crate::Message, iced::Renderer> {
//     return ret;
// }
