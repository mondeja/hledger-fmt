fn create_env_filter() -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::new("trace")
}

#[cfg(not(test))]
pub(crate) fn init_file_tracing(path: &std::path::Path) {
    let filter = create_env_filter();
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
        .with_level(false)
        .with_ansi(false);

    _ = std::fs::File::create(path).expect(&format!("failed to create file {path:?}"));
    let file_appender = tracing_appender::rolling::never(
        path.parent().unwrap_or_else(|| {
            panic!("Invalid trace file path: {:?}", path);
        }),
        path.file_name().unwrap_or_else(|| {
            panic!("Invalid trace file path: {:?}", path);
        }),
    );
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    std::mem::forget(guard);
    let subscriber = builder.with_writer(non_blocking).finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}

#[cfg(test)]
#[ctor::ctor]
fn init_stdout_tracing() {
    let filter = create_env_filter();
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .with_test_writer()
        .with_test_writer()
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
        .with_level(false);
    let subscriber = builder.finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
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
