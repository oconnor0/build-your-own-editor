#[macro_use]
extern crate bitflags;
extern crate bit_set;
extern crate num_traits;

mod types {
  use std::ops::{Add, Sub};
  pub use num_traits::{zero, Zero};
  use std::result;

  pub type Result<T> = result::Result<T, String>;

  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct Coord(pub usize, pub usize);

  impl Coord {
    pub fn col(&self) -> usize { self.0 }
    pub fn row(&self) -> usize { self.1 }
  }

  impl Add for Coord {
    type Output = Self;
    fn add(self, rhs: Coord) -> Coord { Coord(self.0 + rhs.0, self.1 + rhs.1) }
  }

  impl Sub for Coord {
    type Output = Self;
    fn sub(self, rhs: Coord) -> Coord { Coord(self.0 - rhs.0, self.1 - rhs.1) }
  }

  impl Zero for Coord {
    fn zero() -> Self { Coord(0, 0) }
    fn is_zero(&self) -> bool { self.0 == 0 && self.1 == 0 }
  }

  pub trait ToCol {
    fn to_col(&self) -> Coord;
  }

  impl ToCol for usize {
    fn to_col(&self) -> Coord { Coord(*self, 0) }
  }

  pub trait ToRow {
    fn to_row(&self) -> Coord;
  }

  impl ToRow for usize {
    fn to_row(&self) -> Coord { Coord(0, *self) }
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
    pub flags Mod: u32 {
      const NO_MODS     = 0,

      // const RIGHT_ALT   = 0x0001,
      // const LEFT_ALT    = 0x0002,
      const ALT         = 0x0003, //RIGHT_ALT.bits|LEFT_ALT.bits,

      // const RIGHT_CTRL  = 0x0004,
      // const LEFT_CTRL   = 0x0008,
      const CTRL        = 0x000c, //RIGHT_CTRL.bits|LEFT_CTRL.bits,

      const SHIFT       = 0x0010,

      const ALT_CTRL    = ALT.bits|CTRL.bits,
      const ALT_SHIFT   = ALT.bits|SHIFT.bits,
      const CTRL_SHIFT  = CTRL.bits|SHIFT.bits,
      const ALT_CTRL_SHIFT = ALT.bits|CTRL.bits|SHIFT.bits,

      // const CAPS_LOCK   = 0x0020,
      // const NUM_LOCK    = 0x0040,
      // const SCROLL_LOCK = 0x0080,
      // const META        = 0x0100,
      // const MENU        = 0x0200,
    }
  }


  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub enum Key {
    F(u8),
    Char(char),
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
    Enter,
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub enum Event {
    Key(char, Mod, Key),
  }

  bitflags! {
     pub flags Style: u16 {
       const DEFAULT   = 0x8000,

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

  pub trait Textbox {
    fn init() -> Result<Self> where Self: Sized;

    #[inline]
    fn cols(&self) -> usize { self.size().0 }
    #[inline]
    fn rows(&self) -> usize { self.size().1 }
    #[inline]
    fn size(&self) -> Coord;

    fn set_clear_style(&mut self, fg: Style, bg: Style);
    fn clear(&mut self);

    fn present(&mut self);

    fn set_cursor(&mut self, coord: Coord);
    fn hide_cursor(&mut self);

    #[inline]
    fn put_cell(&mut self, coord: Coord, cell: Cell);
    #[inline]
    fn set_cell(&mut self, coord: Coord, ch: char, fg: Style, bg: Style) {
      self.put_cell(coord, Cell { ch: ch, fg: fg, bg: bg })
    }

    fn set_cells(&mut self, coord: Coord, chs: &str, fg: Style, bg: Style) {
      for (i, ch) in chs.chars().enumerate() {
        self.set_cell(coord + i.to_col(), ch, fg, bg);
      }
    }

    fn set_input_mode(&mut self, _: InputMode) -> InputMode;
    fn set_output_mode(&mut self, _: OutputMode) -> OutputMode;

    fn pop_event(&mut self) -> Option<Event>;
  }
}

#[cfg(unix)]
mod nix;
#[cfg(unix)]
pub use nix::*;
#[cfg(unix)]
pub type TextboxImpl = nix::TermboxWrapper;

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::*;
#[cfg(windows)]
pub type TextboxImpl = win::WinConsoleWrapper;
