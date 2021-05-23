use std::ops::Index;

use log::trace;

use alacritty_terminal::ansi::NamedColor;
use alacritty_terminal::term::color::{Rgb, COUNT};

use crate::config::color::Colors;

/// Factor for automatic computation of dim colors.
pub const DIM_FACTOR: f32 = 0.66;

#[derive(Copy, Clone)]
pub struct List {
    active: [Option<Rgb>; COUNT],
    default: [Rgb; COUNT],
}

impl<'a> From<&'a Colors> for List {
    fn from(colors: &Colors) -> List {
        // Type inference fails without this annotation.
        let mut list = List {
            active: [None; COUNT],
            default: [Rgb::default(); COUNT],
        };

        list.update_defaults(colors);

        list
    }
}

impl List {
    /// Get the current value of a color.
    pub fn get(&self, index: usize) -> &Rgb {
        self.active[index].as_ref().unwrap_or(&self.default[index])
    }

    pub fn get_modified(&self, index: usize) -> Option<Rgb> {
        self.active[index]
    }

    /// Override the current value of a color.
    pub fn set(&mut self, index: usize, color: Rgb) {
        self.active[index] = Some(color);
    }

    /// Reset the value of a color to its default.
    pub fn reset(&mut self, index: usize) {
        self.active[index] = None;
    }

    /// Update the unmodified fallback colors.
    pub fn update_defaults(&mut self, colors: &Colors) {
        self.fill_named(colors);
        self.fill_cube(colors);
        self.fill_gray_ramp(colors);
    }

    fn fill_named(&mut self, colors: &Colors) {
        // Normals.
        self.default[NamedColor::Black as usize] = colors.normal.black;
        self.default[NamedColor::Red as usize] = colors.normal.red;
        self.default[NamedColor::Green as usize] = colors.normal.green;
        self.default[NamedColor::Yellow as usize] = colors.normal.yellow;
        self.default[NamedColor::Blue as usize] = colors.normal.blue;
        self.default[NamedColor::Magenta as usize] = colors.normal.magenta;
        self.default[NamedColor::Cyan as usize] = colors.normal.cyan;
        self.default[NamedColor::White as usize] = colors.normal.white;

        // Brights.
        self.default[NamedColor::BrightBlack as usize] = colors.bright.black;
        self.default[NamedColor::BrightRed as usize] = colors.bright.red;
        self.default[NamedColor::BrightGreen as usize] = colors.bright.green;
        self.default[NamedColor::BrightYellow as usize] = colors.bright.yellow;
        self.default[NamedColor::BrightBlue as usize] = colors.bright.blue;
        self.default[NamedColor::BrightMagenta as usize] = colors.bright.magenta;
        self.default[NamedColor::BrightCyan as usize] = colors.bright.cyan;
        self.default[NamedColor::BrightWhite as usize] = colors.bright.white;
        self.default[NamedColor::BrightForeground as usize] =
            colors.primary.bright_foreground.unwrap_or(colors.primary.foreground);

        // Foreground and background.
        self.default[NamedColor::Foreground as usize] = colors.primary.foreground;
        self.default[NamedColor::Background as usize] = colors.primary.background;

        // Dims.
        self.default[NamedColor::DimForeground as usize] =
            colors.primary.dim_foreground.unwrap_or(colors.primary.foreground * DIM_FACTOR);
        match colors.dim {
            Some(ref dim) => {
                trace!("Using config-provided dim colors");
                self.default[NamedColor::DimBlack as usize] = dim.black;
                self.default[NamedColor::DimRed as usize] = dim.red;
                self.default[NamedColor::DimGreen as usize] = dim.green;
                self.default[NamedColor::DimYellow as usize] = dim.yellow;
                self.default[NamedColor::DimBlue as usize] = dim.blue;
                self.default[NamedColor::DimMagenta as usize] = dim.magenta;
                self.default[NamedColor::DimCyan as usize] = dim.cyan;
                self.default[NamedColor::DimWhite as usize] = dim.white;
            },
            None => {
                trace!("Deriving dim colors from normal colors");
                self.default[NamedColor::DimBlack as usize] = colors.normal.black * DIM_FACTOR;
                self.default[NamedColor::DimRed as usize] = colors.normal.red * DIM_FACTOR;
                self.default[NamedColor::DimGreen as usize] = colors.normal.green * DIM_FACTOR;
                self.default[NamedColor::DimYellow as usize] = colors.normal.yellow * DIM_FACTOR;
                self.default[NamedColor::DimBlue as usize] = colors.normal.blue * DIM_FACTOR;
                self.default[NamedColor::DimMagenta as usize] = colors.normal.magenta * DIM_FACTOR;
                self.default[NamedColor::DimCyan as usize] = colors.normal.cyan * DIM_FACTOR;
                self.default[NamedColor::DimWhite as usize] = colors.normal.white * DIM_FACTOR;
            },
        }
    }

    fn fill_cube(&mut self, colors: &Colors) {
        let mut index: usize = 16;
        // Build colors.
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    // Override colors 16..232 with the config (if present).
                    if let Some(indexed_color) =
                        colors.indexed_colors.iter().find(|ic| ic.index() == index as u8)
                    {
                        self.default[index] = indexed_color.color;
                    } else {
                        self.default[index] = Rgb {
                            r: if r == 0 { 0 } else { r * 40 + 55 },
                            b: if b == 0 { 0 } else { b * 40 + 55 },
                            g: if g == 0 { 0 } else { g * 40 + 55 },
                        };
                    }
                    index += 1;
                }
            }
        }

        debug_assert!(index == 232);
    }

    fn fill_gray_ramp(&mut self, colors: &Colors) {
        let mut index: usize = 232;

        for i in 0..24 {
            // Index of the color is number of named colors + number of cube colors + i.
            let color_index = 16 + 216 + i;

            // Override colors 232..256 with the config (if present).
            if let Some(indexed_color) =
                colors.indexed_colors.iter().find(|ic| ic.index() == color_index)
            {
                self.default[index] = indexed_color.color;
                index += 1;
                continue;
            }

            let value = i * 10 + 8;
            self.default[index] = Rgb { r: value, g: value, b: value };
            index += 1;
        }

        debug_assert!(index == 256);
    }
}

impl Index<usize> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        self.get(idx)
    }
}

impl Index<NamedColor> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: NamedColor) -> &Self::Output {
        self.get(idx as usize)
    }
}
