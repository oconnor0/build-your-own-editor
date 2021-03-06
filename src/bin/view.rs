#![allow(dead_code)]
extern crate textbox;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Write};
use std::path::PathBuf;
use textbox::*;

trait Buffer {
  fn name(&self) -> &str;
  fn paint(&self, tbox: &mut Textbox, at: Coord, active: bool);
  fn status(&self) -> String;
  fn view_size(&self) -> Coord;
}

trait Save {
  fn save(&mut self) -> io::Result<usize>;
}

trait Navigable {
  fn cursor_up(&mut self);
  fn cursor_down(&mut self);
  fn cursor_left(&mut self);
  fn cursor_right(&mut self);

  fn page_up(&mut self);
  fn page_down(&mut self);
  fn home(&mut self);
  fn end(&mut self);

  fn goto_line(&mut self, line: usize);
}

trait Editable {
  fn insert(&mut self, ch: char);
  fn delete_line(&mut self) -> String;
}

struct FileEdit {
  path: Option<PathBuf>,
  lines: Vec<String>,
  offset: Coord,
  cursor: Coord,
  v_size: Coord,
  dirty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Mode {
  Edit,
  Find,
  Goto,
}

impl Mode {
  fn is_edit(&self) -> bool { *self == Mode::Edit }
  fn is_cmd(&self) -> bool { *self == Mode::Find || *self == Mode::Goto }
}

struct CommandBar<B> {
  prompt: String,
  entry: String,
  v_size: Coord,
  buf: Box<B>,
  mode: Mode,
}

impl<B> CommandBar<B> {
  fn new(v_size: Coord, buf: B) -> Self {
    CommandBar {
      prompt: ":".to_string(),
      entry: String::new(),
      v_size: v_size,
      buf: Box::new(buf),
      mode: Mode::Edit,
    }
  }

  fn push_mode(&mut self, mode: Mode) { self.mode = mode; }


  fn pop_mode(&mut self) {
    self.mode = match self.mode {
      Mode::Edit => Mode::Edit,
      Mode::Find => Mode::Edit,
      Mode::Goto => Mode::Edit,
    };
  }
}

impl<B: Buffer> Buffer for CommandBar<B> {
  fn name(&self) -> &str { &"command bar" }
  fn status(&self) -> String {
    match self.mode {
      Mode::Edit => "*edit*".to_string(),
      Mode::Find => "*find*".to_string(),
      Mode::Goto => "*goto*".to_string(),
    }
  }

  fn paint(&self, tbox: &mut Textbox, at: Coord, active: bool) {
    self.buf.paint(tbox, at, active & self.mode.is_edit());
    let at = at + (self.buf.view_size().row() + 1).to_row();

    let Coord(cols, rows) = tbox.size();
    let status = self.buf.status();
    for col in 0..cols {
      tbox.set_cell(Coord(col, rows - 2), ' ', DEFAULT, DEFAULT | REVERSE);
      // tbox.set_cell(Coord(col, rows - 1), ' ', DEFAULT, DEFAULT);
    }
    tbox.set_cells(Coord(cols - 2 - status.len(), rows - 2),
                   &status,
                   DEFAULT,
                   DEFAULT | REVERSE);
    let status = self.status();
    tbox.set_cells(Coord(2, rows - 2), &status, DEFAULT, DEFAULT | REVERSE);

    if active & self.mode.is_cmd() {
      tbox.set_cells(at, &self.prompt, DEFAULT, DEFAULT);
      let at = at + self.prompt.len().to_col() + 1.to_col();
      tbox.set_cells(at, &self.entry, DEFAULT, DEFAULT);
      let at = at + self.entry.len().to_col();
      tbox.set_cursor(at);
    }
  }

  fn view_size(&self) -> Coord {
    Coord(self.v_size.col(), self.v_size.row() + self.buf.view_size().row())
  }
}

impl<B: Editable + Navigable> Editable for CommandBar<B> {
  fn insert(&mut self, ch: char) {
    if self.mode.is_edit() {
      self.buf.insert(ch);
    } else {
      match ch {
        '\n' => {
          if self.mode == Mode::Goto {
            if let Ok(line) = self.entry.trim().parse::<usize>() {
              self.buf.goto_line(if line > 0 { line - 1 } else { 0 });
            }
            self.entry.clear();
          }
          self.mode = Mode::Edit;
        }
        '\x08' => {
          // backspace
          if self.entry.len() > 0 {
            self.entry.pop();
          }
        }
        '\x7f' => (), // delete - ignore
        _ => self.entry.push(ch),
      }
    }
  }

