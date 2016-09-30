#![allow(dead_code)]
extern crate textbox;
use textbox::*;
use std::io;
use std::io::{BufWriter, Error, ErrorKind, Write};
use std::path::PathBuf;

trait Buffer {
  fn name(&self) -> &str;
  fn paint(&self, tbox: &mut Textbox, at: Coord, active: bool);
  fn status(&self) -> String;
}

trait Navigable {
  fn cursor_up(&mut self) -> &mut Self;
  fn cursor_down(&mut self) -> &mut Self;
  fn cursor_left(&mut self) -> &mut Self;
  fn cursor_right(&mut self) -> &mut Self;

  fn page_up(&mut self) -> &mut Self;
  fn page_down(&mut self) -> &mut Self;
  fn home(&mut self) -> &mut Self;
  fn end(&mut self) -> &mut Self;
}

trait Editable {
  fn insert(&mut self, ch: char);
}

struct FileEdit {
  path: Option<PathBuf>,
  lines: Vec<String>,
  offset: Coord,
  cursor: Coord,
  view_size: Coord,
  dirty: bool,
}

struct CommandBar {
  prompt: String,
  entry: String,
  view: Coord,
  // handler:
}

impl CommandBar {
  fn new(view: Coord) -> Self {
    CommandBar { prompt: ":".to_string(), entry: String::new(), view: view }
  }
}

impl Buffer for CommandBar {
  fn name(&self) -> &str { &"command bar" }
  fn status(&self) -> String { "command bar".to_string() }

  fn paint(&self, tbox: &mut Textbox, at: Coord, active: bool) {
    if active {
      tbox.set_cells(at, &self.prompt, DEFAULT, DEFAULT);
      let at = at + self.prompt.len().to_col() + 1.to_col();
      tbox.set_cells(at, &self.entry, DEFAULT, DEFAULT);
      let at = at + self.entry.len().to_col();
      tbox.set_cursor(at);
    }
  }
}



