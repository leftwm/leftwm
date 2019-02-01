use super::screen::Screen;

#[derive(Debug, Clone)]
pub struct Workspace{
    pub height :i32,
    pub width :i32,
    pub x :i32,
    pub y :i32,
}

impl Workspace{

    pub fn from_screen(screen: Screen) -> Workspace{
        Workspace{
            height:screen.height,
            width:screen.width,
            x:0,
            y:0,
        }
    }

}

