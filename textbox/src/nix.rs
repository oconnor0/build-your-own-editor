#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate termbox_sys;

use self::termbox_sys::*;

pub use types::*;

#[derive(Debug)]
pub struct TermboxWrapper {
}

impl Textbox for TermboxWrapper {
  fn init() -> Result<Self> { unimplemented!(); }

  fn rows(&self) -> usize { unimplemented!(); }
  fn cols(&self) -> usize { unimplemented!(); }
  fn size(&self) -> Coord { unimplemented!(); }

  fn set_clear_style(&mut self, fg_clear: Style, bg_clear: Style) { unimplemented!(); }
  fn clear(&mut self) { unimplemented!(); }

  fn present(&mut self) { unimplemented!(); }

  fn set_cursor(&mut self, coord: Coord) { unimplemented!(); }
  fn hide_cursor(&mut self) { unimplemented!(); }

  fn put_cell(&mut self, coord: Coord, cell: Cell) { unimplemented!(); }
  fn set_cell(&mut self, coord: Coord, ch: char, fg: Style, bg: Style) { unimplemented!(); }

  fn set_input_mode(&mut self, _: InputMode) -> InputMode { unimplemented!(); }
  fn set_output_mode(&mut self, _: OutputMode) -> OutputMode { unimplemented!(); }

  fn pop_event(&mut self) -> Option<Event> { unimplemented!(); }
}
