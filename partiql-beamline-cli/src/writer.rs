use derive_builder::{Builder, UninitializedFieldError};
use ion_rs::{IonError, IonType, IonWriter};
use miette::Diagnostic;
use partiql_beamline::primitives::{DataSetId, DataSetName, Sample};
use partiql_beamline::sim::{DataSetFilter, DataSetSampler, SampleLimit};
use partiql_beamline::sim::{
    ISim, MultiSim, Sim, SimBuilder, SimConfig, SimError, SimResult, DATETIME_FORMAT,
};
use partiql_beamline::source::SimSource;
use partiql_extension_ion::encode::{
    IonEncodeError, IonEncoderBuilder, IonEncoderConfig, ValueEncoder,
};
use partiql_extension_ion::Encoding;
use std::io::Write;
use std::ops::Add;
use thiserror::Error;
use time::Duration;

#[doc = "Error type for WriterSimSpec Builders"]
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum WriterBuilderError {
    #[error("Sim error: {0}")]
    #[diagnostic(transparent)]
    SimError(#[from] SimError),

    #[error("Ion error: {0}")]
    IonError(#[from] IonError),

    #[error("Uninitialized field `{0}`")]
    UninitializedField(&'static str),

    #[error("Validate error `{0}`")]
    ValidationError(String),
}

type WriterBuilderResult<T> = Result<T, WriterBuilderError>;

impl From<String> for WriterBuilderError {
    fn from(s: String) -> Self {
        Self::ValidationError(s)
    }
}
impl From<UninitializedFieldError> for WriterBuilderError {
    fn from(err: UninitializedFieldError) -> Self {
        Self::UninitializedField(err.field_name())
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "WriterBuilderError"))]
#[non_exhaustive]
pub struct SimSamples {
    count: u64,
}

impl SimSamples {
    fn to_samples(self) -> u64 {
        self.count
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "WriterBuilderError"))]
#[non_exhaustive]
pub struct DataSetFilters {
    filters: Vec<String>,
}

impl DataSetFilters {
    fn to_datasets(self, sim: &impl ISim) -> Vec<(DataSetId, DataSetName)> {
        if self.filters.is_empty() {
            sim.datasets()
        } else {
            self.filters
                .into_iter()
                .map(DataSetName)
                .filter_map(|ds| sim.get_dataset_id(&ds).map(|id| (id, ds)))
                .collect()
        }
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "WriterBuilderError"))]
#[non_exhaustive]
pub struct WriterSim {
    cfg: SimConfig,
    script: SimSource,
    samples: SimSamples,
    dataset_filters: DataSetFilters,
}

impl WriterSim {
    fn parse(self) -> SimResult<(SimBuilder, DataSetFilter, SampleLimit)> {
        let sim = SimBuilder::from_config(self.cfg, self.script)?;
        let filter = DataSetFilter::from_iter(self.dataset_filters.filters);
        let limit = SampleLimit::Constant(self.samples.to_samples());
        Ok((sim, filter, limit))
    }

    fn to_time_ordered(self) -> SimResult<(Sim, DataSetFilter, SampleLimit)> {
        let (sim, filter, limit) = self.parse()?;
        Ok((sim.build_time_ordered()?, filter, limit))
    }
    fn to_multi_sim(self) -> SimResult<(MultiSim, DataSetFilter, SampleLimit)> {
        let (sim, filter, limit) = self.parse()?;
        Ok((sim.build_multi_dataset()?, filter, limit))
    }

    fn to_sampler(self) -> SimResult<DataSetSampler> {
        let (sim, filter, limit) = self.to_multi_sim()?;
        Ok(DataSetSampler::new(sim, filter, limit))
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "WriterBuilderError"))]
#[non_exhaustive]
pub struct WriterText {
    spec: WriterSim,
}

impl WriterText {
    pub fn to_writer(self, out: impl Write + 'static) -> WriterBuilderResult<Box<dyn SimWriter>> {
        let sampler = self.spec.to_sampler()?;
        Ok(Box::new(SimWriterText { sampler, out }))
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "WriterBuilderError"))]
#[non_exhaustive]
pub struct WriterIonCompact {
    spec: WriterSim,
}

