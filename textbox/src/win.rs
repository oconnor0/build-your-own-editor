#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate wio;

use bit_set::BitSet;
use std::collections::VecDeque;
use std::result;
use self::wio::console::{CharInfo, Input, InputBuffer, ScreenBuffer};
use self::winapi as w;

type Result<T> = result::Result<T, String>;

bitflags! {
  pub flags Style: u16 {
    const DEFAULT   = 0,

    const BLACK     = 0x0001,
    const RED       = 0x0002,
    const GREEN     = 0x0004,
    const YELLOW    = 0x0008,
    const BLUE      = 0x0010,
    const MAGENTA   = 0x0020,
    const CYAN      = 0x0040,
    const WHITE     = 0x0080,
    const BRIGHT    = 0x0100,

    const BOLD      = 0x0200,
    const UNDERLINE = 0x0400,
    const REVERSE   = 0x0800,
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cell {
  pub ch: char,
  pub fg: Style,
  pub bg: Style,
}

bitflags! {
  pub flags Mod: u32 {
    const NO_MODS     = 0,

    const RIGHT_ALT   = w::RIGHT_ALT_PRESSED,
    const LEFT_ALT    = w::LEFT_ALT_PRESSED,
    const ALT         = RIGHT_ALT.bits|LEFT_ALT.bits,

    const RIGHT_CTRL  = w::RIGHT_CTRL_PRESSED,
    const LEFT_CTRL   = w::LEFT_CTRL_PRESSED,
    const CTRL        = RIGHT_CTRL.bits|LEFT_CTRL.bits,

    const SHIFT       = w::SHIFT_PRESSED,
    const CAPS_LOCK   = w::CAPSLOCK_ON,
    const NUM_LOCK    = w::NUMLOCK_ON,
    const SCROLL_LOCK = w::SCROLLLOCK_ON,
    const ENHANCED    = w::ENHANCED_KEY,
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
  F(u8),
  Char(char),
  // Num(u8),
  Left,
  Up,
  Right,
  Down,
  Escape,
  Insert,
  Delete,
  Home,
  End,
  PageUp,
  PageDown,
  Backspace,
  Tab,
  Return,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
  Key(char, Mod, Key),
}

pub struct Textbox {
  stdin: InputBuffer,
  events: VecDeque<Input>,
  frontbuf: ScreenBuffer,
  backbuf: Box<[Cell]>,
  dirty_rows: BitSet,
  rows: usize,
  cols: usize,
  fg_clear: Style,
  bg_clear: Style,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputMode {
  Current,
  Esc,
  Alt,
  Mouse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputMode {
  Current,
  Normal,
  Colors256,
  Colors216,
  Grayscale,
}

bitflags! {
    flags FgAttrs: u16 {
      const FG_BLACK      = 0,
      const FG_BLUE       = w::FOREGROUND_BLUE,
      const FG_CYAN       = w::FOREGROUND_BLUE|w::FOREGROUND_GREEN,
      const FG_GREEN      = w::FOREGROUND_GREEN,
      const FG_YELLOW     = w::FOREGROUND_GREEN|w::FOREGROUND_RED,
      const FG_RED        = w::FOREGROUND_RED,
      const FG_MAGENTA    = w::FOREGROUND_BLUE|w::FOREGROUND_RED,
      const FG_WHITE      = w::FOREGROUND_BLUE|w::FOREGROUND_GREEN
                          |w::FOREGROUND_RED,
      const FG_BRIGHT     = w::FOREGROUND_INTENSITY,

      const FG_BOLD       = 0,
      const FG_REVERSE    = w::COMMON_LVB_REVERSE_VIDEO,
      const FG_UNDERSCORE = w::COMMON_LVB_UNDERSCORE,

      const FG_DEFAULT    = w::FOREGROUND_BLUE|w::FOREGROUND_GREEN
                          |w::FOREGROUND_RED,
    }
  }

bitflags! {
    flags BgAttrs: u16 {
      const BG_BLACK      = 0,
      const BG_BLUE       = w::BACKGROUND_BLUE,
      const BG_CYAN       = w::BACKGROUND_BLUE|w::BACKGROUND_GREEN,
      const BG_GREEN      = w::BACKGROUND_GREEN,
      const BG_YELLOW     = w::BACKGROUND_GREEN|w::BACKGROUND_RED,
      const BG_RED        = w::BACKGROUND_RED,
      const BG_MAGENTA    = w::BACKGROUND_BLUE|BG_RED.bits,
      const BG_WHITE      = w::BACKGROUND_BLUE|w::BACKGROUND_GREEN
                          |w::BACKGROUND_RED,
      const BG_BRIGHT     = w::BACKGROUND_INTENSITY,

      const BG_BOLD       = 0,
      const BG_REVERSE    = w::COMMON_LVB_REVERSE_VIDEO,
      const BG_UNDERSCORE = w::COMMON_LVB_UNDERSCORE,

      const BG_DEFAULT    = 0,
    }
  }

impl From<Style> for FgAttrs {
  fn from(s: Style) -> FgAttrs {
    use super::*;

    let mut fg = FG_BLACK;

    if s.contains(BRIGHT) {
      fg |= FG_BRIGHT;
    }
    if s.contains(BOLD) {
      fg |= FG_BOLD;
    }
    if s.contains(REVERSE) {
      fg |= FG_REVERSE;
    }
    if s.contains(UNDERLINE) {
      fg |= FG_UNDERSCORE;
    }

    if s.contains(WHITE) {
      fg |= FG_WHITE
    } else if s.contains(MAGENTA) {
      fg |= FG_MAGENTA
    } else if s.contains(CYAN) {
      fg |= FG_CYAN
    } else if s.contains(YELLOW) {
      fg |= FG_YELLOW
    } else if s.contains(BLUE) {
      fg |= FG_BLUE
    } else if s.contains(GREEN) {
      fg |= FG_GREEN
    } else if s.contains(RED) {
      fg |= FG_RED
    } /*else if BLACK.contains(*s) {
fg |= FG_BLACK
}*/

    // println!("{:?} -> {:?}", s, fg);
    fg
  }
}

impl From<Style> for BgAttrs {
  fn from(s: Style) -> BgAttrs {
    use super::*;

    let mut bg = BG_BLACK;

    if s.contains(BRIGHT) {
      bg |= BG_BRIGHT
    }
    if s.contains(BOLD) {
      bg |= BG_BOLD;
    }
    if s.contains(REVERSE) {
      bg |= BG_REVERSE
    }
    if s.contains(UNDERLINE) {
      bg |= BG_UNDERSCORE
    }

    if s.contains(WHITE) {
      bg |= BG_WHITE
    } else if s.contains(MAGENTA) {
      bg |= BG_MAGENTA
    } else if s.contains(CYAN) {
      bg |= BG_CYAN
    } else if s.contains(YELLOW) {
      bg |= BG_YELLOW
    } else if s.contains(BLUE) {
      bg |= BG_BLUE
    } else if s.contains(GREEN) {
      bg |= BG_GREEN
    } else if s.contains(RED) {
      bg |= BG_RED
    } /*else if BLACK.contains(*s) {
bg |= BG_BLACK
}*/

    // println!("{:?} -> {:?}", s, bg);
    bg
  }
}

impl From<Cell> for CharInfo {
  fn from(c: Cell) -> CharInfo {
    let fg = FgAttrs::from(c.fg);
    let bg = BgAttrs::from(c.bg);
    CharInfo::new(c.ch as u16, fg.bits | bg.bits)
  }
}

impl Textbox {
  pub fn init() -> Result<Textbox> {
    let stdin = InputBuffer::from_conin().unwrap();
    let frontbuf = ScreenBuffer::new().unwrap();
    let (cols, rows) = frontbuf.info().unwrap().size();
    let cell_count = (rows * cols) as usize;
    let backbuf = vec![Cell {
      ch: ' ',
      fg: DEFAULT,
      bg: DEFAULT,
      }; cell_count]
                    .into_boxed_slice();
    frontbuf.set_active().unwrap();
    // println!("{} * {} = {}", cols, rows, cell_count);
    Ok(Textbox {
      stdin: stdin,
      events: VecDeque::new(),
      backbuf: backbuf,
      frontbuf: frontbuf,
      rows: rows as usize,
      cols: cols as usize,
      fg_clear: DEFAULT,
      bg_clear: DEFAULT,
      dirty_rows: BitSet::with_capacity(rows as usize),
    })
  }

  pub fn rows(&self) -> usize { self.rows }
  pub fn cols(&self) -> usize { self.cols }
  pub fn size(&self) -> (usize, usize) { (self.cols, self.rows) }

  pub fn set_clear_style(&mut self, fg_clear: Style, bg_clear: Style) {
    self.fg_clear = fg_clear;
    self.bg_clear = bg_clear;
  }

  pub fn clear(&mut self) {
    for row in 0..self.rows {
      self.dirty_rows.insert(row);
    }
    for ref mut cell in &mut *self.backbuf {
      cell.ch = ' ';
      cell.fg = self.fg_clear;
      cell.bg = self.bg_clear;
    }
  }

  pub fn present(&mut self) {
    if self.dirty_rows.len() == self.rows {
      let slice: Vec<CharInfo> = self.backbuf
                                     .iter()
                                     .map(|&cell| CharInfo::from(cell))
                                     .collect();
      self.frontbuf
          .write_output(&slice, (self.cols as i16, self.rows as i16), (0, 0))
          .unwrap();
    } else {
      for row in self.dirty_rows.iter() {
        let slice: Vec<CharInfo> = self.backbuf[row * self.cols..(row + 1) *
                                                                 self.cols]
                                     .iter()
                                     .map(|&cell| CharInfo::from(cell))
                                     .collect();
        self.frontbuf
            .write_output(&slice, (self.cols as i16, 1), (0, row as i16))
            .unwrap();
      }
    }
    self.dirty_rows.clear()
  }

  pub fn set_cursor(&mut self, col: usize, row: usize) {
    // TODO: Should this make these changes to the backbuf?
    if col < self.cols && row < self.rows {
      self.frontbuf.set_cursor_position((col as i16, row as i16)).unwrap()
    }
  }
  pub fn hide_cursor(&mut self) { unimplemented!() }

  #[inline]
  pub fn put_cell(&mut self, col: usize, row: usize, cell: Cell) {
    if col < self.cols && row < self.rows {
      self.dirty_rows.insert(row);
      self.backbuf[col + row * self.cols] = cell
    }
  }
  pub fn set_cell(&mut self,
                  col: usize,
                  row: usize,
                  ch: char,
                  fg: Style,
                  bg: Style) {
    self.put_cell(col,
                  row,
                  Cell {
                    ch: ch,
                    fg: fg,
                    bg: bg,
                  })
  }

  // pub fn set_cells(&mut self,
  //                  x: i16,
  //                  y: i16,
  //                  chs: &str,
  //                  fg: Color,
  //                  bg: Color) {
  //   let color = (bg | fg).bits() as u16;
  //   let cis: Vec<_> = chs.chars()
  //                        .map(|ch| CharInfo::new(ch as u16, color))
  //                        .collect();
  //   self.backbuf
  //       .write_output(&cis, (chs.len() as i16, 1), (x, y))
  //       .unwrap();
  // }

  pub fn set_input_mode(&mut self, _: InputMode) -> InputMode {
    unimplemented!();
  }

  pub fn set_output_mode(&mut self, _: OutputMode) -> OutputMode {
    OutputMode::Normal
  }

  pub fn pop_event(&mut self) -> Option<Event> {
    if self.events.len() > 0 {
      to_event(self.events.pop_front().unwrap())
    } else if self.stdin.available_input().unwrap_or(0) > 0 {
      match self.stdin.read_input() {
        Ok(inputs) => {
          self.events.extend(inputs);
          to_event(self.events.pop_front().unwrap())
        }
        Err(_) => None,
      }
    } else {
      None
    }
  }
}

fn to_event(input: Input) -> Option<Event> {
  match input {
    Input::Key {key_down, key_code, wide_char, control_key_state, ..} => {
      if key_down {
        println!("{:?}", key_code);
        let kc = match key_code {
          kc if kc == w::VK_ESCAPE as u16 => Key::Escape,
          kc if kc == w::VK_LEFT as u16 => Key::Left,
          kc if kc == w::VK_UP as u16 => Key::Up,
          kc if kc == w::VK_RIGHT as u16 => Key::Right,
          kc if kc == w::VK_DOWN as u16 => Key::Down,
          kc if kc == w::VK_DOWN as u16 => Key::Down,
          kc if kc == w::VK_INSERT as u16 => Key::Insert,
          kc if kc == w::VK_DELETE as u16 => Key::Delete,
          kc if kc == w::VK_HOME as u16 => Key::Home,
          kc if kc == w::VK_END as u16 => Key::End,
          kc if kc == w::VK_PRIOR as u16 => Key::PageUp,
          kc if kc == w::VK_NEXT as u16 => Key::PageDown,
          kc if kc == w::VK_BACK as u16 => Key::Backspace,
          kc if kc == w::VK_TAB as u16 => Key::Tab,
          kc if kc == w::VK_RETURN as u16 => Key::Return,
          kc if kc >= w::VK_F1 as u16 && kc <= w::VK_F24 as u16 => {
            Key::F((kc - w::VK_F1 as u16 + 1) as u8)
          }
          // Doesn't handle shift.
          // Letters
          kc if kc >= 65 && kc <= 90 => Key::Char(kc as u8 as char),
          // Numbers
          kc if kc >= 48 && kc <= 57 => Key::Char(kc as u8 as char),
          _ => Key::Char('\0'),
        };
        Some(Event::Key(wide_char as u8 as char,
                        Mod::from_bits_truncate(control_key_state),
                        kc))
      } else {
        None
      }
    }
    _ => None,
  }
}
