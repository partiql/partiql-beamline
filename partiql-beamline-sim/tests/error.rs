use assert_matches::assert_matches;
use itertools::Itertools;
use miette::{Diagnostic, GraphicalTheme, IntoDiagnostic, MietteHandler, ReportHandler};
use partiql_beamline::sim::{
    ISim, SimBuilder, SimConfigBuilder, SimError, SimResult, DATETIME_FORMAT,
};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding::PartiqlEncodedAsIon;
use partiql_value::{tuple, BindingsName, List, Value};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use time::macros::datetime;

#[track_caller]
fn repeatable_sims(script: &[u8]) -> SimResult<(SimBuilder, SimBuilder)> {
    let config = SimConfigBuilder::default().build()?;
    let t0 = config.t0;
    let seed = config.seed;
    let config2 = SimConfigBuilder::default().t0(t0).seed(seed).build()?;

    Ok((
        SimBuilder::from_config(config, script)?,
        SimBuilder::from_config(config2, script)?,
    ))
}

#[track_caller]
fn verify_repeatable(script: &[u8]) -> SimResult<()> {
    let (sim, sim2) = repeatable_sims(script)?;
    let mut sim = sim.build_time_ordered()?;
    let mut sim2 = sim2.build_time_ordered()?;

    for (sample1, sample2) in sim.iter_mut().zip(sim2.iter_mut()).take(100) {
        assert_eq!(sample1?, sample2?);
    }

    let _test_final = sim.next_sample();

    Ok(())
}

#[track_caller]
fn verify_repeatable_multi(script: &[u8]) -> SimResult<()> {
    let (sim, sim2) = repeatable_sims(script)?;
    let mut sim = sim.build_multi_dataset()?;
    let mut sim2 = sim2.build_multi_dataset()?;

    let ds1 = sim.datasets();
    let ds2 = sim2.datasets();
    assert_eq!(ds1, ds2);

    for (id, _n) in ds1 {
        for (sample1, sample2) in
            std::iter::zip(sim.for_dataset(id)?, sim2.for_dataset(id)?).take(100)
        {
            assert_eq!(sample1?, sample2?);
        }
    }

    Ok(())
}

struct FormatTester<T, E>
where
    T: ReportHandler,
    E: Diagnostic,
{
    handler: T,
    err: E,
}

impl<T, E> Debug for FormatTester<T, E>
where
    T: ReportHandler,
    E: Diagnostic,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.handler.debug(&self.err, f)
    }
}

#[track_caller]
#[inline]
fn assert_error_output_snapshot(name: &str, script: String) -> miette::Result<()> {
    let result = repeatable_sims(script.as_bytes());
    assert!(result.is_err());

    let handler =
        miette::GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
    let err = result.unwrap_err();
    let tester = FormatTester { handler, err };
    let output = format!("{:?}", tester);
    println!("{output}");

    insta::assert_snapshot!(name, output);

    Ok(())
}
#[test]
fn verify_parse_error_regex() -> miette::Result<()> {
    let script_template = r##"
    rand_processes::{
        stuff: rand_process::{
            $r: Uniform::{ choices: [5,10] },
            $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },

            $data: {
                bad: Regex::{ pattern: "$$PATTERN$$" },
            }
        }
    }
    "##;

    let bad_escapes = script_template.replace("$$PATTERN$$", r##"^some val \b\d{1,4}\b$"##);
    assert_error_output_snapshot("bad_escapes", bad_escapes)?;

    let unsupported_lookaround =
        script_template.replace("$$PATTERN$$", r##"^some val \\b\\d{1,4}\\b$"##);
    assert_error_output_snapshot("unsupported_lookaround", unsupported_lookaround)?;

    let unsupported_anchors = script_template.replace("$$PATTERN$$", r##"^some val \\d{1,4}$"##);
    assert_error_output_snapshot("unsupported_anchors", unsupported_anchors)?;

    Ok(())
}
