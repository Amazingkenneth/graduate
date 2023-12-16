use iced_core::{
    event, layout::Limits, mouse, overlay, Clipboard, Event, Layout, Length, Point, Rectangle,
    Shell, Size,
};
use iced_core::{widget::Operation, widget::Tree, Element, Widget};

/// The internal overlay of a [`FloatingElement`](crate::FloatingElement) for
/// rendering a [`Element`](iced_core::Element) as an overlay.
#[allow(missing_debug_implementations)]
pub struct FloatingElementOverlay<'a, Message: Clone, Renderer: iced_core::Renderer> {
    /// The state of the element.
    state: &'a mut Tree,
    /// The floating element
    element: Element<'a, Message, Renderer>,
    /// The anchor of the element.
    anchor: &'a Anchor,
    /// The offset of the element.
    offset: &'a Offset,
}

impl<'a, Message, Renderer> FloatingElementOverlay<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    /// Creates a new [`FloatingElementOverlay`] containing the given
    /// [`Element`](iced_core::Element).
    pub fn new<B>(state: &'a mut Tree, element: B, anchor: &'a Anchor, offset: &'a Offset) -> Self
    where
        B: Into<Element<'a, Message, Renderer>>,
    {
        FloatingElementOverlay {
            state,
            element: element.into(),
            anchor,
            offset,
        }
    }

    /// Turns the [`FloatingElementOverlay`](FloatingElementOverlay) into an
    /// overlay [`Element`](iced_core::overlay::Element) at the given target
    /// position.
    #[must_use]
    pub fn overlay(self, position: Point) -> overlay::Element<'a, Message, Renderer> {
        overlay::Element::new(position, Box::new(self))
    }
}

