#![allow(dead_code)]
extern crate textbox;
use textbox::*;
use std::path::PathBuf;

struct Buffer {
  path: Option<PathBuf>,
  lines: Vec<String>,
  offset: Coord,
  cursor: Coord,
  view_size: Coord,
}

impl Buffer {
  fn from_file(view_size: Coord, filename: &str) -> Buffer {
    use std::io::{BufRead, BufReader};
    use std::fs::File;

    let path = PathBuf::from(filename);
    let mut lines = vec![];

    if path.exists() {
      let file = File::open(path.as_path()).unwrap();
      let bufr = BufReader::new(&file);
      for line in bufr.lines() {
        lines.push(line.unwrap());
      }
    } else {
      lines.push(String::new());
    }

    Buffer {
      path: Some(path),
      lines: lines,
      offset: zero(),
      cursor: zero(),
      view_size: view_size,
    }
  }

  fn name(&self) -> &str {
    match self.path {
      Some(ref path) => path.to_str().unwrap(),
      None => "-- buffer --",
    }
  }

  fn paint(&self, tbox: &mut Textbox, global: Coord) {
    // TODO: Only set cursor if active buffer.
    tbox.set_cursor(global + self.cursor);
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

  fn insert(&mut self, ch: char) {
    use std::cmp::min;

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
      _ => {
        self.lines[row_at].insert(col_at, ch);
        self.cursor_right();
      }
    }
  }

  fn cursor_down(&mut self) {
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
  }

  fn page_down(&mut self) {
    if self.offset.1 + self.cursor.1 >= self.lines.len() - 1 {
      // do nothing
    } else if self.lines.len() < self.view_size.row() {
      self.cursor.1 = self.lines.len() - 1;
    } else {
      self.offset.1 += self.view_size.1;
      if self.offset.1 + self.view_size.1 >= self.lines.len() - 1 {
        self.offset.1 = self.lines.len() - self.view_size.1;
        self.cursor.1 = self.view_size.1 - 1;
      }
    }
  }

  fn cursor_up(&mut self) {
    if self.offset.row() + self.cursor.row() == 0 {
      // do nothing
    } else if self.cursor.row() == 0 {
      self.offset.1 -= 1;
    } else {
      self.cursor.1 -= 1;
    }
  }

  fn cursor_right(&mut self) {
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

  fn page_up(&mut self) {
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
  }

  fn home(&mut self) {
    self.offset.0 = 0;
    self.cursor.0 = 0;
  }

  fn end(&mut self) {
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
            "{} - {}/{} - {}/{}",
            self.name(),
            curr_col,
            cols_in_row,
            curr_row,
            rows_in_buf)
  }
}

fn paint_status_bar(tbox: &mut Textbox, buf: &Buffer) {
  let Coord(cols, rows) = tbox.size();
  let status = buf.status();
  for col in 0..cols {
    tbox.set_cell(Coord(col, rows - 2), ' ', DEFAULT, DEFAULT | REVERSE);
    tbox.set_cell(Coord(col, rows - 1), ' ', DEFAULT, DEFAULT);
  }
  tbox.set_cells(Coord(cols - 2 - status.len(), rows - 2),
                 &status,
                 DEFAULT,
                 DEFAULT | REVERSE);
}


fn main() {
  let mut tbox = TextboxImpl::init().unwrap();
  let size = tbox.size();
  let Coord(cols, rows) = size;
  // tbox.set_cursor(0, rows - 1);
  tbox.set_clear_style(DEFAULT, DEFAULT);

  'arg_loop: for arg in std::env::args().skip(1) {
    tbox.clear();
    tbox.present();

    let mut buf = Buffer::from_file(size - 2.to_row(), &arg);
    buf.paint(&mut tbox, zero());
    paint_status_bar(&mut tbox, &buf);
    tbox.present();

    {
      // let mut ch = ' ';
      let mut changed = false;
      // let mut x = 0;
      'event_loop: loop {
        {
          let e = tbox.pop_event();
          println!("{:?}", e);
          match e {
            Some(Event::Key(_, CTRL, Key::Char('Q'))) => break 'arg_loop,
            Some(Event::Key(_, _, Key::Escape)) => break 'event_loop,
            Some(Event::Key(_, _, Key::PageUp)) => {
              buf.page_up();
              changed = true;
            }
            Some(Event::Key(_, _, Key::Up)) => {
              buf.cursor_up();
              changed = true;
            }
            Some(Event::Key(_, _, Key::Down)) => {
              buf.cursor_down();
              changed = true;
            }
            Some(Event::Key(_, _, Key::Left)) => {
              buf.cursor_left();
              changed = true;
            }
            Some(Event::Key(_, _, Key::Right)) => {
              buf.cursor_right();
              changed = true;
            }
            Some(Event::Key(_, _, Key::PageDown)) => {
              buf.page_down();
              changed = true;
            }
            Some(Event::Key(_, _, Key::End)) => {
              buf.end();
              changed = true;
            }
            Some(Event::Key(_, _, Key::Home)) => {
              buf.home();
              changed = true;
            }
            Some(Event::Key(ch, _, Key::Char(_))) => {
              buf.insert(ch);
              changed = true;
            }
            Some(Event::Key(_, _, Key::Enter)) => {
              buf.insert('\n');
              changed = true;
            }
            Some(Event::Key(_, _, Key::Backspace)) => {
              buf.insert('\x08');
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
        {
          if changed {
            tbox.clear();
            buf.paint(&mut tbox, Coord(0, 0));
            paint_status_bar(&mut tbox, &buf);
            changed = false;
            tbox.present();
          }
        }
      }
    }
  }
}
