// Copyright (c) 2018-present, Facebook, Inc.
// All Rights Reserved.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

//! Base types used throughout Mononoke.

#![deny(warnings)]
// The allow(dead_code) is temporary until Thrift serialization is done.
#![allow(dead_code)]
#![feature(conservative_impl_trait)]
#![feature(try_from)]
#![feature(const_fn)]

extern crate ascii;
extern crate bincode;
extern crate blake2;
extern crate bytes;
extern crate chrono;
#[macro_use]
extern crate failure_ext as failure;
extern crate heapsize;
#[macro_use]
extern crate heapsize_derive;
#[macro_use]
extern crate lazy_static;
#[cfg_attr(test, macro_use)]
extern crate quickcheck;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate mononoke_types_thrift;

pub mod datetime;
pub mod errors;
pub mod file_change;
pub mod file_contents;
pub mod hash;
pub mod path;
pub mod tiny_changeset;
pub mod typed_hash;

pub use datetime::DateTime;
pub use file_change::{FileChange, FileType};
pub use file_contents::FileContents;
pub use path::{MPath, MPathElement, RepoPath};
pub use tiny_changeset::TinyChangeset;
pub use typed_hash::{ChangesetId, ContentId, UnodeId};

mod thrift {
    pub use mononoke_types_thrift::*;
}