impl<'a, Message, Renderer> iced_core::Overlay<Message, Renderer>
    for FloatingElementOverlay<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
        _translation: iced_core::Vector,
    ) -> iced_core::layout::Node {
        let limits = Limits::new(Size::ZERO, bounds);
        let mut element = self
            .element
            .as_widget()
            .layout(self.state, renderer, &limits);
        let fixed_offset_x = self.offset.x / 1500.0 * bounds.width;
        let fixed_offset_y = self.offset.y / 900.0 * bounds.height;

        match self.anchor {
            Anchor::NorthWest => element.move_to(Point::new(
                position.x + fixed_offset_x,
                position.y + fixed_offset_y,
            )),
            Anchor::NorthEast => element.move_to(Point::new(
                position.x - element.bounds().width - fixed_offset_x,
                position.y + fixed_offset_y,
            )),
            Anchor::SouthWest => element.move_to(Point::new(
                position.x + fixed_offset_x,
                position.y - element.bounds().height - fixed_offset_y,
            )),
            Anchor::SouthEast => element.move_to(Point::new(
                position.x - element.bounds().width - fixed_offset_x,
                position.y - element.bounds().height - fixed_offset_y,
            )),
            Anchor::North => element.move_to(Point::new(
                position.x + fixed_offset_x - element.bounds().width / 2.0,
                position.y + fixed_offset_y,
            )),
            Anchor::East => element.move_to(Point::new(
                position.x - element.bounds().width - fixed_offset_x,
                position.y - element.bounds().height / 2.0,
            )),
            Anchor::South => element.move_to(Point::new(
                position.x + fixed_offset_x - element.bounds().width / 2.0,
                position.y - element.bounds().height - fixed_offset_y,
            )),
            Anchor::West => element.move_to(Point::new(
                position.x + fixed_offset_x,
                position.y - element.bounds().height / 2.0,
            )),
            Anchor::Center => element.move_to(Point::new(
                position.x + fixed_offset_x - element.bounds().width / 2.0,
                position.y + fixed_offset_y - element.bounds().height / 2.0,
            )),
        }

        element
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: iced_core::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();

        self.element.as_widget_mut().on_event(
            self.state,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
            &bounds,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
        renderer: &Renderer,
    ) -> iced_core::mouse::Interaction {
        self.element.as_widget().mouse_interaction(
            self.state,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &iced_core::renderer::Style,
        layout: Layout<'_>,
        cursor_position: iced_core::mouse::Cursor,
    ) {
        self.element.as_widget().draw(
            self.state,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            &layout.bounds(),
        );
    }
}

/// The [`Offset`](Offset) for the [`FloatingButton`](super::FloatingButton).
#[derive(Copy, Clone, Debug)]
pub struct Offset {
    /// Offset on the x-axis from the [`Anchor`](super::Anchor)
    pub x: f32,
    /// Offset on the y-axis from the [`Anchor`](super::Anchor)
    pub y: f32,
}

impl From<f32> for Offset {
    fn from(float: f32) -> Self {
        Self { x: float, y: float }
    }
}

impl From<[f32; 2]> for Offset {
    fn from(array: [f32; 2]) -> Self {
        Self {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Offset> for Point {
    fn from(offset: Offset) -> Self {
        Self::new(offset.x, offset.y)
    }
}

impl From<&Offset> for Point {
    fn from(offset: &Offset) -> Self {
        Self::new(offset.x, offset.y)
    }
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum Anchor {
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
    North,
    East,
    South,
    West,
    Center,
}

#[allow(missing_debug_implementations)]
pub struct Component<'a, B, Message, Renderer>
where
    B: Fn() -> Element<'a, Message, Renderer>,
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    /// The anchor of the element.
    anchor: Anchor,
    /// The offset of the element.
    offset: Offset,
    /// The visibility of the element.
    hidden: bool,
    /// The underlying element.
    underlay: Element<'a, Message, Renderer>,
    /// The floating element of the [`FloatingElementOverlay`](FloatingElementOverlay).
    element: B,
}

impl<'a, B, Message, Renderer> Component<'a, B, Message, Renderer>
where
    B: Fn() -> Element<'a, Message, Renderer>,
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    /// Creates a new [`FloatingElement`](FloatingElement) over some content,
    /// showing the given [`Element`](iced_core::Element).
    ///
    /// It expects:
    ///     * the underlay [`Element`](iced_core::Element) on which this [`FloatingElement`](FloatingElement)
    ///         will be wrapped around.
    ///     * a function that will lazy create the [`Element`](iced_core::Element) for the overlay.
    pub fn new<U>(underlay: U, element: B) -> Self
    where
        U: Into<Element<'a, Message, Renderer>>,
    {
        Component {
            anchor: Anchor::SouthWest,
            offset: 5.0.into(),
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

    /// Sets the [`Offset`](Offset) of the [`FloatingElement`](FloatingElement).
    #[must_use]
    pub fn offset<O>(mut self, offset: O) -> Self
    where
        O: Into<Offset>,
    {
        self.offset = offset.into();
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

impl<'a, B, Message, Renderer> Widget<Message, Renderer> for Component<'a, B, Message, Renderer>
where
    B: Fn() -> Element<'a, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer,
{
    fn children(&self) -> Vec<iced_core::widget::Tree> {
        vec![Tree::new(&self.underlay), Tree::new(&(self.element)())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.underlay, &(self.element)()]);
    }

    fn width(&self) -> Length {
        self.underlay.as_widget().width()
    }

    fn height(&self) -> Length {
        self.underlay.as_widget().height()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        self.underlay.as_widget().layout(tree, renderer, limits)
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: iced_core::mouse::Cursor,
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
        cursor_position: iced_core::mouse::Cursor,
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
        cursor_position: iced_core::mouse::Cursor,
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

    fn operate<'b>(
        &'b self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        self.underlay
            .as_widget()
            .operate(&mut state.children[0], layout, renderer, operation);
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

        if state.children.len() == 2 {
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

            Some(
                FloatingElementOverlay::new(
                    &mut state.children[1],
                    (self.element)(),
                    &self.anchor,
                    &self.offset,
                )
                .overlay(position),
            )
        } else {
            None
        }
    }
}

impl<'a, B, Message, Renderer> From<Component<'a, B, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    B: 'a + Fn() -> Element<'a, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer,
{
    fn from(floating_element: Component<'a, B, Message, Renderer>) -> Self {
        Element::new(floating_element)
    }
}
