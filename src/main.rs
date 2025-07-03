use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hex_patch::{app::App, args};
use ratatui::backend::CrosstermBackend;
#[macro_use]
extern crate rust_i18n;

i18n!();

fn main() {
    let args = args::Args::parse();
    let theme = termbg::theme(Duration::from_secs(2));

    enable_raw_mode().expect(&t!("errors.enable_raw_mode"));
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).expect(&t!("errors.setup_commands"));
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend).expect(&t!("errors.create_terminal"));

    terminal.clear().expect(&t!("errors.clear_terminal"));
    let mut app = App::new(args, &mut terminal, theme).expect(&t!("errors.create_app"));
    let res = app.run(&mut terminal);
    terminal.clear().expect(&t!("errors.clear_terminal"));

    disable_raw_mode().expect(&t!("errors.disable_raw_mode"));

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect(&t!("errors.teardown_commands"));
    terminal.show_cursor().expect(&t!("errors.show_cursor"));

    if let Err(err) = res {
        println!("{err:?}")
    }
}
