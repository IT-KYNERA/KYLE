// klc_runtime::error — Error type for KL runtime

pub struct KlError {
    pub message: String,
    pub code: Option<i32>,
}
