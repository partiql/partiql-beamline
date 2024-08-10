use crate::gen::DataGenerationError;
use crate::source::SimSource;
use ion_rs::IonError;
use miette::{Diagnostic, LabeledSpan, SourceCode, SourceSpan};
use thiserror::Error;

pub type ProcessParseResult<T> = Result<T, ProcessParseError>;
pub type ProcessConfigResult<T> = Result<T, ProcessConfigError>;

#[derive(Debug, Error, Diagnostic)]
#[error("Error in process configuration")]
pub struct ProcessParseError {
    // source code for diagnostics
    #[source_code]
    pub script: Option<SimSource>,
    // related may probide labeles spans that will reference into `script`
    #[related]
    pub related: Vec<ProcessConfigError>,
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

    #[error("Random Variable error: `{0}`")]
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

    #[error("Error: `{0}`")]
    UnknownGenerator(String),

    #[error("Error: `{0}`")]
    UnknownArrival(String),

    #[error("Error: `{0}`")]
    UnknownImmediate(String),

    #[error("Error: `{0}`")]
    Other(String),

    #[error("Fatal Internal Error: `{0}`")]
    Fatal(String),
}

#[derive(Debug, Error, Diagnostic)]
#[error("When processing key: `{key}`, Error `{err}`")]
pub struct ConfigValueError {
    pub key: String,
    #[source]
    #[diagnostic_source]
    #[diagnostic(transparent)]
    pub err: ProcessConfigError,
}

#[derive(Debug, Error, Diagnostic)]
#[error("`{generator}` Generator Configuration: {err}")]
pub struct GeneratorConfigError {
    pub generator: String,
    #[source]
    #[diagnostic_source]
    #[diagnostic(transparent)]
    pub err: ProcessConfigError,
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ProcessConfigIonError {
    #[from]
    pub err: IonError,
}

impl From<IonError> for ProcessConfigError {
    fn from(err: IonError) -> Self {
        ProcessConfigIonError::from(err).into()
    }
}

impl Diagnostic for ProcessConfigIonError {}
