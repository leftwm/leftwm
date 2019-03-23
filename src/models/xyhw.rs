#[derive(Clone, Debug, PartialEq, Copy)]
pub struct XYHW{
    pub x: i32,
    pub y: i32,
    pub h: i32,
    pub w: i32,
}

impl Default for XYHW {
    fn default() -> Self {
        XYHW{
            x:0,
            y:0,
            w:0,
            h:0,
        }
    }
}

impl XYHW{
    pub fn contains_point(&self, x:i32 ,y:i32) -> bool{
        let maxx = self.x + self.w;
        let maxy = self.y + self.h;
        (self.x <= x && x <= maxx) && (self.y <= y && y <= maxy)
    }
}
