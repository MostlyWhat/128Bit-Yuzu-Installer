//! Provides interfaces to various archives.

use zip::ZipArchive as UpstreamZipArchive;

use tar::Archive as UpstreamTarArchive;
use tar::EntryType;

use std::io::Cursor;
use std::io::Read;
use std::iter::Iterator;
use std::path::PathBuf;

use xz2::read::XzDecoder;

pub trait Archive<'a> {
    /// func: iterator value, max size, file name, file contents
    fn for_each(
        &mut self,
        func: &mut dyn FnMut(usize, Option<usize>, PathBuf, &mut dyn Read) -> Result<(), String>,
    ) -> Result<(), String>;
}

struct ZipArchive<'a> {
    archive: UpstreamZipArchive<Cursor<&'a [u8]>>,
}

impl<'a> Archive<'a> for ZipArchive<'a> {
    fn for_each(
        &mut self,
        func: &mut dyn FnMut(usize, Option<usize>, PathBuf, &mut dyn Read) -> Result<(), String>,
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
    archive: UpstreamTarArchive<Box<dyn Read + 'a>>,
}

impl<'a> Archive<'a> for TarArchive<'a> {
    fn for_each(
        &mut self,
        func: &mut dyn FnMut(usize, Option<usize>, PathBuf, &mut dyn Read) -> Result<(), String>,
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
pub fn read_archive<'a>(name: &str, data: &'a [u8]) -> Result<Box<dyn Archive<'a> + 'a>, String> {
    if name.ends_with(".zip") {
        // Decompress a .zip file
        let archive = UpstreamZipArchive::new(Cursor::new(data))
            .map_err(|x| format!("Error while reading .zip file: {:?}", x))?;

        Ok(Box::new(ZipArchive { archive }))
    } else if name.ends_with(".tar.xz") {
        // Decompress a .tar.xz file
        let mut decompresser = XzDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decompresser
            .read_to_end(&mut decompressed_data)
            .map_err(|x| format!("Failed to decompress data: {:?}", x))?;

        let decompressed_contents: Box<dyn Read> = Box::new(Cursor::new(decompressed_data));

        let tar = UpstreamTarArchive::new(decompressed_contents);

        Ok(Box::new(TarArchive { archive: tar }))
    } else {
        Err(format!("No decompression handler for {:?}.", name))
    }
}
