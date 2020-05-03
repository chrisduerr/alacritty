// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use bitflags::bitflags;

use serde::{Deserialize, Serialize};

use crate::ansi::{Color, NamedColor};
use crate::grid::{self, GridCell};
use crate::index::Column;

// Maximum number of zerowidth characters which will be stored per cell.
pub const MAX_ZEROWIDTH_CHARS: usize = 5;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct Flags: u16 {
        const INVERSE           = 0b00_0000_0001;
        const BOLD              = 0b00_0000_0010;
        const ITALIC            = 0b00_0000_0100;
        const BOLD_ITALIC       = 0b00_0000_0110;
        const UNDERLINE         = 0b00_0000_1000;
        const WRAPLINE          = 0b00_0001_0000;
        const WIDE_CHAR         = 0b00_0010_0000;
        const WIDE_CHAR_SPACER  = 0b00_0100_0000;
        const DIM               = 0b00_1000_0000;
        const DIM_BOLD          = 0b00_1000_0010;
        const HIDDEN            = 0b01_0000_0000;
        const STRIKEOUT         = 0b10_0000_0000;
    }
}

const fn default_extra() -> [char; MAX_ZEROWIDTH_CHARS] {
    [' '; MAX_ZEROWIDTH_CHARS]
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Cell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub flags: Flags,
    #[serde(default = "default_extra")]
    pub extra: [char; MAX_ZEROWIDTH_CHARS],
}

impl Default for Cell {
    fn default() -> Cell {
        Cell::new(' ', Color::Named(NamedColor::Foreground), Color::Named(NamedColor::Background))
    }
}

impl GridCell for Cell {
    #[inline]
    fn is_empty(&self) -> bool {
        (self.c == ' ' || self.c == '\t')
            && self.extra[0] == ' '
            && self.bg == Color::Named(NamedColor::Background)
            && self.fg == Color::Named(NamedColor::Foreground)
            && !self.flags.intersects(
                Flags::INVERSE
                    | Flags::UNDERLINE
                    | Flags::STRIKEOUT
                    | Flags::WRAPLINE
                    | Flags::WIDE_CHAR_SPACER,
            )
    }

    #[inline]
    fn flags(&self) -> &Flags {
        &self.flags
    }

    #[inline]
    fn flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }

    #[inline]
    fn fast_eq(&self, other: Self) -> bool {
        self.bg == other.bg
    }
}

/// Get the length of occupied cells in a line
pub trait LineLength {
    /// Calculate the occupied line length
    fn line_length(&self) -> Column;
}

impl LineLength for grid::Row<Cell> {
    fn line_length(&self) -> Column {
        let mut length = Column(0);

        if self[Column(self.len() - 1)].flags.contains(Flags::WRAPLINE) {
            return Column(self.len());
        }

        for (index, cell) in self[..].iter().rev().enumerate() {
            if cell.c != ' ' || cell.extra[0] != ' ' {
                length = Column(self.len() - index);
                break;
            }
        }

        length
    }
}

impl Cell {
    #[inline]
    pub fn bold(&self) -> bool {
        self.flags.contains(Flags::BOLD)
    }

    #[inline]
    pub fn inverse(&self) -> bool {
        self.flags.contains(Flags::INVERSE)
    }

    #[inline]
    pub fn dim(&self) -> bool {
        self.flags.contains(Flags::DIM)
    }

    pub fn new(c: char, fg: Color, bg: Color) -> Cell {
        Cell { extra: [' '; MAX_ZEROWIDTH_CHARS], c, bg, fg, flags: Flags::empty() }
    }

    #[inline]
    pub fn reset(&mut self, template: &Cell) {
        // memcpy template to self
        *self = Cell { c: template.c, bg: template.bg, ..Cell::default() };
    }

    #[inline]
    pub fn chars(&self) -> [char; MAX_ZEROWIDTH_CHARS + 1] {
        unsafe {
            let mut chars = [std::mem::MaybeUninit::uninit(); MAX_ZEROWIDTH_CHARS + 1];
            std::ptr::write(chars[0].as_mut_ptr(), self.c);
            std::ptr::copy_nonoverlapping(
                self.extra.as_ptr() as *mut std::mem::MaybeUninit<char>,
                chars.as_mut_ptr().offset(1),
                self.extra.len(),
            );
            std::mem::transmute(chars)
        }
    }

