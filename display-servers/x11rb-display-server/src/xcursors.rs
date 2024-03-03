use crate::error::Result;
use x11rb::{
    cursor::Handle as CursorHandle, protocol::xproto, resource_manager::Database,
    rust_connection::RustConnection,
};

#[derive(Clone, Debug)]
pub struct XCursor {
    pub normal: xproto::Cursor,
    pub resize: xproto::Cursor,
    pub move_: xproto::Cursor,
}

// Cursors
const CURSOR_NORMAL: &str = "left_ptr";
const CURSOR_RESIZE: &str = "se-resize";
const CURSOR_MOVE: &str = "fleur";

impl XCursor {
    pub(crate) fn new(conn: &RustConnection, display: usize, db: &Database) -> Result<Self> {
        let handle = CursorHandle::new(conn, display, db)?.reply()?;
        Ok(Self {
            normal: handle.load_cursor(conn, CURSOR_NORMAL)?,
            resize: handle.load_cursor(conn, CURSOR_RESIZE)?,
            move_: handle.load_cursor(conn, CURSOR_MOVE)?,
        })
    }
}
