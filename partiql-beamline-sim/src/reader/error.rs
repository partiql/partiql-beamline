use crate::gen::DataGenerationError;
use crate::source::SimSource;
use ion_rs::IonError;
use miette::{
    Diagnostic, LabeledSpan, MietteError, Severity, SourceCode, SourceSpan, SpanContents,
};
use std::any::TypeId;
use std::error::Error;
use std::fmt::{Display, Formatter, Pointer};
use std::ops::Deref;
use std::sync::Arc;
use thiserror::Error;

pub type ProcessParseResult<T> = Result<T, ProcessParseError>;
pub type ProcessConfigResult<T> = Result<T, ProcessConfigError>;

#[derive(Debug, Diagnostic)]
pub struct ProcessParseError {
    // source code for diagnostics
    #[source_code]
    pub source: Option<Arc<SimSource>>,
    #[related]
    pub related: Vec<ProcessConfigError>,
}

impl Display for ProcessParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        "Error in process configuration".fmt(f)
    }
}

impl std::error::Error for ProcessParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl ProcessParseError {
    pub fn new<I>(source: SimSource, related: I) -> Self
    where
        I: IntoIterator<Item=ProcessConfigError>,
    {
        let source = Arc::new(source);
        let mut related: Vec<ProcessConfigError> = related.into_iter().collect();

        for err in &mut related {
            err.add_source(source.clone());
        }
        Self {
            source: Some(source),
            related,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ProcessConfigError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ReadError(#[from] ProcessConfigIonError),

    #[error("Script contains no data")]
    EmptyReadError,

    #[error("Format string error: `{0}`")]
    FormatStringError(String),

    #[error("Nullability/Optionality config error: `{0}`")]
    DensityError(String),

    #[error(transparent)]
    #[diagnostic(transparent)]
    RandomVariableError(#[from] DataGenerationError),

    #[error("No $arrival for random process")]
    NoArrival,

    #[error("No data for random process")]
    NoData,

    #[error(transparent)]
    #[diagnostic(transparent)]
    GeneratorConfig(Box<GeneratorConfigError>),

    #[error("Expected Configuration")]
    ConfigExpected,

    #[error("Unexpected Configuration")]
    ConfigUnexpected,

    #[error("Duplicate Configuration key: `{0}`")]
    ConfigDuplicateKey(String),

    #[error("Unexpected Configuration key: `{0}`")]
    ConfigInvalidKey(String),

    #[error("Did not find expected Configuration key: `{0}`")]
    ConfigMissingKey(String),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ConfigValue(Box<ConfigValueError>),

    #[error("Unknown Generator `{0}`")]
    UnknownGenerator(String),

    #[error("Unknown Arrival `{0}`")]
    UnknownArrival(String),

    #[error("Unknown Immediate Value `{0}`")]
    UnknownImmediate(String),

    #[error("Unknown Error: `{0}`")]
    Other(String),

    #[error("Fatal Internal Error: `{0}`")]
    Fatal(String),
}

impl ProcessConfigError {
    pub(crate) fn add_source(&mut self, source: Arc<SimSource>) {
        match self {
            ProcessConfigError::ReadError(e) => e.add_source(source),
            ProcessConfigError::GeneratorConfig(e) => e.add_source(source),
            ProcessConfigError::ConfigValue(e) => e.add_source(source),
            _ => {}
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Configuration Key `{key}`")]
pub struct ConfigValueError {
    pub key: String,
    #[source]
    #[diagnostic_source]
    #[diagnostic(transparent)]
    pub err: ProcessConfigError,
}

impl ConfigValueError {
    pub(crate) fn add_source(&mut self, source: Arc<SimSource>) {
        self.err.add_source(source);
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("`{generator}` Generator Configuration")]
pub struct GeneratorConfigError {
    pub generator: String,
    #[source]
    #[diagnostic_source]
    #[diagnostic(transparent)]
    pub err: ProcessConfigError,
}

impl GeneratorConfigError {
    pub(crate) fn add_source(&mut self, source: Arc<SimSource>) {
        self.err.add_source(source);
    }
}

#[derive(Debug)]
pub struct SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    pub inner: T,
    pub source_code: Option<Arc<SimSource>>,
}

impl<T> SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    pub fn wrap(inner: T) -> Self {
        Self {
            inner,
            source_code: None,
        }
    }

    pub(crate) fn add_source(&mut self, source: Arc<SimSource>) {
        self.source_code = Some(source);
    }
}

impl<T> Display for SourcedErrorWrapper<T>
where
    T: Diagnostic + std::error::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

impl<T> std::error::Error for SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.source()
    }
}

impl<T> Diagnostic for SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.inner.code()
    }

    fn severity(&self) -> Option<Severity> {
        self.inner.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.inner.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.inner.url()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.source_code.as_ref().map(|sc| sc as &dyn SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item=LabeledSpan> + '_>> {
        self.inner.labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item=&'a dyn Diagnostic> + 'a>> {
        self.inner.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.inner.diagnostic_source()
    }
}

impl<T> From<T> for SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    fn from(inner: T) -> Self {
        SourcedErrorWrapper {
            inner,
            source_code: None,
        }
    }
}

pub type ProcessConfigIonError = SourcedErrorWrapper<SimIonError>;

impl From<IonError> for ProcessConfigError {
    fn from(err: IonError) -> Self {
        ProcessConfigIonError::from(SourcedErrorWrapper::wrap(SimIonError::from(err))).into()
    }
}

#[derive(Debug)]
pub struct SimIonError(IonError);

impl From<IonError> for SimIonError {
    fn from(value: IonError) -> Self {
        Self(value)
    }
}
impl Display for SimIonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ion Decoding Error: {}", self.0)
    }
}

impl Error for SimIonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Diagnostic for SimIonError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item=LabeledSpan> + '_>> {
        let position = match &self.0 {
            IonError::Incomplete(e) => Some(e.position()),
            IonError::Decoding(e) => e.position(),
            _ => None,
        };

        position.map(|pos| {
            let start = pos.byte_offset();
            let len = pos.byte_length();
            let end = start + len.unwrap_or(0);
            let range: SourceSpan = (start..end).into();
            let lspan = LabeledSpan::new_with_span(None, range);
            let iter = std::iter::once(lspan);
            let bx: Box<dyn Iterator<Item=LabeledSpan>> = Box::new(iter);
            bx
        })
    }
}
