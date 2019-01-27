use super::screen::Screen;

#[derive(Debug, Clone)]
pub struct Workspace{
    pub height :i32,
    pub width :i32,
}

impl Workspace{

    pub fn new(height:i32, width:i32) -> Workspace{
        Workspace{
            height:height,
            width:width,
        }
    }

    pub fn from_screen(screen: Screen) -> Workspace{
        Workspace{
            height:screen.height,
            width:screen.width,
        }
    }

}