impl WriterIonCompact {
    pub fn to_writer(self, out: impl Write + 'static) -> WriterBuilderResult<Box<dyn SimWriter>> {
        let sampler = self.spec.to_sampler()?;
        let writer = ion_rs::TextWriterBuilder::compact().build(out)?;
        Ok(Box::new(SimWriterIon { sampler, writer }))
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "WriterBuilderError"))]
#[non_exhaustive]
pub struct WriterIonPretty {
    spec: WriterSim,
}

impl WriterIonPretty {
    pub fn to_writer(self, out: impl Write + 'static) -> WriterBuilderResult<Box<dyn SimWriter>> {
        let sampler = self.spec.to_sampler()?;
        let writer = ion_rs::TextWriterBuilder::pretty().build(out)?;
        Ok(Box::new(SimWriterIon { sampler, writer }))
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "WriterBuilderError"))]
#[non_exhaustive]
pub struct WriterIonBinary {
    spec: WriterSim,
}

impl WriterIonBinary {
    pub fn to_writer(self, out: impl Write + 'static) -> WriterBuilderResult<Box<dyn SimWriter>> {
        let sampler = self.spec.to_sampler()?;
        let writer = ion_rs::BinaryWriterBuilder::new().build(out)?;
        Ok(Box::new(SimWriterIon { sampler, writer }))
    }
}

/// Error during simulation
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum SimWriterError {
    #[error("Ion error: {0}")]
    IonError(#[from] IonError),

    #[error("Ion Encode error: {0}")]
    IonEncodeError(#[from] IonEncodeError),

    #[error("{0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    TimeFormat(#[from] time::error::Format),

    #[error(transparent)]
    #[diagnostic(transparent)]
    SimError(#[from] SimError),
}

pub type SimWriterResult<T> = Result<T, SimWriterError>;

pub trait SimWriter {
    fn write(&mut self) -> SimWriterResult<()>;
}

pub struct SimWriterText<W>
where
    W: Write,
{
    sampler: DataSetSampler,
    out: W,
}

impl<W> SimWriter for SimWriterText<W>
where
    W: Write,
{
    fn write(&mut self) -> SimWriterResult<()> {
        let seed = self.sampler.seed();
        let t0 = self.sampler.t0();
        writeln!(self.out, "Seed: {}", seed)?;
        writeln!(self.out, "Start: {}", t0.format(&DATETIME_FORMAT)?)?;
        for (dataset, samples) in self.sampler.iter_mut() {
            for sample in samples {
                let Sample { tick, value } = sample?;
                let time = t0.add(Duration::milliseconds(tick.0 as i64));
                writeln!(self.out, "[{time}] : {dataset:?} {value:?}")?;
            }
        }

        Ok(())
    }
}

pub struct SimWriterIon<W, I>
where
    W:,
    I: IonWriter<Output = W>,
{
    sampler: DataSetSampler,
    writer: I,
}

impl<W, I> SimWriter for SimWriterIon<W, I>
where
    W:,
    I: IonWriter<Output = W>,
{
    fn write(&mut self) -> SimWriterResult<()> {
        let seed = self.sampler.seed();
        let t0 = self.sampler.t0();
        let start = t0.format(&DATETIME_FORMAT)?;

        let w = &mut self.writer;
        w.step_in(IonType::Struct)?;
        {
            w.set_field_name("seed");
            w.write_i64(seed as i64)?;
            w.set_field_name("start");
            w.write_string(start)?;
            w.set_field_name("data");
            w.step_in(IonType::Struct)?;
            {
                for (dataset, samples) in self.sampler.iter_mut() {
                    w.set_field_name(dataset.0.as_str());
                    w.step_in(IonType::List)?;
                    {
                        let ion_mode = IonEncoderConfig::default().with_mode(Encoding::Ion);
                        let mut encoder = IonEncoderBuilder::new(ion_mode).build(w)?;
                        for sample in samples {
                            encoder.write_value(&sample?.value)?
                        }
                    }
                    w.step_out()?;
                }
            }
            w.step_out()?;
        }
        w.step_out()?;

        w.flush()?;

        Ok(())
    }
}
