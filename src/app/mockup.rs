#[cfg(test)]
pub mod test
{
    use std::io::{Stdout, Write};

    use ratatui::backend::CrosstermBackend;

    use crate::app::App;

    impl App
    {
        pub fn mockup(data: Vec<u8>) -> Self
        {
            let mut app = App::default();
            let mut input_file = tempfile::NamedTempFile::new().expect("Failed to create tempfile for mockup.");
            input_file.write_all(&data).expect("Failed to write data to tempfile for mockup.");
            app.open_file::<CrosstermBackend<Stdout>>(&input_file.path().to_string_lossy(), None).expect("Failed to open file for mockup.");
            app
        }
    }

    #[test]
    fn test_mockup()
    {
        let data = b"Hello, World!";
        let app = App::mockup(data.to_vec());
        assert_eq!(app.data, data);
    }
}