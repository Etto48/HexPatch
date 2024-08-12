#[cfg(test)]
pub mod test {
    use std::io::Write;

    use ratatui::{backend::TestBackend, Terminal};

    use crate::app::App;

    impl App {
        pub fn mockup(data: Vec<u8>) -> Self {
            let mut app = App::default();
            let mut input_file =
                tempfile::NamedTempFile::new().expect("Failed to create tempfile for mockup.");
            input_file
                .write_all(&data)
                .expect("Failed to write data to tempfile for mockup.");
            let mut terminal = Terminal::new(TestBackend::new(80, 25)).unwrap();
            app.open_file(&input_file.path().to_string_lossy(), &mut terminal)
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
