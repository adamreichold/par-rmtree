use std::error::Error;
use std::fs::{read_dir, remove_dir, remove_file};
use std::path::Path;

use clap::{crate_authors, crate_name, crate_version, App, Arg};
use glob::glob;
use rayon::{
    iter::{IntoParallelIterator, ParallelBridge, ParallelIterator},
    ThreadPoolBuilder,
};

fn main() -> Fallible {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(", "))
        .arg(Arg::with_name("PATTERNS").required(true).multiple(true))
        .arg(
            Arg::with_name("JOBS")
                .short("j")
                .long("jobs")
                .default_value("1"),
        )
        .get_matches();

    let patterns = matches.values_of("PATTERNS").unwrap().collect::<Vec<_>>();
    let jobs = matches.value_of("JOBS").unwrap().parse::<usize>()?;

    ThreadPoolBuilder::new().num_threads(jobs).build_global()?;

    fn rmtree(dir: &Path) -> Fallible {
        read_dir(dir)?
            .par_bridge()
            .try_for_each(|entry| -> Fallible {
                let entry = entry?;
                let path = entry.path();

                if !entry.file_type()?.is_dir() {
                    remove_file(&path)?;
                } else {
                    rmtree(&path)?;
                }

                Ok(())
            })?;

        remove_dir(dir)?;

        Ok(())
    }

    patterns.into_par_iter().try_for_each(|pattern| {
        glob(pattern)?.par_bridge().try_for_each(|path| {
            let path = path?;

            if !path.is_dir() {
                remove_file(&path)?;
            } else {
                rmtree(&path)?;
            }

            Ok(())
        })
    })
}

type Fallible<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;
