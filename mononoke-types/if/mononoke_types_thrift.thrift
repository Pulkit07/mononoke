// Copyright (c) 2018-present, Facebook, Inc.
// All Rights Reserved.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

//! ------------
//! IMPORTANT!!!
//! ------------
//! Do not change the order of the fields! Changing the order of the fields
//! results in compatible but *not* identical serializations, so hashes will
//! change.
//! ------------
//! IMPORTANT!!!
//! ------------

// Thrift doesn't have fixed-length arrays, so a 256-bit hash can be
// represented in one of two ways:
// 1. as four i64s
// 2. as just a newtype around a `binary`
//
// Representation 1 is very appealing as it provides a 1:1 map between Rust's
// data structures and Thrift's. But it means that the full hash is not
// available as a single contiguous block in memory. That makes some
// zero-copy optimizations hard.
// Representation 2 does have the benefit of the hash being available as a
// contiguous block, but it requires runtime length checks. With the default
// Rust representation it would also cause a heap allocation.
// Going with representation 2, with the hope that this will be able to use
// SmallVecs soon.
// TODO (T26959816): add support to represent these as SmallVecs.
typedef binary Blake2 (hs.newtype)

// Allow the hash type to change in the future.
union IdType {
  1: Blake2 Blake2,
}

typedef IdType ChangesetId (hs.newtype)
typedef IdType ContentId (hs.newtype)

// mercurial_types defines Sha1, and it's most convenient to stick this in here.
// This can be moved away in the future if necessary.
typedef binary Sha1 (hs.newtype)

// A path in a repo is stored as a list of elements. This is so that the sort
// order of paths is the same as that of a tree traversal, so that deltas on
// manifests can be applied in a streaming way.
typedef binary MPathElement (hs.newtype)
typedef list<MPathElement> MPath (hs.newtype)

// Parent ordering
// ---------------
// "Ordered" parents means that behavior will change if the order of parents
// changes.
// Whether parents are ordered varies by source control system.
// * In Mercurial, parents are stored ordered and the UI is order-dependent,
//   but are hashed unordered.
// * In Git, parents are stored and hashed ordered and the UI is also order-
//   dependent.
// These data structures will store parents in ordered form, as presented by
// Mercurial. This does hypothetically mean that a single Mercurial changeset
// can map to two Mononoke changesets -- those cases are extremely unlikely
// in practice, and if they're deliberately constructed Mononoke will probably
// end up rejecting whatever comes later.

// Other notes:
// * This uses sorted (B-tree) sets and maps to ensure deterministic
//   serialization.
// * Added and modified files are both part of file_changes.
// * file_changes is at the end of the struct so that a deserializer that just
//   wants to read metadata can stop early.
// * The "required" fields are only for data that is absolutely core to the
//   model. Note that Thrift does allow changing "required" to unqualified.
// * MPath, Id and DateTime fields do not have a reasonable default value, so
//   they must always be either "required" or "optional".
// * The set of keys in file_changes is path-conflict-free (pcf): no changed
//   path is a directory prefix of another path. So file_changes can never have
//   "foo" and "foo/bar" together, but "foo" and "foo1" are OK.
//   * If a directory is replaced by a file, the bonsai changeset will only
//     record the file being added. The directory being deleted is implicit.
//   * This only applies if the potential prefix is changed. Deleted files can
//     have conflicting subdirectory entries recorded for them.
//   * Corollary: The file list in Mercurial is not pcf, so the Bonsai diff is
//     computed separately.
struct BonsaiChangeset {
  1: required list<ChangesetId> parents,
  2: string author,
  3: optional DateTime author_date,
  // Mercurial won't necessarily have a committer, so this is optional.
  4: optional string committer,
  5: optional DateTime committer_date,
  6: string message,
  7: map<string, string> extra,
  8: map<MPath, FileChangeOpt> file_changes,
}

// DateTime fields do not have a reasonable default value! They must
// always be required or optional.
struct DateTime {
  1: required i64 timestamp_secs,
  // Timezones can go up to UTC+13 (which would be represented as -46800), so
  // an i16 can't fit them.
  2: required i32 tz_offset_secs,
}

union FileContents {
  1: binary Bytes,
}

enum FileType {
  Regular = 0,
  Executable = 1,
  Symlink = 2,
}

struct FileChangeOpt {
  // The value being absent here means that the file was deleted.
  1: optional FileChange change,
}

struct FileChange {
  1: required ContentId content_id,
  2: FileType file_type,
  // size is a u64 stored as an i64
  3: required i64 size,
  4: optional CopyInfo copy_from,
}

// This is only used optionally so it is OK to use `required` here.
struct CopyInfo {
  1: required MPath file,
  2: required ChangesetId cs_id,
}