  fn delete_line(&mut self) -> String {
    if self.mode.is_edit() {
      self.buf.delete_line()
    } else {
      let out = self.entry.to_string();
      self.entry.clear();
      out
    }
  }
}

impl<B: Navigable> Navigable for CommandBar<B> {
  fn cursor_up(&mut self) {
    if self.mode.is_edit() {
      self.buf.cursor_up();
    }
  }

  fn cursor_down(&mut self) {
    if self.mode.is_edit() {
      self.buf.cursor_down();
    }
  }

  fn cursor_left(&mut self) {
    if self.mode.is_edit() {
      self.buf.cursor_left();
    }
  }

  fn cursor_right(&mut self) {
    if self.mode.is_edit() {
      self.buf.cursor_right();
    }
  }

  fn page_up(&mut self) {
    if self.mode.is_edit() {
      self.buf.page_up();
    }
  }

  fn page_down(&mut self) {
    if self.mode.is_edit() {
      self.buf.page_down();
    }
  }

  fn home(&mut self) {
    if self.mode.is_edit() {
      self.buf.home();
    }
  }

  fn end(&mut self) {
    if self.mode.is_edit() {
      self.buf.end();
    }
  }

  fn goto_line(&mut self, line: usize) {
    if self.mode.is_edit() {
      self.buf.goto_line(line);
    }
  }
}

impl<B: Save> Save for CommandBar<B> {
  fn save(&mut self) -> io::Result<usize> { self.buf.save() }
}

impl FileEdit {
  fn from_file(v_size: Coord, filename: &str) -> Self {
    let path = PathBuf::from(filename);
    let mut lines = vec![];

    if path.exists() && path.is_file() {
      let file = File::open(path.as_path()).unwrap();
      let bufr = BufReader::new(&file);
      for line in bufr.lines() {
        lines.push(line.unwrap());
      }
    }
    if lines.len() == 0 {
      lines.push(String::new());
    }

    FileEdit {
      path: Some(path),
      lines: lines,
      offset: zero(),
      cursor: zero(),
      v_size: v_size,
      dirty: false,
    }
  }
}

impl Save for FileEdit {
  fn save(&mut self) -> io::Result<usize> {
    if self.dirty {
      if let Some(ref path) = self.path {
        let file = OpenOptions::new()
          .create(true)
          .write(true)
          .truncate(true)
          .open(path)
          .unwrap();
        let mut file = BufWriter::new(file);
        let nl = vec!['\n' as u8];
        let mut written = 0;
        for ref line in self.lines.iter() {
          written += try!(file.write(line.as_bytes()));
          written += try!(file.write(&nl));
        }
        self.dirty = false;
        Ok(written)
      } else {
        Err(Error::new(ErrorKind::NotFound, "no filename given"))
      }
    } else {
      Ok(0)
    }
  }
}

impl Buffer for FileEdit {
  fn name(&self) -> &str {
    match self.path {
      Some(ref path) => path.to_str().unwrap(),
      None => "-- buffer --",
    }
  }

  fn paint(&self, tbox: &mut Textbox, global: Coord, active: bool) {
    if active {
      tbox.set_cursor(global + self.cursor);
    }
    for (row, line) in self.lines[self.offset.row()..].iter().enumerate() {
      if row >= self.v_size.row() {
        break;
      }
      // let mut initial_spaces = true;
      if self.offset.col() < line.len() {
        for (col, ch) in line[self.offset.col()..].chars().enumerate() {
          if col >= self.v_size.col() {
            break;
            // } else if ch == ' ' {
            //   tbox.set_cell(Coord(col, row), 183 as char, BRIGHT | DEFAULT, DEFAULT);
          } else {
            // initial_spaces = false;
            tbox.set_cell(Coord(col, row), ch, DEFAULT, DEFAULT);
          }
        }
      }
    }
  }

  fn status(&self) -> String {
    let curr_col = 1 + self.offset.0 + self.cursor.0;
    let curr_row = 1 + self.offset.1 + self.cursor.1;
    let rows_in_buf = self.lines.len();
    let cols_in_row = //if curr_row < rows_in_buf {
      self.lines[self.offset.1 + self.cursor.1].len();
    // } else {
    // 0
    // };
    format!(// "{} - {:2}/{:2} - {:3}/{:3}",
            "{}{} - {}/{} - {}/{}",
            self.name(),
            if self.dirty { "*" } else { "" },
            curr_col,
            cols_in_row,
            curr_row,
            rows_in_buf)
  }

  fn view_size(&self) -> Coord { self.v_size }
}

impl Navigable for FileEdit {
  fn cursor_up(&mut self) {
    if self.offset.row() + self.cursor.row() == 0 {
      // do nothing
    } else if self.cursor.row() == 0 {
      self.offset.1 -= 1;
    } else {
      self.cursor.1 -= 1;
    }

    if self.offset.0 + self.cursor.0 >=
       self.lines[self.offset.1 + self.cursor.1].len() {
      self.end();
    }
  }

