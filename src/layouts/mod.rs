use super::models::Window;
use super::models::Workspace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Layout {
    MainAndVertStack,
    GridHorizontal,
    EvenHorizontal,
    EvenVertical,
    Fibonacci,
    CenterMain,
}

impl Default for Layout {
    fn default() -> Self {
        Self::MainAndVertStack
    }
}

// This is tedious, but simple and effective.
impl Layout {
    pub fn next_layout(&self) -> Self {
        match self {
            Self::MainAndVertStack => Self::GridHorizontal,
            Self::GridHorizontal => Self::EvenHorizontal,
            Self::EvenHorizontal => Self::EvenVertical,
            Self::EvenVertical => Self::Fibonacci,
            Self::Fibonacci => Self::CenterMain,
            Self::CenterMain => Self::MainAndVertStack,
        }
    }

    pub fn prev_layout(&self) -> Self {
        match self {
            Self::MainAndVertStack => Self::CenterMain,
            Self::GridHorizontal => Self::MainAndVertStack,
            Self::EvenHorizontal => Self::GridHorizontal,
            Self::EvenVertical => Self::EvenHorizontal,
            Self::Fibonacci => Self::EvenVertical,
            Self::CenterMain => Self::Fibonacci,
        }
    }

    pub fn update_windows(&self, workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
        match self {
            Self::MainAndVertStack => main_and_vert_stack(workspace, windows),
            Self::GridHorizontal => grid_horizontal(workspace, windows),
            Self::EvenHorizontal => even_horizontal(workspace, windows),
            Self::EvenVertical => even_vertical(workspace, windows),
            Self::Fibonacci => fibonacci(workspace, windows),
            Self::CenterMain => center_main(workspace, windows),
        }
    }
}

/// Layout which splits the workspace into two columns, gives one window all of the left column,
/// and divides the right column among all the other windows.
fn main_and_vert_stack(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len();
    if window_count == 0 {
        return;
    }

    let width = match window_count {
        1 => workspace.width() as i32,
        _ => (workspace.width() as f32 / 2 as f32).floor() as i32,
    };

    //build build the main window.
    let mut iter = windows.iter_mut();
    {
        if let Some(first) = iter.next() {
            first.set_height(workspace.height());
            first.set_width(width);
            first.set_x(workspace.x());
            first.set_y(workspace.y());
        }
    }

    //stack all the others
    let height_f = workspace.height() as f32 / (window_count - 1) as f32;
    let height = height_f.floor() as i32;
    let mut y = 0;
    for w in iter {
        w.set_height(height);
        w.set_width(width);
        w.set_x(workspace.x() + width);
        w.set_y(workspace.y() + y);
        y += height;
    }
}

/// Layout which splits the workspace into N columns, and then splits each column into rows.
/// Example arrangement (4 windows):
/// ```text
/// +---+---+
/// |   |   |
/// +---+---+
/// |   |   |
/// +---+---+
/// ```
/// or with 8 windows:
/// ```text
/// +---+---+---+
/// |   |   |   |
/// |   +---+---+
/// +---+   |   |
/// |   +---+---+
/// |   |   |   |
/// +---+---+---+
/// ```
fn grid_horizontal(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len() as i32;

    // choose the number of columns so that we get close to an even NxN grid.
    let num_cols = (window_count as f32).sqrt().ceil() as i32;

    let mut iter = windows.iter_mut().enumerate().peekable();
    for col in 0..num_cols {
        let remaining_windows = window_count - iter.peek().unwrap().0 as i32;
        let remaining_columns = num_cols - col;
        let num_rows_in_this_col = remaining_windows / remaining_columns;

        let win_height = workspace.height() / num_rows_in_this_col;
        let win_width = workspace.width() / num_cols;

        for row in 0..num_rows_in_this_col {
            let (_idx, win) = iter.next().unwrap();
            win.set_height(win_height);
            win.set_width(win_width);
            win.set_x(workspace.x() + win_width * col);
            win.set_y(workspace.y() + win_height * row);
        }
    }
}

/// Layout which gives each window full height, but splits the workspace width among them all.
fn even_horizontal(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let width_f = workspace.width() as f32 / windows.len() as f32;
    let width = width_f.floor() as i32;
    let mut x = 0;
    for w in windows.iter_mut() {
        w.set_height(workspace.height());
        w.set_width(width);
        w.set_x(workspace.x() + x);
        w.set_y(workspace.y());
        x += width;
    }
}

/// Layout which gives each window full width, but splits the workspace height among them all.
fn even_vertical(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let height_f = workspace.height() as f32 / windows.len() as f32;
    let height = height_f.floor() as i32;
    let mut y = 0;
    for w in windows.iter_mut() {
        w.set_height(height);
        w.set_width(workspace.width());
        w.set_x(workspace.x());
        w.set_y(workspace.y() + y);
        y += height;
    }
}

