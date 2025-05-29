use std::{fs::File, io::{self, Write}};

use crossterm::{
    cursor, event::{read, Event, KeyCode}, execute, style::Print, terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen
    }
};
use ropey::Rope;

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
    ro: u16,          // Row offset for scrolling
    co: u16,          // Column offset for scrolling
    mode: Mode,       // Current editor mode
    filename: String, // Name of file being edited
    quit: bool,       // Whether the program should quit next iteration
    out: io::Stdout,  // Where to write output to
    text: Rope,       // The current buffer being edited
}

impl Editor {
    pub fn default(filename: String) -> Self {
        Editor {
            cx: 0,
            cy: 0,
            sc: 0,
            sr: 0,
            ro: 0,
            co: 0,
            mode: Mode::Normal,
            filename: filename,
            quit: false,
            out: io::stdout(),
            text: Rope::new(),
        }
    }

    fn init(&mut self) -> io::Result<()> {
        (self.sc, self.sr) = terminal::size()?;

        execute!(self.out, EnterAlternateScreen)?;

        self.text = Rope::from_reader(
            File::open(self.filename.as_str())?
        )?;

        terminal::enable_raw_mode()
    }

    fn deinit(&mut self) -> io::Result<()> {
        execute!(self.out, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()
    }

    fn redraw_screen(&mut self) -> io::Result<()> {
        execute!(
            self.out,
            cursor::Hide,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
        )?;

        for y in 0..(self.sr - 2) {
            let linenum = (y + self.ro) as usize;
            if linenum >= self.text.len_lines() {
                break;
            }

            let line = self.text.line(linenum);
            execute!(self.out, cursor::MoveToColumn(0), Print(line))?;
        }

        // draw the status bar
        // draw the command line

        execute!(
            self.out,
            cursor::MoveTo(self.cx, self.cy),
            cursor::Show,
        )?;

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
