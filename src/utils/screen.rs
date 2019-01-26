
#[derive(Debug, Clone)]
pub struct Screen{
    pub height :i32,
    pub width :i32,
}

impl Screen{

    pub fn new(height:i32, width:i32) -> Screen{
        Screen{
            height:height,
            width:width,
        }
    }

}

