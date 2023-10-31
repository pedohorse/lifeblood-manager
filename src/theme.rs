use fltk::enums::Color;

pub const BG_COLOR: [u8; 3] = [48, 48, 60];  // main bg
pub const BG2_COLOR: [u8; 3] = [32, 32, 32];  // bg for input widgets and such
pub const FG_COLOR: [u8; 3] = [200, 200, 200];  // main fg
pub const SEL_COLOR: [u8; 3] = [128, 16, 16];  // selection
pub const CELL_BG_COLOR: Color = Color::from_rgb(64, 64, 80);
pub const CELL_BG_SEL_COLOR: Color = Color::from_rgb(128, 32, 32);
pub const CELL_BG_CUR_COLOR: Color = Color::from_rgb(64, 96, 80);
pub const CELL_FG_COLOR: Color = Color::from_rgb(192, 192, 192);