/// Fibonacci layout, which divides the workspace in subsequent halves and assignes them to the windows
/// ```text
/// +-----------+-----------+
/// |           |           |
/// |           |     2     |
/// |           |           |
/// |     1     +-----+-----+
/// |           |     |  4  |
/// |           |  3  +--+--+
/// |           |     | 5|-.|
/// +-----------+-----+-----+
/// ```
fn fibonacci(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let mut x = workspace.x();
    let mut y = workspace.y();
    let mut height = workspace.height() as i32;
    let mut width = workspace.width() as i32;
    let window_count = windows.len() as i32;

    for i in 0..window_count {
        if i % 2 != 0 {
            continue;
        }

        let half_width = (width as f32 / 2 as f32).floor() as i32;
        let half_height = (height as f32 / 2 as f32).floor() as i32;

        match window_count - 1 - i {
            0 => {
                setter(&mut windows[i as usize], height, width, x, y);
            }
            1 => {
                setter(&mut windows[i as usize], height, half_width, x, y);

                setter(
                    &mut windows[(i + 1) as usize],
                    height,
                    half_width,
                    x + half_width,
                    y,
                );
            }
            _ => {
                setter(&mut windows[i as usize], height, half_width, x, y);

                setter(
                    &mut windows[(i + 1) as usize],
                    half_height,
                    half_width,
                    x + half_width,
                    y,
                );

                x = x + half_width;
                y = y + half_height;
                width = half_width;
                height = half_height;
            }
        }
    }
}

/// Layout which splits the workspace into three columns.
/// Gives first window all of the center column.
/// Gives second window all of the left column.
/// Divides the right column among all the other windows.
///
/// 1 window
/// ```text
/// +-----------------------+
/// |                       |
/// |                       |
/// |           1           |
/// |                       |
/// |                       |
/// +-----------------------+
/// ```
/// 2 windows
/// ```text
/// +-----------+-----------+
/// |           |           |
/// |           |           |
/// |      2    |     1     |
/// |           |           |
/// |           |           |
/// +-----------+-----------+
/// ```
/// 3 windows
/// ```text
/// +-----+-----------+-----+
/// |     |           |     |
/// |     |           |     |
/// |  2  |     1     |  3  |
/// |     |           |     |
/// |     |           |     |
/// +-----+-----------+-----+
/// ```
/// 4 windows
/// ```text
/// +-----+-----------+-----+
/// |     |           |  3  |
/// |     |           |     |
/// |  2  |     1     +-----+
/// |     |           |  4  |
/// |     |           |     |
/// +-----+-----------+-----+
/// ```
/// 5 windows
/// ```text
/// +-----+-----------+-----+
/// |     |           |  3  |
/// |     |           +-----+
/// |  2  |     1     |  4  |
/// |     |           +-----+
/// |     |           |  5  |
/// +-----+-----------+-----+
/// ```
fn center_main(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len();

    if window_count == 0 {
        return;
    }

    let primary_width = match window_count {
        1 => workspace.width() as i32,
        _ => (workspace.width() as f32 / 2.0).floor() as i32,
    };

    let primary_x = match window_count {
        1 => 0 as i32,
        2 => (workspace.width() as f32 / 2.0).floor() as i32,
        _ => (workspace.width() as f32 / 4.0).floor() as i32,
    };

    let mut iter = windows.iter_mut();

    // build the primary window
    {
        if let Some(first) = iter.next() {
            first.set_height(workspace.height());
            first.set_width(primary_width);
            first.set_x(workspace.x() + primary_x);
            first.set_y(workspace.y());
        }
    }

    let secondary_width = match window_count {
        1 => 0,
        2 => (workspace.width() as f32 / 2.0).floor() as i32,
        _ => (workspace.width() as f32 / 4.0).floor() as i32,
    };

    // build the second window
    {
        if let Some(second) = iter.next() {
            second.set_height(workspace.height());
            second.set_width(secondary_width);
            second.set_x(workspace.x());
            second.set_y(workspace.y());
        }
    }

    // stack all the others
    if window_count > 2 {
        let height_f = workspace.height() as f32 / (window_count - 2) as f32;
        let height = height_f.floor() as i32;
        let mut y = 0;

        for w in iter {
            w.set_height(height);
            w.set_width(secondary_width);
            w.set_x(workspace.x() + primary_width + secondary_width);
            w.set_y(workspace.y() + y);
            y += height;
        }
    }
}

fn setter(window: &mut Window, height: i32, width: i32, x: i32, y: i32) {
    window.set_height(height);
    window.set_width(width);
    window.set_x(x);
    window.set_y(y);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BBox, WindowHandle};

    #[test]
    fn should_fullscreen_a_single_window() {
        //size defaults to 600x800
        let mut ws = Workspace::new(BBox {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
        });
        ws.xyhw.set_minh(600);
        ws.xyhw.set_minw(800);
        ws.update_avoided_areas();
        let mut w = Window::new(WindowHandle::MockHandle(1), None);
        w.border = 0;
        w.margin = 0;
        let mut windows = vec![&mut w];
        let mut windows_filters = windows.iter_mut().filter(|_f| true).collect();
        even_horizontal(&ws, &mut windows_filters);
        assert!(
            w.height() == 600,
            "window was not size to the correct height"
        );
        assert!(w.width() == 800, "window was not size to the correct width");
    }
}
