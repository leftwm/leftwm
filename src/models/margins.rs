use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Margins {
    Int(u32),
    Vec(Vec<u32>),
} // format: [top, right, bottom, left] as per HTML

impl Margins {
    pub fn left(self) -> i32 {
        self.into_vec()[3] as i32
    }
    pub fn right(self) -> i32 {
        self.into_vec()[1] as i32
    }
    pub fn top(self) -> i32 {
        self.into_vec()[0] as i32
    }
    pub fn bottom(self) -> i32 {
        self.into_vec()[2] as i32
    }

    pub fn into_vec(self) -> Vec<u32> {
        match self {
            Self::Vec(v) => match v.len() {
                1 => vec![v[0], v[0], v[0], v[0]],
                2 => vec![v[0], v[0], v[1], v[1]],
                3 => vec![v[0], v[1], v[2], v[2]],
                4 => v,
                0 => {
                    log::error!("Empty margin or border array");
                    vec![10, 10, 10, 10] //assume 5 px borders for now
                }
                _ => {
                    log::error!("Too many entries in margin or border array");
                    vec![v[0], v[1], v[2], v[3]] //assume later entries are invalid
                }
            },
            Self::Int(x) => vec![x, x, x, x],
        }
    }
}
