// klc_core::diagnostic — Error and warning reporting
//
// Error codes defined in: docs/14-error-catalog.md

pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
}

pub enum Severity {
    Error,
    Warning,
    Note,
}
