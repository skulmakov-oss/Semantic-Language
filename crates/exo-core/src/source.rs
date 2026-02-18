pub type FileId = u32;

#[cfg(feature = "alloc")]
use alloc::string::ToString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SourceMark {
    pub line: u32,
    pub col: u32,
    pub file_id: FileId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: SourceMark,
    pub end: SourceMark,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub name: alloc::string::String,
    pub source: alloc::string::String,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
    files: alloc::vec::Vec<SourceFile>,
}

#[cfg(feature = "alloc")]
impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, name: &str, source: &str) -> FileId {
        let id = self.files.len() as FileId;
        self.files.push(SourceFile {
            name: name.to_string(),
            source: source.to_string(),
        });
        id
    }

    pub fn file_name(&self, file_id: FileId) -> Option<&str> {
        self.files.get(file_id as usize).map(|f| f.name.as_str())
    }

    pub fn source(&self, file_id: FileId) -> Option<&str> {
        self.files.get(file_id as usize).map(|f| f.source.as_str())
    }

    pub fn line(&self, file_id: FileId, line_no: u32) -> Option<&str> {
        let src = self.source(file_id)?;
        let idx = line_no.saturating_sub(1) as usize;
        src.lines().nth(idx)
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn sourcemap_add_and_lookup() {
        let mut sm = SourceMap::new();
        let fid = sm.add_file("a.exo", "x\ny\n");
        assert_eq!(sm.file_name(fid), Some("a.exo"));
        assert_eq!(sm.line(fid, 1), Some("x"));
        assert_eq!(sm.line(fid, 2), Some("y"));
        assert_eq!(sm.line(fid, 3), None);
    }
}
