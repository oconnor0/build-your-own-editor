extern crate textbox;
use textbox::*;

fn main() {
  use std::io::BufReader;
  use std::io::BufRead;
  use std::fs::File;

  let mut tbox = Textbox::init().unwrap();
  let (cols, rows) = tbox.size();
  tbox.set_cursor(0, rows - 1);
  tbox.set_clear_style(BLUE, BLUE);
  tbox.clear();
  tbox.present();

  let f = File::open("src/main.rs").unwrap();
  let file = BufReader::new(&f);
  for (row, line) in file.lines().enumerate() {
    if row >= rows - 2 {
      break;
    }
    for (col, ch) in line.unwrap().chars().enumerate() {
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
          tbox.set_cursor(x + 1, rows - 1);
          tbox.set_cell(x, rows - 1, ch, WHITE, BLUE);
          x += 1;
          changed = false;
          tbox.present();
        }
      }
    }
  }
}
