use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use spin::Mutex;

lazy_static! {
    pub static ref FILESYSTEM: Mutex<RamFileSystem> = Mutex::new(RamFileSystem::new());
}

pub struct RamFile {
    pub name: String,
    pub content: Vec<u8>
}

pub struct RamFileSystem {
    pub files: BTreeMap<String, RamFile>
}

impl RamFileSystem {
    pub fn new() -> Self {
        Self { files: BTreeMap::new() }
    }

    pub fn write_file(&mut self, path: String, content: Vec<u8>) {
        self.files.insert(path.clone(), RamFile { name: path, content });
    }

    pub fn read_file(&mut self, path: String) -> Vec<u8> {
        self.files.get(path).map(|f| &f.content)
    }

    pub fn delete_file(&mut self, path: String) -> bool {
        self.files.remove(path).is_some();
    }
}