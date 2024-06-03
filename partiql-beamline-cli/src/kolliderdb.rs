use crate::cli::{encode_ion_text, get_multi_sim, IonPrintMode};
use ion_rs::element::writer::TextKind;
use partiql_beamline::sim::{SimConfig, DATETIME_FORMAT};
use partiql_beamline_serde::kollider::PartiqlKolliderEncoder;
use partiql_beamline_serde::serde::PartiqlShapeEncoder;
use partiql_extension_ion::Encoding;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use time::OffsetDateTime;

pub(crate) fn create_kollider_db(
    cfg: &SimConfig,
    catalog_name: &str,
    catalog_path: &str,
    script: &str,
    sample_count: u64,
) -> miette::Result<()> {
    let catalog_full_path = format!("{catalog_path}/{catalog_name}/");
    let mut sim = get_multi_sim(cfg, script).expect("sim");

    print!("writing shape file(s)...");
    let shapes = sim.shape();
    for (dataset, ty) in shapes.into_iter() {
        let mut out: Vec<u8> = Vec::new();
        let mut writer = ion_rs::TextWriterBuilder::new(TextKind::Pretty)
            .build(&mut out)
            .expect("pretty writer");
        let mut encoder = PartiqlKolliderEncoder::new(&mut writer);
        encoder.write_shape(&ty).expect("write shape");
        drop(writer);

        let mut dataset_shape_file =
            fs::File::create(format!("{:}/{dataset}.shape.ion", &catalog_full_path))
                .expect("dataset file");
        dataset_shape_file
            .write_all(out.as_slice())
            .expect("write data set file");
        drop(out)
    }

    println!("[COMPLETED]");
    print!("writing data file(s)...");
    let sim_datasets = sim.datasets();
    for (id, ds_n) in sim_datasets {
        let filename = ds_n.clone().0;
        let mut dataset_file = fs::File::create(format!("{:}/{filename}.ion", &catalog_full_path))
            .expect("dataset file");
        let sim = sim.for_dataset(id);
        sim.iter_mut().take(sample_count as usize).for_each(|s| {
            let s = s.expect("value");
            let val = s.value;
            let out =
                encode_ion_text(IonPrintMode::Compact, &val, Encoding::Ion).expect("Ion value");
            dataset_file
                .write_all(format!("{:}\n", out).as_bytes())
                .expect("write file");
        });
    }

    println!("[COMPLETED]");
    println!("done!");

    Ok(())
}

pub(crate) fn create_catalog_dir(
    force: bool,
    catalog_name: &str,
    catalog_path: &str,
) -> miette::Result<()> {
    let catalog_full_path = catalog_full_path(catalog_name, catalog_path);
    let catalog_exists = Path::new(&catalog_full_path).exists();

    if force {
        println!("command is using --force ...");
        if catalog_exists {
            let catalog_bkp_path = format!(
                "{:}.{:}.bkp",
                &catalog_name,
                OffsetDateTime::now_utc()
                    .format(&DATETIME_FORMAT)
                    .expect("now utc")
            );
            println!(
                "Beamline catalog {:} exists, backing it up to {:?}...",
                &catalog_full_path, &catalog_bkp_path
            );
            fs::rename(
                &catalog_full_path,
                format!("{:}/{:}", &catalog_path, &catalog_bkp_path),
            )
            .expect("catalog backup");
            println!("back up completed");
            fs::create_dir_all(&catalog_full_path).expect("create dir");
            Ok(())
        } else {
            fs::create_dir(&catalog_full_path).expect("create dir");
            Ok(())
        }
    } else {
        match fs::create_dir(&catalog_full_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!(
                    "creating directory {:} failed with the following error:\n{:}",
                    &catalog_full_path, e
                );
                exit(1);
            }
        }
    }
}

pub(crate) fn create_manifest_file(cfg: &SimConfig, catalog_full_path: &str) -> miette::Result<()> {
    let manifest_filename = format!("{:}.beamline-manifest", catalog_full_path);
    print!("writing manifest file {:} ...", &manifest_filename);
    let mut manifest_file = fs::File::create(&manifest_filename).expect("manifest file");
    manifest_file
        .write_all(
            format!(
                "{{\"seed\": \"{:}\", \"start\": \"{:}\" }}",
                &cfg.seed,
                &cfg.t0.format(&DATETIME_FORMAT).expect("format"),
            )
            .as_bytes(),
        )
        .expect("manifest file");

    println!("[COMPLETED]");

    Ok(())
}

pub(crate) fn create_script_file(catalog_full_path: &str, script: &str) -> miette::Result<()> {
    let script_filename = format!("{:}.beamline-script", catalog_full_path);
    print!("writing script file {:} ...", &script_filename);
    let mut script_file = fs::File::create(&script_filename).expect("script file");
    script_file
        .write_all(script.as_bytes())
        .expect("script file");

    println!("[COMPLETED]");

    Ok(())
}

pub(crate) fn catalog_full_path(catalog_name: &str, catalog_path: &str) -> String {
    format!("{catalog_path}/{catalog_name}/")
}
