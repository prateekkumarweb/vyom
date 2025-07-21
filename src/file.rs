use jiff::Timestamp;

pub struct FileMetadata {
    name: String,
    size: u64,
    created_at: Timestamp,
    modified_at: Timestamp,
}
