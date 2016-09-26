#![allow(dead_code)]
#![allow(unused_imports)]

extern crate winapi;
extern crate wio;

use bit_set::BitSet;
use std::collections::VecDeque;
use self::wio::console::{CharInfo, Input, InputBuffer, ScreenBuffer};
use self::winapi as w;

pub use types::*;

pub struct WinConsoleWrapper {
  stdin: InputBuffer,
  events: VecDeque<Input>,
  frontbuf: ScreenBuffer,
  backbuf: Box<[Cell]>,
  dirty_rows: BitSet,
  size: Coord,
  fg_clear: Style,
  bg_clear: Style,
}

fn to_fg(s: Style) -> u16 {
  use super::*;

  let mut fg = 0;

  if s.contains(BRIGHT) {
    fg |= w::FOREGROUND_INTENSITY
  }
  if s.contains(BOLD) {
    fg |= w::FOREGROUND_INTENSITY;
  }
  if s.contains(REVERSE) {
    fg |= w::COMMON_LVB_REVERSE_VIDEO
  }
  if s.contains(UNDERLINE) {
    fg |= w::COMMON_LVB_UNDERSCORE
  }

  if s.contains(WHITE) || s.contains(DEFAULT) {
    fg |= w::FOREGROUND_RED | w::FOREGROUND_GREEN | w::FOREGROUND_BLUE;
  } else if s.contains(MAGENTA) {
    fg |= w::FOREGROUND_RED | w::FOREGROUND_BLUE;
  } else if s.contains(CYAN) {
    fg |= w::FOREGROUND_GREEN | w::FOREGROUND_BLUE;
  } else if s.contains(YELLOW) {
    fg |= w::FOREGROUND_RED | w::FOREGROUND_GREEN;
  } else if s.contains(BLUE) {
    fg |= w::FOREGROUND_BLUE;
  } else if s.contains(GREEN) {
    fg |= w::FOREGROUND_GREEN;
  } else if s.contains(RED) {
    fg |= w::FOREGROUND_RED;
  } else if s.contains(BLACK) {
    // do nothing
  }

  fg
}

fn to_bg(s: Style) -> u16 {
  use super::*;

  let mut bg = 0;

  if s.contains(BRIGHT) {
    bg |= w::BACKGROUND_INTENSITY
  }
  if s.contains(BOLD) {
    bg |= w::BACKGROUND_INTENSITY;
  }
  if s.contains(REVERSE) {
    bg |= w::COMMON_LVB_REVERSE_VIDEO
  }
  if s.contains(UNDERLINE) {
    bg |= w::COMMON_LVB_UNDERSCORE
  }

  if s.contains(WHITE) {
    bg |= w::BACKGROUND_RED | w::BACKGROUND_GREEN | w::BACKGROUND_BLUE;
  } else if s.contains(MAGENTA) {
    bg |= w::BACKGROUND_RED | w::BACKGROUND_BLUE;
  } else if s.contains(CYAN) {
    bg |= w::BACKGROUND_GREEN | w::BACKGROUND_BLUE;
  } else if s.contains(YELLOW) {
    bg |= w::BACKGROUND_RED | w::BACKGROUND_GREEN;
  } else if s.contains(BLUE) {
    bg |= w::BACKGROUND_BLUE;
  } else if s.contains(GREEN) {
    bg |= w::BACKGROUND_GREEN;
  } else if s.contains(RED) {
    bg |= w::BACKGROUND_RED;
  }

  bg
}

fn to_mod(raw: u32) -> Mod {
  let mut m = NO_MODS;

  if raw & w::RIGHT_ALT_PRESSED > 0 {
    m |= ALT
  }
  if raw & w::LEFT_ALT_PRESSED > 0 {
    m |= ALT
  }

  if raw & w::RIGHT_CTRL_PRESSED > 0 {
    m |= CTRL
  }
  if raw & w::LEFT_CTRL_PRESSED > 0 {
    m |= CTRL
  }

  if raw & w::SHIFT_PRESSED > 0 {
    m |= SHIFT
  }
  // if raw & w::CAPSLOCK_ON > 0 {
  //   m |= CAPS_LOCK
  // }
  // if raw & w::NUMLOCK_ON > 0 {
  //   m |= NUM_LOCK
  // }
  // if raw & w::SCROLLLOCK_ON > 0 {
  //   m |= SCROLL_LOCK
  // }

  // META
  // MENU
  // if raw & w::RIGHT_CTRL_PRESSED > 0 {
  //   m |= RIGHT_CTRL
  // }
  // if raw & w::LEFT_CTRL_PRESSED > 0 {
  //   m |= LEFT_CTRL
  // }

  m
}