impl Editable for CommandBar {
  fn insert(&mut self, ch: char) {
    match ch {
      '\n' => {
        println!("done typing command, call handler!");
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

impl FileEdit {
  fn from_file(view_size: Coord, filename: &str) -> Self {
    use std::io::{BufRead, BufReader};
    use std::fs::File;

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
      view_size: view_size,
      dirty: false,
    }
  }

  fn save(&mut self) -> io::Result<usize> {
    use std::fs::OpenOptions;

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
      if row >= self.view_size.row() {
        break;
      }
      // let mut initial_spaces = true;
      if self.offset.col() < line.len() {
        for (col, ch) in line[self.offset.col()..].chars().enumerate() {
          if col >= self.view_size.col() {
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
}

impl Navigable for FileEdit {
  fn cursor_up(&mut self) -> &mut Self {
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

    self
  }

  fn cursor_down(&mut self) -> &mut Self {
    if self.offset.row() + self.cursor.row() >= self.lines.len() - 1 {
      // do nothing
    } else if self.cursor.row() >= self.view_size.row() - 1 {
      self.cursor.1 = self.view_size.row() - 1;
      self.offset.1 += 1;
    } else {
      self.cursor.1 += 1;
    }

    if self.offset.0 + self.cursor.0 >=
       self.lines[self.offset.1 + self.cursor.1].len() {
      self.end();
    }

    self
  }

  fn cursor_left(&mut self) -> &mut Self {
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

    self
  }

  fn cursor_right(&mut self) -> &mut Self {
    let offset_row = self.offset.1;
    let cursor_row = self.cursor.1;
    let view_cols = self.view_size.0;
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

    self
  }

  fn page_up(&mut self) -> &mut Self {
    if self.offset.row() + self.cursor.row() == 0 {
      // do nothing
    } else if self.offset.1 == 0 {
      self.cursor.1 = 0;
    } else if self.offset.row() <= self.view_size.row() - 1 {
      if self.cursor.row() > self.view_size.row() - self.offset.row() {
        self.cursor.1 -= self.view_size.row() - self.offset.row();
      }
      self.offset.1 = 0;
    } else {
      self.offset.1 -= self.view_size.row();
    }

    if self.offset.0 + self.cursor.0 >=
       self.lines[self.offset.1 + self.cursor.1].len() {
      self.end();
    }

    self
  }

  fn page_down(&mut self) -> &mut Self {
    if self.offset.1 + self.cursor.1 >= self.lines.len() - 1 {
      // do nothing
    } else if self.lines.len() < self.view_size.1 {
      self.cursor.1 = self.lines.len() - 1;
    } else if self.offset.1 >= self.lines.len() - self.view_size.1 {
      self.cursor.1 = self.lines.len() - self.offset.1 - 1;
    } else {
      self.offset.1 += self.view_size.1;
      if self.offset.1 + self.view_size.1 >= self.lines.len() - 1 {
        self.offset.1 = self.lines.len() - self.view_size.1;
      }
    }

    if self.offset.0 + self.cursor.0 >=
       self.lines[self.offset.1 + self.cursor.1].len() {
      self.end();
    }

    self
  }

  fn home(&mut self) -> &mut Self {
    self.offset.0 = 0;
    self.cursor.0 = 0;

    self
  }

  fn end(&mut self) -> &mut Self {
    let offset_row = self.offset.1;
    let cursor_row = self.cursor.1;
    let view_cols = self.view_size.0;
    let line_len = self.lines[offset_row + cursor_row].len();

    if view_cols >= line_len {
      self.offset.0 = 0;
      self.cursor.0 = line_len;
    } else {
      self.offset.0 = line_len + 1 - view_cols;
      self.cursor.0 = view_cols - 1;
    }

    self
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
}

fn paint_status_bar(tbox: &mut Textbox, buf: &FileEdit) {
  let Coord(cols, rows) = tbox.size();
  let status = buf.status();
  for col in 0..cols {
    tbox.set_cell(Coord(col, rows - 2), ' ', DEFAULT, DEFAULT | REVERSE);
    // tbox.set_cell(Coord(col, rows - 1), ' ', DEFAULT, DEFAULT);
  }
  tbox.set_cells(Coord(cols - 2 - status.len(), rows - 2),
                 &status,
                 DEFAULT,
                 DEFAULT | REVERSE);
}

fn main() {
  let mut tbox = TextboxImpl::init().unwrap();
  let size = tbox.size();
  tbox.set_clear_style(DEFAULT, DEFAULT);

  'arg_loop: for arg in std::env::args().skip(1) {
    tbox.clear();
    tbox.present();

    let mut buf = FileEdit::from_file(size - 2.to_row(), &arg);
    let mut cmd = CommandBar::new(Coord(size.col(), 1));
    let mut edit_mode = true;
    buf.paint(&mut tbox, zero(), edit_mode);
    cmd.paint(&mut tbox, Coord(0, size.row() - 1), !edit_mode);
    paint_status_bar(&mut tbox, &buf);
    tbox.present();

    {
      // let mut ch = ' ';
      let mut changed = false;
      // let mut x = 0;
      'event_loop: loop {
        {
          if let Some(e) = tbox.pop_event() {
            match e {
              Event::Key(_, CTRL, Key::Char('Q')) => break 'arg_loop,
              Event::Key(_, _, Key::Escape) => break 'event_loop,
              Event::Key(_, CTRL, Key::Char('S')) => {
                buf.save().unwrap();
                changed = true;
              }
              Event::Key(_, CTRL, Key::Char('G')) => {
                edit_mode = false;
                changed = true;
              }
              Event::Key(_, CTRL, Key::Char('F')) => {
                // buf.save().unwrap();
                changed = true;
              }
              Event::Key(_, _, Key::Up) |
              Event::Key(_, CTRL, Key::Char('K')) => {
                buf.cursor_up();
                changed = true;
              }
              Event::Key(_, _, Key::Down) |
              Event::Key(_, CTRL, Key::Char('J')) => {
                buf.cursor_down();
                changed = true;
              }
              Event::Key(_, _, Key::Left) |
              Event::Key(_, CTRL, Key::Char('H')) => {
                buf.cursor_left();
                changed = true;
              }
              Event::Key(_, _, Key::Right) |
              Event::Key(_, CTRL, Key::Char('L')) => {
                buf.cursor_right();
                changed = true;
              }
              Event::Key(_, _, Key::PageUp) |
              Event::Key(_, CTRL_SHIFT, Key::Char('K')) => {
                buf.page_up();
                changed = true;
              }
              Event::Key(_, _, Key::PageDown) |
              Event::Key(_, CTRL_SHIFT, Key::Char('J')) => {
                buf.page_down();
                changed = true;
              }
              Event::Key(_, _, Key::Home) |
              Event::Key(_, CTRL_SHIFT, Key::Char('H')) => {
                buf.home();
                changed = true;
              }
              Event::Key(_, _, Key::End) |
              Event::Key(_, CTRL_SHIFT, Key::Char('L')) => {
                buf.end();
                changed = true;
              }
              Event::Key(ch, _, Key::Char(_)) => {
                if edit_mode { buf.insert(ch) } else { cmd.insert(ch) }
                changed = true;
              }
              Event::Key(_, _, Key::Enter) => {
                if edit_mode { buf.insert('\n') } else { cmd.insert('\n') }
                changed = true;
              }
              Event::Key(_, _, Key::Backspace) => {
                if edit_mode { buf.insert('\x08') } else { cmd.insert('\x08') }
                changed = true;
              }
              Event::Key(_, _, Key::Delete) => {
                if edit_mode { buf.insert('\x7f') } else { cmd.insert('\x7f') }
                changed = true;
              }
              Event::Key(_, _, Key::Tab) => {
                if edit_mode {
                  buf.insert(' ');
                  buf.insert(' ');
                } else {
                  cmd.insert(' ');
                  cmd.insert(' ');
                }
                changed = true;
              }
              // Some(Event::Key(c, k, m)) => {
              //   // println!("({:?}, {:?}, {:?})", c, k, m);
              //   // ch = c;
              //   // changed = true;
              // }
              _ => (),
            }
          }
        }
        {
          if changed {
            tbox.clear();
            buf.paint(&mut tbox, zero(), edit_mode);
            cmd.paint(&mut tbox, Coord(0, size.row() - 1), !edit_mode);
            paint_status_bar(&mut tbox, &buf);
            changed = false;
            tbox.present();
          }
        }
      }
    }
  }
}
