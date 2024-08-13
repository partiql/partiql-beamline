use crate::gen::DataGenerationError;
use crate::reader::util::{ToSourceSpan, CONFIG_KEY_NULLABLE, CONFIG_KEY_OPTIONAL};
use crate::source::SimSource;
use ion_rs::IonError;
use miette::{Diagnostic, LabeledSpan, Severity, SourceCode, SourceSpan};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Pointer};
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
        std::fmt::Display::fmt("Error in process configuration", f)
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

    #[error(transparent)]
    #[diagnostic(transparent)]
    DensityError(#[from] SourcedErrorWrapper<DensityError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    RandomVariableError(#[from] SourcedErrorWrapper<DataGenerationError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    NoArrival(#[from] SourcedErrorWrapper<NoArrivalError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    NoData(#[from] SourcedErrorWrapper<NoDataError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ArrivalConfig(Box<ArrivalConfigError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    GeneratorConfig(Box<GeneratorConfigError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ConfigExpected(#[from] SourcedErrorWrapper<ConfigExpectedError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ConfigUnexpected(#[from] SourcedErrorWrapper<ConfigUnexpectedError>),

    #[error("Duplicate Configuration key: `{0}`")]
    ConfigDuplicateKey(String),

    #[error("Unexpected Configuration key: `{0}`")]
    ConfigInvalidKey(String),

    #[error("Did not find expected Configuration key: `{0}`")]
    ConfigMissingKey(String),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ConfigValue(Box<ConfigValueError>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    NotKnown(#[from] SourcedErrorWrapper<NotKnownError>),

    #[error("Unknown Error: `{0}`")]
    Other(String),

    #[error("Fatal Internal Error: `{0}`")]
    Fatal(String),
}

pub(crate) trait Sourceable: Sized {
    fn add_source(&mut self, source: Arc<SimSource>);
    fn add_context(&mut self, span: Option<SourceSpan>);

    fn with_source(mut self, source: Arc<SimSource>) -> Self {
        self.add_source(source);
        self
    }

    fn with_context(mut self, span: Option<SourceSpan>) -> Self {
        self.add_context(span);
        self
    }
}

impl<T, E> Sourceable for Result<T, E>
where
    E: Sourceable,
{
    fn add_source(&mut self, source: Arc<SimSource>) {
        if let Err(err) = self {
            err.add_source(source);
        }
    }

    fn add_context(&mut self, span: Option<SourceSpan>) {
        if let Err(err) = self {
            err.add_context(span);
        }
    }
}

impl Sourceable for ProcessConfigError {
    fn add_source(&mut self, source: Arc<SimSource>) {
        match self {
            ProcessConfigError::ReadError(e) => e.add_source(source),
            ProcessConfigError::DensityError(e) => e.add_source(source),
            ProcessConfigError::RandomVariableError(e) => e.add_source(source),
            ProcessConfigError::NotKnown(e) => e.add_source(source),
            ProcessConfigError::ArrivalConfig(e) => e.add_source(source),
            ProcessConfigError::GeneratorConfig(e) => e.add_source(source),
            ProcessConfigError::ConfigValue(e) => e.add_source(source),
            ProcessConfigError::NoData(e) => e.add_source(source),
            ProcessConfigError::NoArrival(e) => e.add_source(source),
            ProcessConfigError::ConfigExpected(e) => e.add_source(source),
            ProcessConfigError::ConfigUnexpected(e) => e.add_source(source),
            _ => {}
        }
    }

    fn add_context(&mut self, span: Option<SourceSpan>) {
        match self {
            ProcessConfigError::ReadError(e) => e.add_context(span),
            ProcessConfigError::DensityError(e) => e.add_context(span),
            ProcessConfigError::RandomVariableError(e) => e.add_context(span),
            ProcessConfigError::NotKnown(e) => e.add_context(span),
            ProcessConfigError::ArrivalConfig(e) => e.add_context(span),
            ProcessConfigError::GeneratorConfig(e) => e.add_context(span),
            ProcessConfigError::ConfigValue(e) => e.add_context(span),
            ProcessConfigError::NoData(e) => e.add_context(span),
            ProcessConfigError::NoArrival(e) => e.add_context(span),
            ProcessConfigError::ConfigExpected(e) => e.add_context(span),
            ProcessConfigError::ConfigUnexpected(e) => e.add_context(span),
            _ => {}
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[non_exhaustive]
// Deliberately not `pub`
pub(crate) enum NotKnownError {
    #[error("Unknown Generator {0}")]
    Generator(String),
    #[error("Unknown Arrival {0}")]
    Arrival(String),
    #[error("Unknown Immediate {0}")]
    Immediate(String),
    #[error("Unknown Binding {0}")]
    Binding(String),
    #[error("Unknown Variable {0}")]
    Variable(String),
}

impl From<NotKnownError> for ProcessConfigError {
    fn from(err: NotKnownError) -> Self {
        SourcedErrorWrapper::wrap(err).into()
    }
}

#[derive(Error, Default, Debug, Diagnostic)]
#[error("No $arrival for random process")]
pub struct NoArrivalError {}

#[derive(Error, Default, Debug, Diagnostic)]
#[error("No data for random process")]
pub struct NoDataError {}

#[derive(Error, Default, Debug, Diagnostic)]
#[error("Expected Configuration")]
pub struct ConfigExpectedError {}

#[derive(Error, Default, Debug, Diagnostic)]
#[error("Unexpected Configuration")]
pub struct ConfigUnexpectedError {}

#[derive(Error, Debug, Diagnostic)]
pub struct DensityError {
    pub nullable: Option<f64>,
    pub nullable_default: bool,
    pub optional: Option<f64>,
    pub optional_default: bool,
}

impl Display for DensityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fmt_msg = |name: &str, val: Option<f64>, default: bool| {
            format!(
                "{}: `{}`{}",
                name,
                val.unwrap_or(0.0),
                if default {
                    "(from simulation default)"
                } else {
                    ""
                }
            )
        };

        let nullability = fmt_msg(CONFIG_KEY_NULLABLE, self.nullable, self.nullable_default);
        let optionality = fmt_msg(CONFIG_KEY_OPTIONAL, self.optional, self.optional_default);

        let msg = format!(
            "Combined Nullability and Optionality Percents must be between 0.0 and 1.0; {}; {}.",
            nullability, optionality
        );

        std::fmt::Display::fmt(&msg, f)
    }
}

impl From<DensityError> for ProcessConfigError {
    fn from(err: DensityError) -> Self {
        ProcessConfigError::DensityError(SourcedErrorWrapper::wrap(err))
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

impl Sourceable for ConfigValueError {
    fn add_source(&mut self, source: Arc<SimSource>) {
        self.err.add_source(source);
    }

    fn add_context(&mut self, span: Option<SourceSpan>) {
        self.err.add_context(span);
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("`{arrival}` Arrival Configuration")]
pub struct ArrivalConfigError {
    pub arrival: String,
    #[source]
    #[diagnostic_source]
    #[diagnostic(transparent)]
    pub err: ProcessConfigError,
}

impl Sourceable for ArrivalConfigError {
    fn add_source(&mut self, source: Arc<SimSource>) {
        self.err.add_source(source);
    }

    fn add_context(&mut self, span: Option<SourceSpan>) {
        self.err.add_context(span);
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

impl Sourceable for GeneratorConfigError {
    fn add_source(&mut self, source: Arc<SimSource>) {
        self.err.add_source(source);
    }

    fn add_context(&mut self, span: Option<SourceSpan>) {
        self.err.add_context(span);
    }
}

impl From<DataGenerationError> for ProcessConfigError {
    fn from(err: DataGenerationError) -> Self {
        SourcedErrorWrapper::wrap(err).into()
    }
}

#[derive(Debug)]
pub struct SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    pub inner: T,
    pub source_code: Option<Arc<SimSource>>,
    pub source_span: Vec<SourceSpan>,
}

impl<T> Default for SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic + Default,
{
    fn default() -> Self {
        Self::wrap(T::default())
    }
}

impl<T> From<Option<SourceSpan>> for SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic + Default,
{
    fn from(span: Option<SourceSpan>) -> Self {
        Self::default().with_context(span)
    }
}

impl<T> SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    pub fn wrap(inner: T) -> Self {
        Self {
            inner,
            source_code: None,
            source_span: Vec::default(),
        }
    }
}
impl<T> Sourceable for SourcedErrorWrapper<T>
where
    T: std::error::Error + Diagnostic,
{
    fn add_source(&mut self, source: Arc<SimSource>) {
        self.source_code = Some(source);
    }

    fn add_context(&mut self, span: Option<SourceSpan>) {
        // If not empty, context has already been provided deeper in the stack
        if self.source_span.is_empty() {
            if let Some(span) = span {
                self.source_span.push(span);
            }
        }
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
        self.inner.labels().or_else(|| {
            let labels = self
                .source_span
                .iter()
                .map(|span| LabeledSpan::new_with_span(None, *span));
            Some(Box::new(labels))
        })
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item=&'a dyn Diagnostic> + 'a>> {
        self.inner.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.inner.diagnostic_source()
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
        match &self.0 {
            IonError::Io(e) => write!(f, "Ion Io Error: {}", e),
            IonError::Incomplete(_) => write!(f, "Ion Incomplete Error"),
            IonError::Encoding(e) => write!(f, "Ion Encoding Error: {}", e),
            IonError::Decoding(_) => write!(f, "Ion Decoding Error"),
            IonError::IllegalOperation(e) => write!(f, "Ion Illegal Operation Error: {}", e),
            other => write!(f, "Unknown Ion Error: {other}"),
        }
    }
}

impl Error for SimIonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Diagnostic for SimIonError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item=LabeledSpan> + '_>> {
        let name = self.0.to_string();
        let span = LabeledSpan::new_with_span(Some(name), self.0.source_span()?);
        let bx: Box<dyn Iterator<Item=LabeledSpan>> = Box::new(std::iter::once(span));
        Some(bx)
    }
}
