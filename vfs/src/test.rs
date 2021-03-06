// Copyright (c) 2004-present, Facebook, Inc.
// All Rights Reserved.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

use itertools::assert_equal;

use mononoke_types::{MPath, MPathElement};

pub fn pel(path: &'static str) -> MPathElement {
    MPath::new(path).unwrap().into_iter().next().unwrap()
}

pub fn cmp<'a, P, S>(paths: P, expected: S)
where
    P: IntoIterator<Item = &'a MPathElement>,
    S: IntoIterator<Item = &'static str>,
{
    let mut paths: Vec<_> = paths.into_iter().cloned().collect();
    paths.sort();
    assert_equal(paths, expected.into_iter().map(pel));
}
