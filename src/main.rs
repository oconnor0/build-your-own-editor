#![allow(dead_code)]
extern crate textbox;
use textbox::*;
// use std::ops::Add;

// type Point = (usize, usize);

// // impl Point {
// //   fn col(&self) -> usize { self.0 }
// //   fn row(&self) -> usize { self.1 }
// // }

// impl Add for Point {
//   type Output = Point;
//   fn add(self, rhs: Point) -> Point {
//     (self.0 + rhs.0, self.1 + rhs.1)
//   }
// }

struct Buffer {
  // filename: String,
  lines: Vec<String>,
  offset_row: usize,
  offset_col: usize,
  cursor_row: usize,
  cursor_col: usize,
  view_rows: usize,
  view_cols: usize,
}

impl Buffer {
  fn from_file(view: (usize, usize), filename: &str) -> Buffer {
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
      offset_row: 0,
      offset_col: 0,
      cursor_row: 0,
      cursor_col: 0,
      view_cols: view.0,
      view_rows: view.1,
    }
  }

  fn paint(&self, tbox: &mut Textbox, global: (usize, usize)) {
    // TODO: Only set cursor if active buffer.
    tbox.set_cursor(global.0 + self.cursor_col, global.1 + self.cursor_row);
    for (row, line) in self.lines[self.offset_row..].iter().enumerate() {
      if row >= self.view_rows {
        break;
      }
      // let mut initial_spaces = true;
      for (col, ch) in line.chars().enumerate() {
        if col >= self.view_cols {
          break;
        } else if /*initial_spaces &&*/ ch == ' ' {
          tbox.set_cell(col, row, 183 as char, BRIGHT | BLACK, BLACK);
        } else {
          // initial_spaces = false;
          tbox.set_cell(col, row, ch, WHITE, BLACK);
        }
      }
    }
  }

  fn cursor_down(&mut self) -> bool {
    if self.offset_row + self.cursor_row >= self.lines.len() - 1 {
      // do nothing
      false
    } else if self.cursor_row >= self.view_rows - 1 {
      self.cursor_row = self.view_rows - 1;
      self.offset_row += 1;
      true
    } else {
      self.cursor_row += 1;
      true
    }
  }

  fn page_down(&mut self) -> bool {
    if self.offset_row + self.cursor_row >= self.lines.len() - 1 {
      false
    } else if self.lines.len() < self.view_rows {
      self.cursor_row = self.lines.len() - 1;
      true
    } else if self.offset_row == self.lines.len() - self.view_rows  {
      self.cursor_row = self.view_rows - 1;
      true
    } else if self.offset_row >= self.lines.len() - 2 * self.view_rows {
      self.offset_row = self.lines.len() - self.view_rows;
      true
    } else {
      self.offset_row += self.view_rows;
      true
    }
  }

  fn cursor_up(&mut self) -> bool {
    if self.offset_row + self.cursor_row == 0 {
      // do nothing
      false
    } else if self.cursor_row == 0 {
      self.offset_row -= 1;
      true
    } else {
      self.cursor_row -= 1;
      true
    }
  }

  fn page_up(&mut self) -> bool {
    if self.offset_row + self.cursor_row == 0 {
      false
    } else if self.offset_row == 0 {
      self.cursor_row = 0;
      true
    } else if self.offset_row <= self.view_rows - 1 {
      if self.cursor_row > self.view_rows - self.offset_row {
        self.cursor_row -= self.view_rows - self.offset_row;
      }
      self.offset_row = 0;
      true
    } else {
      self.offset_row -= self.view_rows;
      true
    }
  }
}

fn main() {
  let mut tbox = Textbox::init().unwrap();
  let (cols, rows) = tbox.size();
  tbox.set_cursor(0, rows - 1);
  tbox.set_clear_style(WHITE, BLACK);
  tbox.clear();
  tbox.present();

  let mut buf = Buffer::from_file((cols, rows - 2), "src/main.rs");
  // let mut buf = Buffer::from_file((cols, rows - 2), "Cargo.toml");
  buf.paint(&mut tbox, (0, 0));
  for col in 0..cols {
    tbox.set_cell(col, rows - 2, ' ', BLACK, WHITE);
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
          buf.paint(&mut tbox, (0, 0));
          for col in 0..cols {
            tbox.set_cell(col, rows - 2, ' ', WHITE, BLACK | REVERSE);
          }
          tbox.set_cell(x, rows - 1, ch, WHITE, BLACK);
          changed = false;
          // tbox.set_cursor(x + 1, rows - 1);
          tbox.present();
          x += 1;
        }
      }
    }
  }
}
