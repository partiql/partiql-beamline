use miette::{Diagnostic, GraphicalTheme, ReportHandler};
use partiql_beamline::sim::{SimBuilder, SimConfigBuilder, SimResult};
use partiql_beamline::source::SimSource;
use std::fmt;
use std::fmt::Debug;

#[track_caller]
fn get_sim(source: SimSource) -> SimResult<SimBuilder> {
    let config = SimConfigBuilder::default().build()?;
    SimBuilder::from_config(config, source)
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
fn assert_script_error_snapshot(name: &str, script: String) -> miette::Result<()> {
    let source_name = format!("{name}.script");
    let source = SimSource::new(source_name, script)?;
    let result = get_sim(source);
    assert!(result.is_err());
    assert_error_output_snapshot(name, result.unwrap_err())
}

#[track_caller]
#[inline]
fn assert_error_output_snapshot<E>(name: &str, err: E) -> miette::Result<()>
where
    E: Diagnostic,
{
    dbg!(&err);
    let handler =
        miette::GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());

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
    assert_script_error_snapshot("bad_escapes", bad_escapes)?;

    let unsupported_lookaround =
        script_template.replace("$$PATTERN$$", r##"^some val \\b\\d{1,4}\\b$"##);
    assert_script_error_snapshot("unsupported_lookaround", unsupported_lookaround)?;

    let unsupported_anchors = script_template.replace("$$PATTERN$$", r##"^some val \\d{1,4}$"##);
    assert_script_error_snapshot("unsupported_anchors", unsupported_anchors)?;

    Ok(())
}
