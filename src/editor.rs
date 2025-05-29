use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo, event::{read, Event, KeyCode}, execute, style::{Attribute, Print, SetAttribute}, terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen
    }
};

enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    cx: u16,          // Cursor X
    cy: u16,          // Cursor Y
    sc: u16,          // Screen columns
    sr: u16,          // Screen rows
    rowcount: u16,    // amount of rows in the file
    mode: Mode,       // Current editor mode
    filename: String, // Name of file being edited
    quit: bool,       // Whether the program should quit next iteration
    out: io::Stdout,  // Where to write output to
}

impl Editor {
    pub fn default(filename: String) -> Self {
        Editor {
            cx: 0,
            cy: 0,
            sc: 0,
            sr: 0,
            rowcount: 0,
            mode: Mode::Normal,
            filename: filename,
            quit: false,
            out: io::stdout(),
        }
    }

    fn init(&mut self) -> io::Result<()> {
        (self.sc, self.sr) = size()?;
        execute!(self.out, EnterAlternateScreen)?;
        enable_raw_mode()
    }

    fn deinit(&mut self) -> io::Result<()> {
        execute!(self.out, LeaveAlternateScreen)?;
        disable_raw_mode()
    }

    fn redraw_screen(&mut self) -> io::Result<()> {
        execute!(
            self.out,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print("cool things shall someday fill this realm of wonder".to_string()),
        )?;
        // for each line
        //     draw the line
        // draw the status bar

        Ok(())
    }

    fn process_input(&mut self) -> io::Result<()> {
        let ev = read()?;

        if ev == Event::Key(KeyCode::Char('q').into()) {
            self.quit = true;
        }

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.init()?;

        loop {
            self.redraw_screen()?;
            self.process_input()?;

            if self.quit {
                break;
            }
        }

        self.deinit()?;

        Ok(())
    }
}
