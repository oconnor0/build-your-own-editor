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
    let file = File::open(path.as_path()).unwrap();
    let bufr = BufReader::new(&file);
    let mut lines = vec![];
    for line in bufr.lines() {
      lines.push(line.unwrap());
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
          } else if ch == ' ' {
            tbox.set_cell(Coord(col, row), 183 as char, BRIGHT | BLACK, BLACK);
          } else {
            // initial_spaces = false;
            tbox.set_cell(Coord(col, row), ch, WHITE, BLACK);
          }
        }
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
  }

  fn page_down(&mut self) {
    if self.offset.row() + self.cursor.row() >= self.lines.len() - 1 {
      // do nothing
    } else if self.lines.len() < self.view_size.row() {
      self.cursor.1 = self.lines.len() - 1;
    } else if self.offset.row() == self.lines.len() - self.view_size.row() {
      self.cursor.1 = self.view_size.row() - 1;
    } else if self.offset.row() >= self.lines.len() - 2 * self.view_size.row() {
      self.offset.1 = self.lines.len() - self.view_size.row();
    } else {
      self.offset.1 += self.view_size.row();
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
}

fn main() {
  let mut tbox = TextboxImpl::init().unwrap();
  let size = tbox.size();
  let Coord(cols, rows) = size;
  // tbox.set_cursor(0, rows - 1);
  tbox.set_clear_style(WHITE, BLACK);
  tbox.clear();
  tbox.present();

  let mut buf = Buffer::from_file(size - 2.to_row(), "src/main.rs");
  buf.paint(&mut tbox, zero());
  for col in 0..cols {
    tbox.set_cell(Coord(col, rows - 2), ' ', BLACK, WHITE);
  }
  // tbox.set_cells(Coord(2, rows - 2),
  //                buf.name(),
  //                WHITE,
  //                BLACK | REVERSE);
  let pos = format!("{} - {:2}/{:2} - {:3}/{:3}",
                    buf.name(),
                    buf.offset.0 + buf.cursor.0 + 1,
                    buf.lines[buf.offset.1 + buf.cursor.1].len(),
                    buf.offset.1 + buf.cursor.1 + 1,
                    buf.lines.len());
  tbox.set_cells(Coord(cols - 2 - pos.len(), rows - 2),
                 &pos,
                 WHITE,
                 BLACK | REVERSE);
  tbox.present();

  {
    // let mut ch = ' ';
    let mut changed = false;
    // let mut x = 0;
    loop {
      {
        match tbox.pop_event() {
          Some(Event::Key(_, _, Key::Escape)) => return,
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
          Some(Event::Key(c, k, m)) => {
            println!("({:?}, {:?}, {:?})", c, k, m);
            // ch = c;
            changed = true;
          }
          _ => (),
        }
      }
      {
        if changed {
          tbox.clear();
          buf.paint(&mut tbox, Coord(0, 0));
          for col in 0..cols {
            tbox.set_cell(Coord(col, rows - 2), ' ', WHITE, BLACK | REVERSE);
          }
          // tbox.set_cells(Coord(2, rows - 2),
          //                buf.name(),
          //                WHITE,
          //                BLACK | REVERSE);
          let pos = format!("{} - {:2}/{:2} - {:3}/{:3}",
                            buf.name(),
                            buf.offset.0 + buf.cursor.0 + 1,
                            buf.lines[buf.offset.1 + buf.cursor.1].len(),
                            buf.offset.1 + buf.cursor.1 + 1,
                            buf.lines.len());
          tbox.set_cells(Coord(cols - 2 - pos.len(), rows - 2),
                         &pos,
                         WHITE,
                         BLACK | REVERSE);
          // tbox.set_cell(Coord(x, rows - 1), ch, WHITE, BLACK);
          changed = false;
          // tbox.set_cursor(x + 1, rows - 1);
          tbox.present();
          // x += 1;
        }
      }
    }
  }
}