    #[inline]
    pub fn push_extra(&mut self, c: char) {
        for elem in self.extra.iter_mut() {
            if elem == &' ' {
                *elem = c;
                break;
            }
        }
    }

    pub fn as_escape(&self, buf: &mut String, last: Self) {
        // Always push CSI introducer since it's more efficient to truncate later
        *buf += "\x1b[";
        let empty_len = buf.len();

        self.fg.as_escape(buf, last.fg, true);
        self.bg.as_escape(buf, last.bg, false);

        if self.flags == last.flags {
            if buf.len() == empty_len {
                // Remove previously added CSI introducer if nothing changed
                buf.truncate(empty_len - 2);
            } else {
                unsafe {
                    let last_byte = buf.len() - 1;
                    buf.as_bytes_mut()[last_byte] = b'm';
                }
            }
            return;
        }

        let last_bold = last.flags.contains(Flags::BOLD);
        let last_dim = last.flags.contains(Flags::DIM);
        let bold = self.flags.contains(Flags::BOLD);
        let dim = self.flags.contains(Flags::DIM);
        if last_bold != bold || last_dim != dim {
            if !bold && !dim {
                *buf += "22;";
            } else if bold {
                *buf += "1;";
            } else if dim {
                *buf += "2;";
            }
        }

        let last_italic = last.flags.contains(Flags::ITALIC);
        let italic = self.flags.contains(Flags::ITALIC);
        if italic != last_italic {
            if italic {
                *buf += "3;";
            } else if last_italic {
                *buf += "23;";
            }
        }

        let last_underline = last.flags.contains(Flags::UNDERLINE);
        let underline = self.flags.contains(Flags::UNDERLINE);
        if underline != last_underline {
            if underline {
                *buf += "4;";
            } else if last_underline {
                *buf += "24;";
            }
        }

        let last_inverse = last.flags.contains(Flags::INVERSE);
        let inverse = self.flags.contains(Flags::INVERSE);
        if inverse != last_inverse {
            if inverse {
                *buf += "7;";
            } else if last_inverse {
                *buf += "27;";
            }
        }

        let last_hidden = last.flags.contains(Flags::HIDDEN);
        let hidden = self.flags.contains(Flags::HIDDEN);
        if hidden != last_hidden {
            if hidden {
                *buf += "8;";
            } else if last_hidden {
                *buf += "28;";
            }
        }

        let last_strikeout = last.flags.contains(Flags::STRIKEOUT);
        let strikeout = self.flags.contains(Flags::STRIKEOUT);
        if strikeout != last_strikeout {
            if strikeout {
                *buf += "9;";
            } else if last_strikeout {
                *buf += "29;";
            }
        }

        if buf.len() == empty_len {
            // Remove previously added CSI introducer if nothing changed
            buf.truncate(empty_len - 2);
        } else {
            unsafe {
                let last_byte = buf.len() - 1;
                buf.as_bytes_mut()[last_byte] = b'm';
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, LineLength};

    use crate::grid::Row;
    use crate::index::Column;

    #[test]
    fn line_length_works() {
        let template = Cell::default();
        let mut row = Row::new(Column(10), &template);
        row[Column(5)].c = 'a';

        assert_eq!(row.line_length(), Column(6));
    }

    #[test]
    fn line_length_works_with_wrapline() {
        let template = Cell::default();
        let mut row = Row::new(Column(10), &template);
        row[Column(9)].flags.insert(super::Flags::WRAPLINE);

        assert_eq!(row.line_length(), Column(10));
    }
}

#[cfg(all(test, feature = "bench"))]
mod benches {
    extern crate test;
    use super::Cell;

    #[bench]
    fn cell_reset(b: &mut test::Bencher) {
        b.iter(|| {
            let mut cell = Cell::default();

            for _ in 0..100 {
                cell.reset(test::black_box(&Cell::default()));
            }

            test::black_box(cell);
        });
    }
}
