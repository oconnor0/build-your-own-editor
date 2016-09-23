#[macro_use]
extern crate bitflags;
extern crate bit_set;
extern crate num_traits;

mod types {
  use std::ops::Add;
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

  impl Zero for Coord {
    fn zero() -> Self { Coord(0, 0) }
    fn is_zero(&self) -> bool { self.0 == 0 && self.1 == 0 }
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

      const RIGHT_ALT   = 1,
      const LEFT_ALT    = 2,
      const ALT         = RIGHT_ALT.bits|LEFT_ALT.bits,

      const RIGHT_CTRL  = 4,
      const LEFT_CTRL   = 8,
      const CTRL        = RIGHT_CTRL.bits|LEFT_CTRL.bits,

      const SHIFT       = 16,
      const CAPS_LOCK   = 32,
      const NUM_LOCK    = 64,
      const SCROLL_LOCK = 128,
      const ENHANCED    = 256,
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

  pub trait Textbox
  {
    fn init() -> Result<Self> where Self: Sized;

    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn size(&self) -> Coord;

    fn set_clear_style(&mut self, fg_clear: Style, bg_clear: Style);
    fn clear(&mut self);

    fn present(&mut self);

    fn set_cursor(&mut self, coord: Coord);
    fn hide_cursor(&mut self);

    fn put_cell(&mut self, coord: Coord, cell: Cell);
    fn set_cell(&mut self, coord: Coord, ch: char, fg: Style, bg: Style);

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

// pub fn init() -> Result<TextboxImpl> {
//   TextboxImpl::init()
// }
