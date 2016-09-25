#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate termbox_sys;

use self::termbox_sys::*;
use std::ops::Drop;
use std::os::raw::c_int;

pub use types::*;

#[derive(Debug)]
pub struct TermboxWrapper {
}

fn to_tb_style(s: Style) -> u16 {
  let mut tb = TB_DEFAULT;

  if s.contains(BRIGHT) {
    tb |= TB_BOLD
  }
  if s.contains(BOLD) {
    tb |= TB_BOLD
  }
  if s.contains(REVERSE) {
    tb |= TB_REVERSE
  }
  if s.contains(UNDERLINE) {
    tb |= TB_UNDERLINE
  }

  if s.contains(WHITE) {
    tb |= TB_WHITE
  } else if s.contains(MAGENTA) {
    tb |= TB_MAGENTA
  } else if s.contains(CYAN) {
    tb |= TB_CYAN
  } else if s.contains(YELLOW) {
    tb |= TB_YELLOW
  } else if s.contains(BLUE) {
    tb |= TB_BLUE
  } else if s.contains(GREEN) {
    tb |= TB_GREEN
  } else if s.contains(RED) {
    tb |= TB_RED
  } else if s.contains(BLACK) {
    tb |= TB_BLACK
  } else if s.contains(DEFAULT) {
    tb |= TB_DEFAULT
  }

  tb
}

fn to_mods(raw: u8) -> Mod { NO_MODS }

fn to_event(raw: RawEvent) -> Option<Event> {
  if raw.etype == TB_EVENT_KEY {
    let mut mods = to_mods(raw.emod);
    let kc = match raw.key {
      kc if kc == TB_KEY_ESC => Key::Escape,
      kc if kc == TB_KEY_ARROW_LEFT => Key::Left,
      kc if kc == TB_KEY_ARROW_UP => Key::Up,
      kc if kc == TB_KEY_ARROW_RIGHT => Key::Right,
      kc if kc == TB_KEY_ARROW_DOWN => Key::Down,
      kc if kc == TB_KEY_HOME => Key::Home,
      kc if kc == TB_KEY_END => Key::End,
      kc if kc == TB_KEY_PGUP => Key::PageUp,
      kc if kc == TB_KEY_PGDN => Key::PageDown,
      kc if kc == TB_KEY_BACKSPACE => Key::Backspace,
      kc if kc == TB_KEY_TAB => Key::Tab,
      kc if kc == TB_KEY_ENTER => Key::Enter,
      kc if kc == TB_KEY_CTRL_Q => {
        mods |= CTRL;
        Key::Char('Q')
      }
      // kc if kc >= w::VK_F1 as u16 && kc <= w::VK_F24 as u16 => {
      //   Key::F((kc - w::VK_F1 as u16 + 1) as u8)
      // }
      // // Doesn't handle shift.
      // // Letters
      // kc if kc >= 65 && kc <= 90 => Key::Char(kc as u8 as char),
      // // Numbers
      // kc if kc >= 48 && kc <= 57 => Key::Char(kc as u8 as char),
      _ => Key::Char('\0'),
    };
    Some(Event::Key(char::from_u32(raw.ch).unwrap(), mods, kc))
  } else {
    None
  }
}

impl Textbox for TermboxWrapper {
  fn init() -> Result<Self> {
    unsafe {
      let err = tb_init();
      if err != 0 {
        panic!("tb_init failed! {}", err);
      }
    }
    Ok(TermboxWrapper {})
  }

  fn size(&self) -> Coord {
    let cols = unsafe { tb_width() };
    let rows = unsafe { tb_height() };
    Coord(cols as usize, rows as usize)
  }

  fn set_clear_style(&mut self, fg: Style, bg: Style) {
    let tb_fg = to_tb_style(fg);
    let tb_bg = to_tb_style(bg);
    unsafe { tb_set_clear_attributes(tb_fg, tb_bg) }
  }
  fn clear(&mut self) { unsafe { tb_clear() } }

  fn present(&mut self) { unsafe { tb_present() } }

  fn set_cursor(&mut self, coord: Coord) {
    unsafe { tb_set_cursor(coord.0 as c_int, coord.1 as c_int) }
  }
  fn hide_cursor(&mut self) {
    unsafe { tb_set_cursor(TB_HIDE_CURSOR, TB_HIDE_CURSOR) }
  }

  fn put_cell(&mut self, coord: Coord, cell: Cell) {
    let tb_ch = cell.ch as u32;
    let tb_fg = to_tb_style(cell.fg);
    let tb_bg = to_tb_style(cell.bg);
    unsafe {
      tb_change_cell(coord.0 as c_int, coord.1 as c_int, tb_ch, tb_fg, tb_bg)
    }
  }

  fn set_input_mode(&mut self, _: InputMode) -> InputMode { unimplemented!() }
  fn set_output_mode(&mut self, _: OutputMode) -> OutputMode {
    unimplemented!()
  }

  fn pop_event(&mut self) -> Option<Event> {
    use std::mem::uninitialized;
    unsafe {
      let mut raw: RawEvent = uninitialized();
      let ty = tb_poll_event(&mut raw as *mut RawEvent);
      if ty > 0 { to_event(raw) } else { None }
    }
  }
}

impl Drop for TermboxWrapper {
  fn drop(&mut self) {
    unsafe {
      tb_shutdown();
    }
  }
}