fn to_charinfo(c: Cell) -> CharInfo {
  CharInfo::new(c.ch as u16, to_fg(c.fg) | to_bg(c.bg))
}

fn to_event(raw: Input) -> Option<Event> {
  println!("{:?}", raw);
  match raw {
    Input::Key { key_down, key_code, wide_char, control_key_state, .. } => {
      if key_down {
        let ch = wide_char as u8 as char;
        let mods = to_mod(control_key_state);
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
          kc if kc == w::VK_RETURN as u16 => Key::Enter,
          kc if kc >= w::VK_F1 as u16 && kc <= w::VK_F24 as u16 => {
            Key::F((kc - w::VK_F1 as u16 + 1) as u8)
          }
          // Doesn't handle shift.
          // Printable characters.
          kc if kc >= 0x20 && kc <= 0x7e => Key::Char(kc as u8 as char),
          _ => Key::Char('\0'),
        };
        if ch == '\0' && !mods.is_empty() {
          None
        } else {
          Some(Event::Key(ch, mods, kc))
        }
      } else {
        None
      }
    }
    _ => None,
  }
}

impl Textbox for WinConsoleWrapper {
  fn init() -> Result<Self> {
    let stdin = InputBuffer::from_conin().unwrap();
    let frontbuf = ScreenBuffer::new().unwrap();
    let (cols, rows) = frontbuf.info().unwrap().size();
    let cell_count = (rows * cols) as usize;
    let backbuf = vec![Cell { ch: ' ', fg: DEFAULT, bg: DEFAULT, }; cell_count]
      .into_boxed_slice();
    frontbuf.set_active().unwrap();
    Ok(WinConsoleWrapper {
      stdin: stdin,
      events: VecDeque::new(),
      backbuf: backbuf,
      frontbuf: frontbuf,
      size: Coord(cols as usize, rows as usize),
      fg_clear: DEFAULT,
      bg_clear: DEFAULT,
      dirty_rows: BitSet::with_capacity(rows as usize),
    })
  }

  fn size(&self) -> Coord { self.size }

  fn set_clear_style(&mut self, fg: Style, bg: Style) {
    self.fg_clear = fg;
    self.bg_clear = bg;
  }
  fn clear(&mut self) {
    for row in 0..self.size.1 {
      self.dirty_rows.insert(row);
    }
    for ref mut cell in &mut *self.backbuf {
      cell.ch = ' ';
      cell.fg = self.fg_clear;
      cell.bg = self.bg_clear;
    }
  }

  fn present(&mut self) {
    if self.dirty_rows.len() == self.size.1 {
      let slice: Vec<CharInfo> = self.backbuf
        .iter()
        .map(|&cell| to_charinfo(cell))
        .collect();
      self.frontbuf
        .write_output(&slice, (self.size.0 as i16, self.size.1 as i16), (0, 0))
        .unwrap();
    } else {
      for row in self.dirty_rows.iter() {
        let slice: Vec<CharInfo> = self.backbuf[row * self.size.0..(row + 1) *
                                                                   self.size.0]
          .iter()
          .map(|&cell| to_charinfo(cell))
          .collect();
        self.frontbuf
          .write_output(&slice, (self.size.0 as i16, 1), (0, row as i16))
          .unwrap();
      }
    }
    self.dirty_rows.clear()
  }

  fn set_cursor(&mut self, coord: Coord) {
    // TODO: Should this make these changes to the backbuf?
    if coord.0 < self.size.0 && coord.1 < self.size.1 {
      self.frontbuf
        .set_cursor_position((coord.0 as i16, coord.1 as i16))
        .unwrap()
    }
  }
  fn hide_cursor(&mut self) { unimplemented!() }

  fn put_cell(&mut self, coord: Coord, cell: Cell) {
    if coord.0 < self.size.0 && coord.1 < self.size.1 {
      self.dirty_rows.insert(coord.1);
      self.backbuf[coord.0 + coord.1 * self.size.0] = cell
    }
  }

  fn set_input_mode(&mut self, _: InputMode) -> InputMode { unimplemented!() }
  fn set_output_mode(&mut self, _: OutputMode) -> OutputMode {
    OutputMode::Normal
  }

  fn pop_event(&mut self) -> Option<Event> {
    if self.events.len() > 0 {
      to_event(self.events.pop_front().unwrap())
    } else /*if self.stdin.available_input().unwrap_or(0) > 0*/ {
      match self.stdin.read_input() {
        Ok(inputs) => {
          self.events.extend(inputs);
          if let Some(e) = to_event(self.events.pop_front().unwrap()) {
            Some(e)
          } else {
            // TODO: Possible stack overflow.
            self.pop_event()
          }
        }
        Err(_) => None,
      }
    // } else {
    //   None
    }
  }
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
