pub mod jobs;
pub mod process;
pub mod worker;

pub use jobs::*;
pub use process::process_ocr;

#[derive(Clone, serde::Serialize)]
pub struct OcrUpdate {
    pub user_id: String,
    pub job_id: String,
    pub status: String,
}
