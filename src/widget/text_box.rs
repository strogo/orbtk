use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    event::{Key, KeyEventHandler, MouseEventHandler},
    properties::{ ScrollMode,
        Bounds, Constraint, Focused, FocusedProperty, Offset, Point, ScrollViewerMode, Text,
        TextProperty, TextSelection, WaterMark, PaddingProperty, WaterMarkProperty
    },
    structs::{Position, Size},
    theme::Selector,
    widget::{Container, Context, Grid, SharedProperty, State, Template, TextBlock, Widget, WaterMarkTextBlock},
};

/// The `TextBoxState` handles the text processing of the `TextBox` widget.
#[derive(Default)]
pub struct TextBoxState {
    text: RefCell<String>,
    focused: Cell<bool>,
    updated: Cell<bool>,
    selection_start: Cell<usize>,
    selection_length: Cell<usize>,
    cursor_x: Cell<f64>,
}

impl Into<Rc<dyn State>> for TextBoxState {
    fn into(self) -> Rc<dyn State> {
        Rc::new(self)
    }
}

impl TextBoxState {
    fn click(&self, point: Point) {
        println!("Clicked text box point: ({}, {})", point.x, point.y);
    }

    fn update_selection_start(&self, selection: i32) {
        self.selection_start
            .set(selection.max(0).min(self.text.borrow().len() as i32) as usize);
    }

    fn update_text(&self, key: Key) -> bool {
        if !self.focused.get() {
            return false;
        }

        match <Option<u8>>::from(key) {
            Some(byte) => {
                (*self.text.borrow_mut()).insert(self.selection_start.get(), byte as char);
                self.update_selection_start(self.selection_start.get() as i32 + 1);
            }
            None => match key {
                Key::Left => {
                    self.update_selection_start(self.selection_start.get() as i32 - 1);
                    self.selection_length.set(0);
                }
                Key::Right => {
                    self.update_selection_start(self.selection_start.get() as i32 + 1);
                    self.selection_length.set(0);
                }
                Key::Backspace => {
                    if self.text.borrow().len() > 0 {
                        if self.selection_start.get() > 0 {
                            for _ in 0..(self.selection_length.get() + 1) {
                                (*self.text.borrow_mut()).remove(self.selection_start.get() - 1);
                            }
                            self.update_selection_start(self.selection_start.get() as i32 - 1);
                        }
                    }
                }
                _ => {}
            },
        }

        self.updated.set(true);

        true
    }
}

impl State for TextBoxState {
    fn update(&self, context: &mut Context<'_>) {
        let mut widget = context.widget();

        if let Ok(focused) = widget.borrow_property::<Focused>() {
            self.focused.set(focused.0);
        }

        if let Ok(text) = widget.borrow_mut_property::<Text>() {
            if text.0 != *self.text.borrow() {
                if self.updated.get() {
                    text.0 = self.text.borrow().clone();
                } else {
                    let text_length = self.text.borrow().len();
                    let origin_text_length = text.0.len();
                    let delta = text_length as i32 - origin_text_length as i32;

                    *self.text.borrow_mut() = text.0.clone();

                    // adjust cursor position after labe is changed from outside
                    if text_length < origin_text_length {
                        self.update_selection_start(self.selection_start.get() as i32 - delta);
                    } else {
                        self.update_selection_start(self.selection_start.get() as i32 + delta);
                    }
                }

                self.updated.set(false);
            }
        }

        if let Ok(selection) = widget.borrow_mut_property::<TextSelection>() {
            selection.start_index = self.selection_start.get();
            selection.length = self.selection_length.get();
        }
    }

