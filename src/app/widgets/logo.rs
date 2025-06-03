use ratatui::{
    style::{Color, Style},
    widgets::Widget,
};

pub struct Logo {
    colors: Vec<Style>,
    matrix: Vec<Vec<usize>>,
}

impl Logo {
    pub fn new() -> Self {
        let c1 = Color::Rgb(231, 150, 86);
        let c2 = Color::Rgb(144, 85, 38);
        Self {
            colors: vec![
                Style::default(),
                Style::default().bg(c1),
                Style::default().bg(c2),
                Style::default().fg(c1),
            ],
            matrix: vec![
                vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
                vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0],
                vec![1, 1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 1, 1],
                vec![1, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1],
                vec![1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1],
                vec![1, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1],
                vec![1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 2, 2, 2, 1],
                vec![1, 1, 1, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1, 1, 1],
                vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0],
                vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
            ],
        }
    }

    pub fn get_size(&self) -> (u16, u16) {
        (self.matrix[0].len() as u16, self.matrix.len() as u16 + 2)
    }
}

impl Default for Logo {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Logo {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for y in 0..self.matrix.len() {
            for x in 0..self.matrix[y].len() {
                let index = self.matrix[y][x];
                if index != 0 && (x as u16) < area.width && (y as u16) < area.height {
                    let style = self.colors[index];
                    let x = x as u16;
                    let y = y as u16;
                    buf.set_string(x + area.x, y + area.y, " ", style);
                }
            }
        }
        let string = t!("hexpatch");
        if (area.width < string.len() as u16) || (area.height < self.matrix.len() as u16 + 2) {
            return;
        }
        buf.set_string(
            self.matrix[0].len() as u16 / 2 - string.len() as u16 / 2 + area.x,
            self.matrix.len() as u16 + 1 + area.y,
            string,
            self.colors[3],
        )
    }
}
