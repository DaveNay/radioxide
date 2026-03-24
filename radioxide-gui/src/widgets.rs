use iced::widget::{button, column, row, text};
use iced::{Element, Length};

use crate::styles::radio_button;

/// Build a grid of toggle buttons from a slice of items.
/// The currently selected item is highlighted. Each button sends `on_select(item)`.
pub fn toggle_button_row<'a, T, M>(
    items: &[T],
    selected: T,
    on_select: impl Fn(T) -> M + 'a,
    columns: usize,
) -> Element<'a, M>
where
    T: Copy + Eq + std::fmt::Display + 'a,
    M: Clone + 'a,
{
    let rows: Vec<Element<'a, M>> = items
        .chunks(columns)
        .map(|chunk| {
            let buttons: Vec<Element<'a, M>> = chunk
                .iter()
                .map(|&item| {
                    let is_selected = item == selected;
                    button(text(item.to_string()).size(13))
                        .on_press(on_select(item))
                        .style(radio_button(is_selected))
                        .width(Length::Fixed(60.0))
                        .padding([4, 6])
                        .into()
                })
                .collect();
            row(buttons).spacing(4).into()
        })
        .collect();

    column(rows).spacing(4).into()
}
