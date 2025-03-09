use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy)]
pub struct FrameInfo {
    pub popup: Option<Rect>,
    pub status_bar: Rect,
    pub scroll_bar: Rect,
    pub address_view: Rect,
    pub hex_view: Option<Rect>,
    pub info_view: Option<Rect>,
    pub info_view_frame_info: InfoViewFrameInfo,
    pub blocks_per_row: usize,
    pub scroll: usize,
    pub file_size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum InfoViewFrameInfo {
    TextView,
    AssemblyView { scroll: usize },
}
