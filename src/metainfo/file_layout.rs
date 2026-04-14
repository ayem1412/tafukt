use crate::metainfo::info_dictionary_file::InfoDictionaryFile;

#[derive(Debug)]
pub enum FileLayout {
    /// The length of the file, in bytes.
    /// In the single file case, length maps to the length of the file in bytes.
    SingleFile(u64),

    /// For the purposes of the other keys,
    /// the multi-file case is treated as only having a single file by concatenating the files
    /// in the order they appearin the files list.
    /// The files list is the value files maps to.
    MultiFile(Vec<InfoDictionaryFile>),
}

impl FileLayout {
    /// The length of the file, in bytes.
    /// In the single file case, length maps to the length of the file in bytes.
    pub fn length(&self) -> u64 {
        match self {
            FileLayout::SingleFile(length) => *length,
            FileLayout::MultiFile(files) => files.iter().map(|file| file.length).sum(),
        }
    }
}
