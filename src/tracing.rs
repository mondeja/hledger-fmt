use ctor::ctor;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));

    let subscriber = fmt()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .with_test_writer()
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}

#[ctor]
unsafe fn init() {
    init_tracing();
}

pub(crate) struct Utf8Slice<'a>(pub(crate) &'a [u8]);

impl<'a> std::fmt::Display for Utf8Slice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.0))
    }
}

impl<'a> std::fmt::Debug for Utf8Slice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(self.0))
    }
}
