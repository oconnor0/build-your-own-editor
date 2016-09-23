#![allow(dead_code)]
extern crate textbox;
use textbox::*;

struct Buffer {
  // filename: String,
  lines: Vec<String>,
  offset: Coord,
  cursor: Coord,
  view_size: Coord,
}

impl Buffer {
  fn from_file(view_size: Coord, filename: &str) -> Buffer {
    use std::io::{BufRead, BufReader};
    use std::fs::File;

    let file = File::open(filename).unwrap();
    let bufr = BufReader::new(&file);
    let mut lines = vec![];
    for line in bufr.lines() {
      lines.push(line.unwrap());
    }

    Buffer {
      lines: lines,
      offset: zero(),
      cursor: zero(),
      view_size: view_size,
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
      for (col, ch) in line.chars().enumerate() {
        if col >= self.view_size.col() {
          break;
        } else if /*initial_spaces &&*/ ch == ' ' {
          tbox.set_cell(Coord(col, row), 183 as char, BRIGHT | BLACK, BLACK);
        } else {
          // initial_spaces = false;
          tbox.set_cell(Coord(col, row), ch, WHITE, BLACK);
        }
      }
    }
  }

  fn cursor_down(&mut self) -> bool {
    if self.offset.row() + self.cursor.row() >= self.lines.len() - 1 {
      // do nothing
      false
    } else if self.cursor.row() >= self.view_size.row() - 1 {
      self.cursor.1 = self.view_size.row() - 1;
      self.offset.1 += 1;
      true
    } else {
      self.cursor.1 += 1;
      true
    }
  }

  fn page_down(&mut self) -> bool {
    if self.offset.row() + self.cursor.row() >= self.lines.len() - 1 {
      false
    } else if self.lines.len() < self.view_size.row() {
      self.cursor.1 = self.lines.len() - 1;
      true
    } else if self.offset.row() == self.lines.len() - self.view_size.row()  {
      self.cursor.1 = self.view_size.row() - 1;
      true
    } else if self.offset.row() >= self.lines.len() - 2 * self.view_size.row() {
      self.offset.1 = self.lines.len() - self.view_size.row();
      true
    } else {
      self.offset.1 += self.view_size.row();
      true
    }
  }

  fn cursor_up(&mut self) -> bool {
    if self.offset.row() + self.cursor.row() == 0 {
      // do nothing
      false
    } else if self.cursor.row() == 0 {
      self.offset.1 -= 1;
      true
    } else {
      self.cursor.1 -= 1;
      true
    }
  }

  fn page_up(&mut self) -> bool {
    if self.offset.row() + self.cursor.row() == 0 {
      false
    } else if self.offset.1 == 0 {
      self.cursor.1 = 0;
      true
    } else if self.offset.row() <= self.view_size.row() - 1 {
      if self.cursor.row() > self.view_size.row() - self.offset.row() {
        self.cursor.1 -= self.view_size.row() - self.offset.row();
      }
      self.offset.1 = 0;
      true
    } else {
      self.offset.1 -= self.view_size.row();
      true
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

  let mut buf = Buffer::from_file(Coord(cols, rows - 2), "src/main.rs");
  // let mut buf = Buffer::from_file((cols, rows - 2), "Cargo.toml");
  buf.paint(&mut tbox, zero());
  for col in 0..cols {
    tbox.set_cell(Coord(col, rows - 2), ' ', BLACK, WHITE);
  }
  tbox.present();

  {
    let mut ch = ' ';
    let mut changed = false;
    let mut x = 0;
    loop {
      {
        match tbox.pop_event() {
          Some(Event::Key(_, _, Key::Escape)) => return,
          Some(Event::Key(_, _, Key::PageUp)) => changed = buf.page_up(),
          Some(Event::Key(_, _, Key::Up)) => changed = buf.cursor_up(),
          Some(Event::Key(_, _, Key::Down)) => changed = buf.cursor_down(),
          Some(Event::Key(_, _, Key::PageDown)) => changed = buf.page_down(),
          Some(Event::Key(c, k, m)) => {
            println!("({:?}, {:?}, {:?})", c, k, m);
            ch = c;
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
          tbox.set_cell(Coord(x, rows - 1), ch, WHITE, BLACK);
          changed = false;
          // tbox.set_cursor(x + 1, rows - 1);
          tbox.present();
          x += 1;
        }
      }
    }
  }
}
