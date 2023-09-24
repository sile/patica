//! This crate provides [Image], a data structure for representing and editing raster images.
//!
//! The data format of a image is a sequence of [ImageCommand]s.
//!
//! # See also
//!
//! - [patica](https://github.com/sile/patica): Terminal based pixel art editor using this crate.
#![warn(missing_docs)]
mod command;
mod image;
mod log;
mod pixel;

pub use self::command::{Command, CommandReader, CommandWriter, PatchCommand, PatchEntry};
pub use self::image::{Image, VersionedImage};
pub use self::log::Version;
pub use self::pixel::{Color, Point};
