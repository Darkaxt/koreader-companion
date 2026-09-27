# Case-insensitive EPUB extensions

## Authoritative specification

KOmpanion must index EPUB files whose extension uses any ASCII capitalization,
including `.epub`, `.EPUB`, and mixed-case forms such as `.EpUb`.

Requirements:

- R1: `EpubIndexer` accepts every ASCII capitalization of the `.epub`
  extension while continuing to reject unrelated extensions and suffixes.
- R2: the generated KPM registration routes every ASCII capitalization of the
  EPUB extension to the KOmpanion extractor.
- R3: existing lowercase EPUB behavior and the registration behavior of other
  supported formats remain unchanged.
- R4: the installed Kindle package indexes the two existing uppercase Spanish
  EPUB examples in `/mnt/us/documents/Books` without altering the originals on
  the shared drive.
- R5: opening either newly indexed Library entry launches that exact file in
  KOReader.
- R6: each indexed book's Library cover remains available after KOReader opens
  and closes the book, without overwriting KOReader metadata or a user-selected
  custom cover.

## Staged execution and reconciliation

### Stage 1 - Source contract and host verification (`COMPLETE`)

- Covers: R1, R2, R3.
- Changes: add failing regression tests, implement case-insensitive EPUB
  matching, and generate all EPUB case-variant extractor registrations.
- Verification: focused regression tests, relevant package-generator tests,
  formatting, and the repository test suite.

Verification evidence: the EPUB and generator regression tests failed against
the original implementation, then passed after the change. The Linux test run
used the same package selection as upstream CI and passed for both the extractor
and launcher. The generated `install.sql` contains exactly 16 EPUB extractor
globs. The repository's pre-existing formatting drift is outside this change;
the changed lines are formatted consistently.

### Stage 2 - Kindle integration and persistent covers (`COMPLETE`)

- Covers: R4, R5, R6.
- Changes: build a Kindle HF package, install it over KOmpanion 0.2.0, and
  trigger a scanner reload. Store generated thumbnails in KOReader's real
  extensionless sidecar directory under a KOmpanion-specific filename, and
  remove only KOmpanion-owned current and legacy cover files.
- Verification: the device log records successful metadata extraction for both
  exact `.EPUB` files; both appear in the native Library; each Library entry
  opens the matching file in KOReader; each cover remains after opening and
  closing the corresponding book; KOReader metadata and custom covers remain
  untouched.

Verification evidence: the two original uppercase Spanish EPUBs indexed under
their embedded Spanish titles and launched the exact files in KOReader. The
initial post-open filesystem state reproduced the R6 failure: KOmpanion's
`book.EPUB.sdr` directories were empty while KOReader wrote reading state to
`book.sdr/metadata.EPUB.lua`. After the fix, both generated covers remained in
the real sidecars as `kompanion-cover.*` after repeated open/close cycles, the
metadata files remained unchanged during deployment, and the legacy sidecars
were removed. A third real Spanish `.EPUB` was then copied byte-for-byte from
the shared drive and passed the fresh Add, Library display, launch, and
post-close cover-persistence workflow. Its exact device copy and sidecar were
removed afterward; the shared-drive source and all other books were untouched.

### Stage 3 - Final reconciliation and cleanup (`COMPLETE`)

- Covers: all requirements.
- Changes: remove task-owned diagnostic and staging artifacts, then commit the
  verified source change.
- Verification: blockers and tracked deferrals are both zero, device diagnostic
  artifacts are absent, host cleanup transaction is closed, and the committed
  tree is clean.

Verification evidence: the exact third-book device test copy and its sidecar
were removed, device deployment staging paths were absent, and the untouched
shared-drive source remained available. Transactional cleanup removed the exact
`D:\Temp\kompanion-uppercase-fix` host tree, including its reviewed toolchain
and disposable repository remnants, and confirmed that the root no longer
exists. Final host verification passed for the extractor and launcher, and the
completed source tree was committed with no remaining worktree changes.

Current blockers: none.

Tracked deferrals: none.
