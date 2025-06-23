use std::{fs::File, io};

use crossterm::{
    cursor, event::{read, Event, KeyCode::{self, Char}}, execute, style::Print, terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen
    }
};
use ropey::Rope;

enum Mode {
    Normal,
    Insert,
    Command,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
    }
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

    fn scroll(&self) {
        // set correct scroll offsets and cursor position
        todo!()
    }

    fn insert_char(&mut self, ch: char) {
        let idx = (self.ro + self.cy) as usize;
        let lineidx = (self.co + self.cx) as usize;
        self.text.insert_char(self.text.line_to_char(idx) + lineidx, ch);
        
        if ch != '\n' {
            self.cx += 1;
            return;
        }

        self.cx = 0;
        self.cy += 1;
    }

    fn delete_char(&mut self, direction: i64) {
        let idx = (self.ro + self.cy) as usize;
        let lineidx = idx + (self.co + self.cx) as usize;
        
        let end = (lineidx as i64 + direction * 2) as usize;

        let a = if end < lineidx { end } else { lineidx };
        let b = if end > lineidx { lineidx } else { end };
        self.text.remove(a..b);
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

        execute!(self.out, cursor::MoveTo(0, self.sr - 2), Print(format!("{}", self.mode.as_str())))?;
        execute!(self.out, cursor::MoveTo(0, self.sr - 1), Print(self.cmd.as_str()))?;

        execute!(
            self.out,
            cursor::MoveTo(self.cx, self.cy),
            cursor::Show,
        )?;

        Ok(())
    }

    fn read_key(&mut self) -> io::Result<KeyCode> {
        match read() {
            Ok(Event::Key(event)) => {
                return Ok(event.code);
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

        match key {
            Char('i') => self.mode = Mode::Insert,
            Char(':') => self.mode = Mode::Command,
            _ => (),
        }
        Ok(())
    }

    fn process_command(&mut self) -> io::Result<()> {
        self.cmd = String::from(":");

        loop {
            self.redraw_screen()?;

            let key = self.read_key()?;
            match key {
                KeyCode::Enter => break,
                KeyCode::Esc => {
                    self.cmd = String::new();
                    break;
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
        let key = self.read_key()?;

        match key {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => self.insert_char('\n'),
            KeyCode::Backspace => self.delete_char(-2),
            KeyCode::Delete => self.delete_char(2),
            KeyCode::Up => self.cy -= 1,
            KeyCode::Down => self.cy += 1,
            KeyCode::Right => self.cx += 1,
            KeyCode::Left => self.cx -= 1,
            Char(c) => self.insert_char(c),
            _ => (),
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
