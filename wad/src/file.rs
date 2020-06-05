const MAX_NAME: usize = 16;

use crate::read_cstring;
use byteorder::{ReadBytesExt, LE};
use std::{
    ffi::CString,
    io::{BufRead, Error as IOError, ErrorKind, Result as IOResult, Seek, SeekFrom},
};

pub struct WadEntry {
    pub name: CString,
    pub entry_type: u8,
    pub size: u32,
    file_pos: u32,
    disk_size: u32,
    compression: bool,
}

impl WadEntry {
    fn read<T: BufRead + Seek>(reader: &mut T) -> IOResult<WadEntry> {
        let file_pos = reader.read_u32::<LE>()?;
        let disk_size = reader.read_u32::<LE>()?;
        let size = reader.read_u32::<LE>()?;
        let entry_type = reader.read_u8()?;
        let compression = reader.read_u8()? != 0;
        let _dummy = reader.read_u16::<LE>()?;
        let name = read_cstring(reader, MAX_NAME)?;

        Ok(WadEntry {
            name,
            entry_type,
            size,
            file_pos,
            disk_size,
            compression,
        })
    }
}

pub struct WadReader<R: BufRead + Seek>(R);

impl<R: BufRead + Seek> From<R> for WadReader<R> {
    fn from(reader: R) -> WadReader<R> {
        WadReader(reader)
    }
}

impl<R: BufRead + Seek> WadReader<R> {
    pub fn create(mut reader: R) -> IOResult<WadReader<R>> {
        reader.seek(SeekFrom::Start(0))?;
        let mut header = [0u8; 4];
        reader.read_exact(&mut header)?;
        if header == [b'W', b'A', b'D', b'3'] {
            Ok(WadReader::from(reader))
        } else {
            Err(IOError::new(
                ErrorKind::InvalidData,
                format!("Wrong header: {:?}", header),
            ))
        }
    }

    pub fn entries(&mut self) -> IOResult<Vec<WadEntry>> {
        self.0.seek(SeekFrom::Start(4))?;
        let count = self.0.read_u32::<LE>()?;
        let offset = self.0.read_u32::<LE>()?;
        self.0.seek(SeekFrom::Start(offset as u64))?;
        (0..count).map(|_| WadEntry::read(&mut self.0)).collect()
    }

    pub fn read_entry(&mut self, entry: &WadEntry) -> IOResult<Vec<u8>> {
        self.0.seek(SeekFrom::Start(entry.file_pos as u64))?;
        let mut data = vec![0; entry.size as usize];
        self.0.read_exact(&mut data)?;
        Ok(data)
    }
}
