// automatically generated by the FlatBuffers compiler, do not modify
// @generated
extern crate alloc;
extern crate flatbuffers;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::mem;

use self::flatbuffers::{EndianScalar, Follow};
use super::*;
pub enum GuestLogDataOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct GuestLogData<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for GuestLogData<'a> {
    type Inner = GuestLogData<'a>;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        unsafe {
            Self {
                _tab: flatbuffers::Table::new(buf, loc),
            }
        }
    }
}

impl<'a> GuestLogData<'a> {
    pub const VT_MESSAGE: flatbuffers::VOffsetT = 4;
    pub const VT_SOURCE: flatbuffers::VOffsetT = 6;
    pub const VT_LEVEL: flatbuffers::VOffsetT = 8;
    pub const VT_CALLER: flatbuffers::VOffsetT = 10;
    pub const VT_SOURCE_FILE: flatbuffers::VOffsetT = 12;
    pub const VT_LINE: flatbuffers::VOffsetT = 14;

    #[inline]
    pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        GuestLogData { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        args: &'args GuestLogDataArgs<'args>,
    ) -> flatbuffers::WIPOffset<GuestLogData<'bldr>> {
        let mut builder = GuestLogDataBuilder::new(_fbb);
        builder.add_line(args.line);
        if let Some(x) = args.source_file {
            builder.add_source_file(x);
        }
        if let Some(x) = args.caller {
            builder.add_caller(x);
        }
        if let Some(x) = args.source {
            builder.add_source(x);
        }
        if let Some(x) = args.message {
            builder.add_message(x);
        }
        builder.add_level(args.level);
        builder.finish()
    }

    #[inline]
    pub fn message(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(GuestLogData::VT_MESSAGE, None)
        }
    }
    #[inline]
    pub fn source(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(GuestLogData::VT_SOURCE, None)
        }
    }
    #[inline]
    pub fn level(&self) -> LogLevel {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<LogLevel>(GuestLogData::VT_LEVEL, Some(LogLevel::Trace))
                .unwrap()
        }
    }
    #[inline]
    pub fn caller(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(GuestLogData::VT_CALLER, None)
        }
    }
    #[inline]
    pub fn source_file(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(GuestLogData::VT_SOURCE_FILE, None)
        }
    }
    #[inline]
    pub fn line(&self) -> u32 {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<u32>(GuestLogData::VT_LINE, Some(0))
                .unwrap()
        }
    }
}

impl flatbuffers::Verifiable for GuestLogData<'_> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        use self::flatbuffers::Verifiable;
        v.visit_table(pos)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>("message", Self::VT_MESSAGE, false)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>("source", Self::VT_SOURCE, false)?
            .visit_field::<LogLevel>("level", Self::VT_LEVEL, false)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>("caller", Self::VT_CALLER, false)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "source_file",
                Self::VT_SOURCE_FILE,
                false,
            )?
            .visit_field::<u32>("line", Self::VT_LINE, false)?
            .finish();
        Ok(())
    }
}
pub struct GuestLogDataArgs<'a> {
    pub message: Option<flatbuffers::WIPOffset<&'a str>>,
    pub source: Option<flatbuffers::WIPOffset<&'a str>>,
    pub level: LogLevel,
    pub caller: Option<flatbuffers::WIPOffset<&'a str>>,
    pub source_file: Option<flatbuffers::WIPOffset<&'a str>>,
    pub line: u32,
}
impl<'a> Default for GuestLogDataArgs<'a> {
    #[inline]
    fn default() -> Self {
        GuestLogDataArgs {
            message: None,
            source: None,
            level: LogLevel::Trace,
            caller: None,
            source_file: None,
            line: 0,
        }
    }
}

pub struct GuestLogDataBuilder<'a: 'b, 'b, A: flatbuffers::Allocator + 'a> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a, A>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b, A: flatbuffers::Allocator + 'a> GuestLogDataBuilder<'a, 'b, A> {
    #[inline]
    pub fn add_message(&mut self, message: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(GuestLogData::VT_MESSAGE, message);
    }
    #[inline]
    pub fn add_source(&mut self, source: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(GuestLogData::VT_SOURCE, source);
    }
    #[inline]
    pub fn add_level(&mut self, level: LogLevel) {
        self.fbb_
            .push_slot::<LogLevel>(GuestLogData::VT_LEVEL, level, LogLevel::Trace);
    }
    #[inline]
    pub fn add_caller(&mut self, caller: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(GuestLogData::VT_CALLER, caller);
    }
    #[inline]
    pub fn add_source_file(&mut self, source_file: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            GuestLogData::VT_SOURCE_FILE,
            source_file,
        );
    }
    #[inline]
    pub fn add_line(&mut self, line: u32) {
        self.fbb_.push_slot::<u32>(GuestLogData::VT_LINE, line, 0);
    }
    #[inline]
    pub fn new(
        _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>,
    ) -> GuestLogDataBuilder<'a, 'b, A> {
        let start = _fbb.start_table();
        GuestLogDataBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<GuestLogData<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

impl core::fmt::Debug for GuestLogData<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("GuestLogData");
        ds.field("message", &self.message());
        ds.field("source", &self.source());
        ds.field("level", &self.level());
        ds.field("caller", &self.caller());
        ds.field("source_file", &self.source_file());
        ds.field("line", &self.line());
        ds.finish()
    }
}
#[inline]
/// Verifies that a buffer of bytes contains a `GuestLogData`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_guest_log_data_unchecked`.
pub fn root_as_guest_log_data(buf: &[u8]) -> Result<GuestLogData, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::root::<GuestLogData>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `GuestLogData` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_guest_log_data_unchecked`.
pub fn size_prefixed_root_as_guest_log_data(
    buf: &[u8],
) -> Result<GuestLogData, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::size_prefixed_root::<GuestLogData>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `GuestLogData` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_guest_log_data_unchecked`.
pub fn root_as_guest_log_data_with_opts<'b, 'o>(
    opts: &'o flatbuffers::VerifierOptions,
    buf: &'b [u8],
) -> Result<GuestLogData<'b>, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::root_with_opts::<GuestLogData<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `GuestLogData` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_guest_log_data_unchecked`.
pub fn size_prefixed_root_as_guest_log_data_with_opts<'b, 'o>(
    opts: &'o flatbuffers::VerifierOptions,
    buf: &'b [u8],
) -> Result<GuestLogData<'b>, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::size_prefixed_root_with_opts::<GuestLogData<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a GuestLogData and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `GuestLogData`.
pub unsafe fn root_as_guest_log_data_unchecked(buf: &[u8]) -> GuestLogData {
    unsafe { flatbuffers::root_unchecked::<GuestLogData>(buf) }
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed GuestLogData and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `GuestLogData`.
pub unsafe fn size_prefixed_root_as_guest_log_data_unchecked(buf: &[u8]) -> GuestLogData {
    unsafe { flatbuffers::size_prefixed_root_unchecked::<GuestLogData>(buf) }
}
#[inline]
pub fn finish_guest_log_data_buffer<'a, 'b, A: flatbuffers::Allocator + 'a>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>,
    root: flatbuffers::WIPOffset<GuestLogData<'a>>,
) {
    fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_guest_log_data_buffer<'a, 'b, A: flatbuffers::Allocator + 'a>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>,
    root: flatbuffers::WIPOffset<GuestLogData<'a>>,
) {
    fbb.finish_size_prefixed(root, None);
}