  fn cursor_down(&mut self) {
    if self.offset.row() + self.cursor.row() >= self.lines.len() - 1 {
      // do nothing
    } else if self.cursor.row() >= self.v_size.row() - 1 {
      self.cursor.1 = self.v_size.row() - 1;
      self.offset.1 += 1;
    } else {
      self.cursor.1 += 1;
    }

    if self.offset.0 + self.cursor.0 >=
       self.lines[self.offset.1 + self.cursor.1].len() {
      self.end();
    }
  }

  fn cursor_left(&mut self) {
    if self.offset.0 + self.cursor.0 == 0 {
      if self.offset.1 + self.cursor.1 > 0 {
        self.cursor_up();
        self.end();
      }
    } else if self.cursor.0 == 0 {
      self.offset.0 -= 1;
    } else {
      self.cursor.0 -= 1;
    }
  }

  fn cursor_right(&mut self) {
    let offset_row = self.offset.1;
    let cursor_row = self.cursor.1;
    let view_cols = self.v_size.0;
    let line_len = self.lines[offset_row + cursor_row].len();

    if self.offset.0 + self.cursor.0 >= line_len {
      if offset_row + cursor_row < self.lines.len() - 1 {
        self.cursor_down();
        self.home();
      } else {
        self.end();
      }
    } else if self.cursor.0 >= view_cols - 1 {
      self.offset.0 += 1;
      self.cursor.0 = view_cols - 1;
    } else {
      self.cursor.0 += 1;
    }
  }

  fn page_up(&mut self) {
    if self.offset.row() + self.cursor.row() == 0 {
      // do nothing
    } else if self.offset.1 == 0 {
      self.cursor.1 = 0;
    } else if self.offset.row() <= self.v_size.row() - 1 {
      if self.cursor.row() > self.v_size.row() - self.offset.row() {
        self.cursor.1 -= self.v_size.row() - self.offset.row();
      }
      self.offset.1 = 0;
    } else {
      self.offset.1 -= self.v_size.row();
    }

    if self.offset.0 + self.cursor.0 >=
       self.lines[self.offset.1 + self.cursor.1].len() {
      self.end();
    }
  }

  fn page_down(&mut self) {
    if self.offset.1 + self.cursor.1 >= self.lines.len() - 1 {
      // do nothing
    } else if self.lines.len() < self.v_size.1 {
      self.cursor.1 = self.lines.len() - 1;
    } else if self.offset.1 >= self.lines.len() - self.v_size.1 {
      self.cursor.1 = self.lines.len() - self.offset.1 - 1;
    } else {
      self.offset.1 += self.v_size.1;
      if self.offset.1 + self.v_size.1 >= self.lines.len() - 1 {
        self.offset.1 = self.lines.len() - self.v_size.1;
      }
    }

    if self.offset.0 + self.cursor.0 >=
       self.lines[self.offset.1 + self.cursor.1].len() {
      self.end();
    }
  }

  fn home(&mut self) {
    self.offset.0 = 0;
    self.cursor.0 = 0;
  }

  fn end(&mut self) {
    let offset_row = self.offset.1;
    let cursor_row = self.cursor.1;
    let view_cols = self.v_size.0;
    let line_len = self.lines[offset_row + cursor_row].len();

    if view_cols >= line_len {
      self.offset.0 = 0;
      self.cursor.0 = line_len;
    } else {
      self.offset.0 = line_len + 1 - view_cols;
      self.cursor.0 = view_cols - 1;
    }
  }

  fn goto_line(&mut self, line: usize) {
    let len = self.lines.len();
    let line = if line >= len { len - 1 } else { line };
    if line < self.v_size.1 {
      self.offset.1 = 0;
      self.cursor.1 = line;
    } else {
      self.offset.1 = line - self.cursor.1;
    }
  }
}

impl Editable for FileEdit {
  fn insert(&mut self, ch: char) {
    use std::cmp::min;
    self.dirty = true;

    let col_at = self.offset.col() + self.cursor.col();
    let cols = self.lines[self.offset.row() + self.cursor.row()].len();
    let row_at = self.offset.row() + self.cursor.row();
    match ch {
      '\n' => {
        let curr = self.lines[row_at][0..min(col_at, cols)].to_string();
        let next = self.lines[row_at][min(col_at, cols)..cols].to_string();
        self.lines[row_at] = curr;
        self.lines.insert(row_at + 1, next);
        self.cursor_down();
        self.home();
      }
      '\x08' => {
        // backspace
        let curr_row = self.offset.1 + self.cursor.1;
        let curr_col = self.offset.0 + self.cursor.0;
        if curr_col == 0 {
          if curr_row > 0 {
            // join lines
            let prev_row = curr_row - 1;
            self.cursor_up();
            self.end();
            let curr_str = self.lines.remove(curr_row);
            self.lines[prev_row].push_str(&curr_str);
          }
        } else {
          self.lines[curr_row].remove(curr_col - 1);
          self.cursor_left();
        }
      }
      '\x7f' => {
        // delete
        let curr_row = self.offset.1 + self.cursor.1;
        let curr_col = self.offset.0 + self.cursor.0;
        let line_len = self.lines[curr_row].len();
        if curr_col == line_len {
          if curr_row < self.lines.len() - 1 {
            // join lines
            let next_row = curr_row + 1;
            let next_str = self.lines.remove(next_row);
            self.lines[curr_row].push_str(&next_str);
          }
        } else {
          self.lines[curr_row].remove(curr_col);
        }
      }
      _ => {
        self.lines[row_at].insert(col_at, ch);
        self.cursor_right();
      }
    }
  }

