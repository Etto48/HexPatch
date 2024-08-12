#[derive(Clone, Copy, Debug)]
pub struct RectBorders {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl RectBorders {
    pub fn any(&self) -> bool {
        self.top || self.bottom || self.left || self.right
    }
}
