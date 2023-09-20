//! This crate provides [Canvas], a data structure for editing raster images.
//!
//! The data format of a image is a sequence of [Command]s.
//!
//! # See also
//!
//! - [patica](https://github.com/sile/patica): Terminal based pixel art editor using this crate.
#![warn(missing_docs)]
mod canvas;
mod command;
mod log;
mod pixel;

pub use self::canvas::{Canvas, VersionedCanvas};
pub use self::command::{Command, CommandReader, CommandWriter, PatchCommand, PatchEntry};
pub use self::log::Version;
pub use self::pixel::{Color, Point};
