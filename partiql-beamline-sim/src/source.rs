use crate::gen::DataGenerationError;
use crate::reader::error::{ProcessConfigError, ProcessParseError, ProcessParseResult};
use crate::sim::SimConfigError;
use ion_rs::{AnyEncoding, IonEncoding, IonResult, Reader};
use miette::{Diagnostic, MietteError, MietteSpanContents, SourceCode, SourceSpan, SpanContents};
use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Error in simulation source.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum SimSourceError {
    #[error("Rand error: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Unknown Error: {0}")]
    UnknownError(Box<dyn Error + Send + Sync + 'static>),
}

pub type SimSourceResult<T> = Result<T, SimSourceError>;

#[derive(Clone)]
pub struct SimSource {
    pub name: String,
    pub data: Vec<u8>,
    pub encoding: Option<IonEncoding>,
}

impl std::fmt::Debug for SimSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimSource")
            .field("name", &self.name)
            .field("data", &"<elided>")
            .field("encoding", &self.encoding);
        Ok(())
    }
}

impl SimSource {
    pub fn from_path(path: impl AsRef<Path>) -> SimSourceResult<Self> {
        let path = path.as_ref();
        let data = fs::read(&path)?;
        let name = path.to_string_lossy();
        Self::new(name, data)
    }

    pub fn new(name: impl AsRef<str>, data: impl Into<Vec<u8>>) -> SimSourceResult<Self> {
        Ok(Self {
            name: name.as_ref().to_string(),
            data: data.into(),
            encoding: None,
        })
    }

    /// Gets the name of this `SimSource`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the data for this `SimSource`.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Sets the [`encoding`](IonEncoding) for this `SimSource`'s data.
    pub fn with_encoding(mut self, encoding: IonEncoding) -> Self {
        self.encoding = Some(encoding);
        self
    }
}

impl SourceCode for SimSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let inner_contents =
            self.data()
                .read_span(span, context_lines_before, context_lines_after)?;
        let mut contents = MietteSpanContents::new_named(
            self.name.clone(),
            inner_contents.data(),
            *inner_contents.span(),
            inner_contents.line(),
            inner_contents.column(),
            inner_contents.line_count(),
        );
        if let Some(encoding) = &self.encoding {
            if encoding.is_text() {
                contents = contents.with_language("ion");
            }
        }
        Ok(Box::new(contents))
    }
}
