use ion_rs::{IonError, IonWriter};
use miette::Diagnostic;
use partiql_beamline::sim::{DatasetTypeMapping, SimConfig};
use partiql_types::PartiqlType;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("ShapeEncodingError Error")]
#[non_exhaustive]
pub enum ShapeEncodingError {
    #[error("UnsupportedEncoding: {0}")]
    UnsupportedEncoding(String),
    #[error("IonEncodingError: {0}")]
    IonEncodingError(#[from] IonError),
    #[error("DateTimeEncodingError e: {0}")]
    DateTimeEncodingError(#[from] time::error::Format),
}

/// Result of attempts to encode to Ion.
pub type ShapeEncodeResult<T> = Result<T, ShapeEncodingError>;

/// An encoder which will write [`DatasetTypeMapping`]s as Ion values.
pub trait PartiqlDataSetsEncoder<W, I>
where
    I: IonWriter<Output = W>,
{
    type Output;
    /// A reference to the writer used by this encoder.
    fn writer(&mut self) -> &mut I;

    /// Write an Ion stream value from the given [`DatasetTypeMapping`]
    fn write_datasets(
        &mut self,
        cfg: &SimConfig,
        shapes: DatasetTypeMapping,
    ) -> ShapeEncodeResult<Self::Output>;
}

/// An encoder which will write [`PartiqlType`]s as Ion values.
pub trait PartiqlShapeEncoder<W, I>
where
    I: IonWriter<Output = W>,
{
    /// A reference to the writer used by this encoder.
    fn writer(&mut self) -> &mut I;

    /// Write an Ion stream value from the given [`PartiqlType`]
    fn write_shape(&mut self, shape: &PartiqlType) -> ShapeEncodeResult<()>;
}
