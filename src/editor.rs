use std::{fs::File, io};

use crossterm::{
    cursor, event::{read, Event, KeyCode::{self, Char}, KeyEvent}, execute, style::Print, terminal::{
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
    cmd: String,      // The command being typed
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
            cmd: String::new(),
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

    fn die(&mut self, msg: String) -> io::Result<()> {
        self.deinit()?;
        println!("{}", msg);

        Ok(())
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

        // TODO: draw the status bar
        execute!(self.out, cursor::MoveTo(0, self.sr - 1), Print(self.cmd.as_str()))?;

        execute!(
            self.out,
            cursor::MoveTo(self.cx, self.cy),
            cursor::Show,
        )?;

        Ok(())
    }

    fn read_key(&mut self) -> io::Result<KeyEvent> {
        match read() {
            Ok(Event::Key(event)) => {
                return Ok(event);
            }
            Err(err) => {
                self.die(err.to_string())?;
            }
            _ => (),
        }

        Err(io::Error::other("Failed to read key"))
    }

    fn process_normal(&mut self) -> io::Result<()> {
        let key = self.read_key()?;

        match key.code {
            Char('i') => self.mode = Mode::Insert,
            Char(':') => self.mode = Mode::Command,
            Char('q') => self.quit = true, // TODO: make this happen in command mode only
            _ => (),
        }
        Ok(())
    }

    fn process_command(&mut self) -> io::Result<()> {
        self.cmd = String::from(":");

        loop {
            self.redraw_screen()?;
            let key = self.read_key()?.code;
            match key {
                KeyCode::Enter => break,
                KeyCode::Esc => {
                    self.cmd = String::new();
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => _ = self.cmd.pop(),
                KeyCode::Char(c) => self.cmd.push(c),
                _ => (),
            }
        }

        if self.cmd == String::from(":q") {
            self.quit = true;
        }

        self.mode = Mode::Normal;
        Ok(())
    }

    fn process_insert(&mut self) -> io::Result<()> {
        if self.read_key()?.code == KeyCode::Esc {
            self.mode = Mode::Normal;
        }
        Ok(())
    }

    fn process_input(&mut self) -> io::Result<()> {
        match self.mode {
            Mode::Normal => self.process_normal()?,
            Mode::Command => self.process_command()?,
            Mode::Insert => self.process_insert()?,
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
