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
fn assert_script_error_snapshot(name: &str, script: impl Into<Vec<u8>>) -> miette::Result<()> {
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

#[test]
fn parse_error_missing_comma() -> miette::Result<()> {
    assert_script_error_snapshot(
        "missing_comma",
        r##"
        rand_processes::{
            sensor: [
                {
                    $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
                    sensor: rand_process::{
                        $data: {
                            id: '1'
                            tick: Tick,
                        }
                    }
                }
            ],
        }
    "##,
    )
}

#[test]
fn parse_error_unknown_generator() -> miette::Result<()> {
    assert_script_error_snapshot(
        "unknown_generator",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    ok: UniformI8,
                    unknown_generator: UniformNonsense,
                    ok2: UniformI8,
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_no_arrival() -> miette::Result<()> {
    assert_script_error_snapshot(
        "no_arrival",
        r##"
        rand_processes::{
            sensor: rand_process::{
                $data: {
                    ok: UniformI8,
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_no_data() -> miette::Result<()> {
    assert_script_error_snapshot(
        "no_data",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{}
        }
    "##,
    )
}

#[test]
fn parse_error_density() -> miette::Result<()> {
    assert_script_error_snapshot(
        "density",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: UniformI8::{nullable:0.7, optional: 0.7},
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_config_expected() -> miette::Result<()> {
    assert_script_error_snapshot(
        "config_expected",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: Regex,
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_config_key_duplicate() -> miette::Result<()> {
    assert_script_error_snapshot(
        "config_key_duplicate",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: Regex::{ pattern: "foo", pattern: "foo" },
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_config_key_extra() -> miette::Result<()> {
    assert_script_error_snapshot(
        "config_key_extra",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: Regex::{ pattern: "foo", xyz_pattern: "foo" },
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_config_key_missing() -> miette::Result<()> {
    assert_script_error_snapshot(
        "config_key_missing",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: LoremIpsum::{ min_words: 2 },
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_config_keys_missing() -> miette::Result<()> {
    assert_script_error_snapshot(
        "config_keys_missing",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: Regex::{  },
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_unknown_arrival() -> miette::Result<()> {
    assert_script_error_snapshot(
        "unknown_arrival",
        r##"
        rand_processes::{
            $arrival: NonsenseArrival:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    ok: UniformI8,
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_arrival_missing_config() -> miette::Result<()> {
    assert_script_error_snapshot(
        "arrival_missing_config",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { nonsense: 5 },
            sensor: rand_process::{
                $data: {
                    ok: UniformI8,
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_unknown_immediate() -> miette::Result<()> {
    assert_script_error_snapshot(
        "unknown_immediate",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: {{ }},
                }
            }
        }
    "##,
    )
}

#[test]
fn parse_error_unknown_variable() -> miette::Result<()> {
    assert_script_error_snapshot(
        "unknown_variable",
        r##"
        rand_processes::{
            $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
            sensor: rand_process::{
                $data: {
                    err: $unknown_variable,
                }
            }
        }
    "##,
    )
}
