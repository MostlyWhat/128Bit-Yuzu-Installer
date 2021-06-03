//! Provides interfaces to various archives.

use zip::ZipArchive as UpstreamZipArchive;

use tar::Archive as UpstreamTarArchive;
use tar::EntryType;

use std::io::Cursor;
use std::io::Read;
use std::iter::Iterator;
use std::path::PathBuf;

use xz_decom;

use flate2::read::GzDecoder as gz_decom;

pub trait Archive<'a> {
    /// func: iterator value, max size, file name, file contents
    fn for_each(
        &mut self,
        func: &mut FnMut(usize, Option<usize>, PathBuf, &mut Read) -> Result<(), String>,
    ) -> Result<(), String>;
}

struct ZipArchive<'a> {
    archive: UpstreamZipArchive<Cursor<&'a [u8]>>,
}

impl<'a> Archive<'a> for ZipArchive<'a> {
    fn for_each(
        &mut self,
        func: &mut FnMut(usize, Option<usize>, PathBuf, &mut Read) -> Result<(), String>,
    ) -> Result<(), String> {
        let max = self.archive.len();

        for i in 0..max {
            let mut archive = self
                .archive
                .by_index(i)
                .map_err(|v| format!("Error while reading from .zip file: {:?}", v))?;

            if archive.name().ends_with('/') || archive.name().ends_with('\\') {
                continue;
            }

            func(i, Some(max), archive.sanitized_name(), &mut archive)?;
        }

        Ok(())
    }
}

struct TarArchive<'a> {
    archive: UpstreamTarArchive<Box<Read + 'a>>,
}

impl<'a> Archive<'a> for TarArchive<'a> {
    fn for_each(
        &mut self,
        func: &mut FnMut(usize, Option<usize>, PathBuf, &mut Read) -> Result<(), String>,
    ) -> Result<(), String> {
        let entries = self
            .archive
            .entries()
            .map_err(|x| format!("Error while reading .tar file: {:?}", x))?;

        for (i, entry) in entries.enumerate() {
            let mut entry =
                entry.map_err(|v| format!("Failed to read entry from .tar file: {:?}", v))?;

            if entry.header().entry_type() != EntryType::Regular {
                continue;
            }

            let path = entry
                .path()
                .map(PathBuf::from)
                .map_err(|v| format!("Failed to read entry from .tar file: {:?}", v))?;

            func(i, None, path, &mut entry)?;
        }

        Ok(())
    }
}

/// Reads the named archive with an archive implementation.
pub fn read_archive<'a>(name: &str, data: &'a [u8]) -> Result<Box<Archive<'a> + 'a>, String> {
    if name.ends_with(".zip") {
        // Decompress a .zip file
        let archive = UpstreamZipArchive::new(Cursor::new(data))
            .map_err(|x| format!("Error while reading .zip file: {:?}", x))?;

        Ok(Box::new(ZipArchive { archive }))
    } else if name.ends_with(".tar.xz") {
        // Decompress a .tar.xz file
        let decompressed_data = xz_decom::decompress(data)
            .map_err(|x| format!("Failed to build decompressor: {:?}", x))?;

        let decompressed_contents: Box<Read> = Box::new(Cursor::new(decompressed_data));

        let tar = UpstreamTarArchive::new(decompressed_contents);

        Ok(Box::new(TarArchive { archive: tar }))
    } else if name.ends_with(".tar.gz") {
        // Decompress a .tar.gz file
        let decompressed_data = gz_decom::new(data);

        let decompressed_contents: Box<Read> = Box::new(decompressed_data);

        let tar = UpstreamTarArchive::new(decompressed_contents);

        Ok(Box::new(TarArchive { archive: tar }))
    } else {
        Err(format!("No decompression handler for {:?}.", name))
    }
}
