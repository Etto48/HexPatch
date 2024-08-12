#[cfg(test)]
pub mod test {
    use std::io::{Stdout, Write};

    use ratatui::{
        backend::CrosstermBackend,
        layout::{Position, Size},
        prelude::Backend,
    };

    use crate::app::App;

    pub struct MockupBackend {
        cursor_position: Position,
        size: Size,
    }

    impl MockupBackend {
        pub fn new(size_x: u16, size_y: u16) -> Self {
            Self {
                cursor_position: Position::default(),
                size: Size::new(size_x, size_y),
            }
        }
    }

    impl Backend for MockupBackend {
        fn draw<'a, I>(&mut self, _content: I) -> std::io::Result<()>
        where
            I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
        {
            Ok(())
        }

        fn hide_cursor(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn show_cursor(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn get_cursor_position(&mut self) -> std::io::Result<Position> {
            Ok(self.cursor_position)
        }

        fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> std::io::Result<()> {
            self.cursor_position = position.into();
            Ok(())
        }

        fn clear(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn size(&self) -> std::io::Result<Size> {
            Ok(self.size)
        }

        fn window_size(&mut self) -> std::io::Result<ratatui::backend::WindowSize> {
            Ok(ratatui::backend::WindowSize {
                columns_rows: self.size,
                pixels: Size::new(self.size.width * 8, self.size.height * 8),
            })
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl App {
        pub fn mockup(data: Vec<u8>) -> Self {
            let mut app = App::default();
            let mut input_file =
                tempfile::NamedTempFile::new().expect("Failed to create tempfile for mockup.");
            input_file
                .write_all(&data)
                .expect("Failed to write data to tempfile for mockup.");
            app.open_file::<CrosstermBackend<Stdout>>(&input_file.path().to_string_lossy(), None)
                .expect("Failed to open file for mockup.");
            app
        }
    }

    #[test]
    fn test_mockup() {
        let data = b"Hello, World!";
        let app = App::mockup(data.to_vec());
        assert_eq!(app.data.bytes(), data);
    }
}
