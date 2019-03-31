//! This crate is a selection of helpers that are more
//! opinionated than the standard HAL implementation. They may use
//! external crates, or do things like "take ownership" of an
//! interrupt to perform "automagical" behavior, intended to make
//! life easier for developers who want "sane defaults", or a more
//! robust starting place.

#![no_std]

pub mod wall_clock;
