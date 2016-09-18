#![allow(dead_code)]
extern crate textbox;
use textbox::*;

struct Buffer {
  // filename: String,
  lines: Vec<String>,
  offset_row: usize,
  offset_col: usize,
}

impl Buffer {
  fn from_file(filename: &str) -> Buffer {
    use std::io::BufReader;
    use std::io::BufRead;
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
    }
  }
}

fn main() {
  let mut tbox = Textbox::init().unwrap();
  let (cols, rows) = tbox.size();
  tbox.set_cursor(0, rows - 1);
  tbox.set_clear_style(BLUE, BLUE);
  tbox.clear();
  tbox.present();

  // let f = File::open("src/main.rs").unwrap();
  // let file = BufReader::new(&f);
  // for (row, line) in file.lines().enumerate() {
  //   if row >= rows - 2 {
  //     break;
  //   }
  //   for (col, ch) in line.unwrap().chars().enumerate() {
  //     if col >= cols {
  //       break;
  //     }
  //     tbox.set_cell(col, row, ch, WHITE, BLUE);
  //   }
  // }
  let mut buf = Buffer::from_file("src/main.rs");
  for (row, line) in buf.lines[buf.offset_row..].iter().enumerate() {
    if row >= rows - 2 {
      break;
    }
    for (col, ch) in line.chars().enumerate() {
      if col >= cols {
        break;
      }
      tbox.set_cell(col, row, ch, WHITE, BLUE);
    }
  }
  for col in 0..cols {
    tbox.set_cell(col, rows - 2, ' ', BLUE, WHITE);
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
          Some(Event::Key(_, _, Key::PageUp)) => {
            if buf.offset_row < rows {
              buf.offset_row = 0;
              changed = true;
            } else {
              buf.offset_row -= rows - 2;
              changed = true;
            }
          }
          Some(Event::Key(_, _, Key::Up)) => {
            if buf.offset_row > 0 {
              buf.offset_row -= 1;
              changed = true;
            }
          }
          Some(Event::Key(_, _, Key::Down)) => {
            if buf.offset_row < buf.lines.len() {
              buf.offset_row += 1;
              changed = true;
            }
          }
          Some(Event::Key(_, _, Key::PageDown)) => {
            if buf.offset_row >= buf.lines.len() - rows + 2 {
              buf.offset_row = buf.lines.len();
              changed = true;
            } else {
              buf.offset_row += rows - 2;
              changed = true;
            }
          }
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
          for (row, line) in buf.lines[buf.offset_row..].iter().enumerate() {
            if row >= rows - 2 {
              break;
            }
            for (col, ch) in line.chars().enumerate() {
              if col >= cols {
                break;
              }
              tbox.set_cell(col, row, ch, WHITE, BLUE);
            }
          }
          for col in 0..cols {
            tbox.set_cell(col, rows - 2, ' ', BLUE, WHITE);
          }
          tbox.set_cell(x, rows - 1, ch, WHITE, BLUE);
          changed = false;
          tbox.set_cursor(x + 1, rows - 1);
          tbox.present();
          x += 1;
        }
      }
    }
  }
}