    fn update_post_layout(&self, context: &mut Context<'_>) {
        let mut cursor_x_delta = 0.0;
        let mut scroll_viewer_width = 0.0;

        {
            let scroll_viewer = context.widget_from_id("TextBoxScrollViewer");

            if let Ok(bounds) = scroll_viewer.unwrap().borrow_property::<Bounds>() {
                scroll_viewer_width = bounds.width();
            }
        }

        // maybe not use scrollviewer here

        // Adjust offset of text and cursor if cursor position is out of bounds

        {
            // let cursor = context.widget_from_id("TextBoxCursor");

            // if let Ok(bounds) = cursor.unwrap().borrow_mut_property::<Bounds>() {
            //     if bounds.x() < 0.0 || bounds.x() > scroll_viewer_width {
            //         cursor_x_delta = self.cursor_x.get() - bounds.x();
            //         bounds.set_x(self.cursor_x.get());
            //     }
            //     self.cursor_x.set(bounds.x());
            // }
        }

        if cursor_x_delta != 0.0 {
            {
                let text_block = context.widget_from_id("TextBoxTextBlock");

                if let Ok(bounds) = text_block.unwrap().borrow_mut_property::<Bounds>() {
                    bounds.set_x(bounds.x() + cursor_x_delta);
                }
            }

            if let Ok(offset) = context.widget().borrow_mut_property::<Offset>() {
                offset.0 += cursor_x_delta;
            }
        }
    }
}

/// The `TextBox` represents a single line text input widget.
///
/// # Shared Properties
///
/// * `Text` - String used to display the text of the text box.
/// * `Watermark` - String used to display a placeholder text if `Text` string is empty.
/// * `Selector` - CSS selector with  element name `textbox`, used to request the theme of the widget.
/// * `TextSelection` - Represents the current selection of the text used by the cursor.
/// * `Focused` - Defines if the widget is focues and handles the current text input.
///
/// # Others
///
/// * `TextBoxState` - Handles the inner state of the widget.
/// * `KeyEventHandler` - Process the text input of the control if it is focuesd.
pub struct TextBox;

impl Widget for TextBox {
    type Template = TextBoxTemplate;

    fn create() -> Self::Template {
        let text = SharedProperty::new(Text::default());
        let water_mark = SharedProperty::new(WaterMark::default());
        let selector = Selector::from("textbox");
        let selection = SharedProperty::new(TextSelection::default());
        let offset = SharedProperty::new(Offset::default());
        let focused = SharedProperty::new(Focused(false));
        let state = Rc::new(TextBoxState::default());
        let click_state = state.clone();

        TextBoxTemplate::new()
            .constraint(Constraint::create().size(128.0, 32.0).build())
            .state(state.clone())
            .debug_name("TextBox")
            .child(
                Container::create()
                    .padding((8.0, 2.0))
                    .child(
                        Grid::create()
                            .child(
                                Grid::create()
                                    .child(
                                        WaterMarkTextBlock::create()
                                            .vertical_alignment("Center")
                                            .shared_text(text.clone())
                                            .shared_water_mark(water_mark.clone())
                                            .attach_shared_property(focused.clone())
                                            .selector(selector.clone().id("TextBoxTextBlock")),
                                    )
                                    // .shared_property(offset.clone())
                                    // .property(ScrollViewerMode::new(
                                    //     ScrollMode::None,
                                    //     ScrollMode::None,
                                    // ))
                                    .selector(Selector::from("scrollviewer").id("TextBoxScrollViewer")),
                            )
                            // .child(
                            //     Cursor::create()
                            //         .shared_property(text.clone())
                            //         .shared_property(selection.clone())
                            //         .shared_property(offset.clone())
                            //         .shared_property(focused.clone())
                            //         .property(Selector::from("cursor").id("TextBoxCursor")),
                            // )
                            // .event_handler(MouseEventHandler::default().on_mouse_down(Rc::new(
                            //     move |pos: Point| -> bool {
                            //         click_state.click(pos);
                            //         false
                            //     },
                            // ))),
                    )
                    .attach_shared_property(focused.clone())
                    .attach_property(selector.clone()),
            )
            .shared_text(text)
            .selector(selector)
            .shared_water_mark(water_mark)
            // .shared_property(selection)
            .attach_property(offset)
            .shared_focused(focused)
            .event_handler(
                KeyEventHandler::default()
                    .on_key_down(Rc::new(move |key: Key| -> bool { state.update_text(key) })),
            )
    }
}

template!(TextBoxTemplate, [TextProperty, FocusedProperty, WaterMarkProperty]);