  fn delete_line(&mut self) -> String {
    self.dirty = true;
    let curr_row = self.offset.1 + self.cursor.1;
    let curr_col = self.offset.0 + self.cursor.0;
    let curr_str = self.lines.remove(curr_row);
    if self.lines.len() == 0 {
      self.lines.push(String::new());
    }
    if curr_row >= self.lines.len() {
      self.cursor_up();
    }
    let curr_row = self.offset.1 + self.cursor.1;
    if curr_col >= self.lines[curr_row].len() {
      self.end();
    }
    curr_str
  }
}

fn main() {
  let mut tbox = TextboxImpl::init().unwrap();
  let size = tbox.size();
  tbox.set_clear_style(DEFAULT, DEFAULT);

  'arg_loop: for arg in std::env::args().skip(1) {
    tbox.clear();
    tbox.present();

    let buf = FileEdit::from_file(size - 2.to_row(), &arg);
    let mut cmd = CommandBar::new(Coord(size.col(), 1), buf);
    cmd.paint(&mut tbox, zero(), true);
    tbox.present();

    {
      'event_loop: loop {
        {
          if let Some(e) = tbox.pop_event() {
            match e {
              Event::Key(_, CTRL, Key::Char('Q')) => break 'arg_loop,
              Event::Key(_, NO_MODS, Key::Escape) => cmd.pop_mode(),
              Event::Key(_, CTRL, Key::Char('S')) => {
                cmd.save().unwrap();
              }
              Event::Key(_, CTRL, Key::Char('G')) => cmd.push_mode(Mode::Goto),
              Event::Key(_, CTRL, Key::Char('F')) => cmd.push_mode(Mode::Find),
              Event::Key(_, CTRL, Key::Char('X')) => {
                cmd.delete_line();
              }
              Event::Key(_, NO_MODS, Key::Up) |
              Event::Key(_, CTRL, Key::Char('K')) => cmd.cursor_up(),
              Event::Key(_, NO_MODS, Key::Down) |
              Event::Key(_, CTRL, Key::Char('J')) => cmd.cursor_down(),
              Event::Key(_, NO_MODS, Key::Left) |
              Event::Key(_, CTRL, Key::Char('H')) => cmd.cursor_left(),
              Event::Key(_, NO_MODS, Key::Right) |
              Event::Key(_, CTRL, Key::Char('L')) => cmd.cursor_right(),
              Event::Key(_, NO_MODS, Key::PageUp) |
              Event::Key(_, CTRL_SHIFT, Key::Char('K')) => cmd.page_up(),
              Event::Key(_, NO_MODS, Key::PageDown) |
              Event::Key(_, CTRL_SHIFT, Key::Char('J')) => cmd.page_down(),
              Event::Key(_, NO_MODS, Key::Home) |
              Event::Key(_, CTRL_SHIFT, Key::Char('H')) => cmd.home(),
              Event::Key(_, NO_MODS, Key::End) |
              Event::Key(_, CTRL_SHIFT, Key::Char('L')) => cmd.end(),
              // Event::Key('/', ALT, _) => {
              //   println!("divide");
              //   cmd.insert(0xf7 as char);
              // }
              Event::Key(ch, NO_MODS, Key::Char(_)) |
              Event::Key(ch, SHIFT, Key::Char(_)) => cmd.insert(ch),
              Event::Key(_, NO_MODS, Key::Enter) => cmd.insert('\n'),
              Event::Key(_, NO_MODS, Key::Backspace) => cmd.insert('\x08'),
              Event::Key(_, NO_MODS, Key::Delete) => cmd.insert('\x7f'),
              Event::Key(_, NO_MODS, Key::Tab) => {
                cmd.insert(' ');
                cmd.insert(' ');
              }
              _ => (),
            }
          }
        }

        tbox.clear();
        cmd.paint(&mut tbox, zero(), true);
        tbox.present();
      }
    }
  }
}
