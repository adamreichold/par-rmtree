use std::error::Error;
use std::fs::{read_dir, remove_dir, remove_file};
use std::path::Path;

use clap::{command, value_parser, Arg};
use glob::glob;
use rayon::{
    iter::{IntoParallelIterator, ParallelBridge, ParallelIterator},
    ThreadPoolBuilder,
};

fn main() -> Fallible {
    let matches = command!()
        .arg(Arg::new("PATTERNS").required(true).num_args(1..))
        .arg(
            Arg::new("JOBS")
                .short('j')
                .long("jobs")
                .default_value("1")
                .value_parser(value_parser!(usize)),
        )
        .arg(Arg::new("VERBOSE").short('v').long("verbose"))
        .get_matches();

    let patterns = matches
        .get_many::<String>("PATTERNS")
        .unwrap()
        .collect::<Vec<_>>();
    let jobs = *matches.get_one::<usize>("JOBS").unwrap();
    let verbose = matches.get_flag("VERBOSE");

    ThreadPoolBuilder::new().num_threads(jobs).build_global()?;

    fn rmtree(verbose: bool, dir: &Path) -> Fallible {
        read_dir(dir)?
            .par_bridge()
            .try_for_each(|entry| -> Fallible {
                let entry = entry?;
                let path = entry.path();

                if !entry.file_type()?.is_dir() {
                    if verbose {
                        eprintln!("{}", path.display());
                    }

                    remove_file(&path)?;
                } else {
                    rmtree(verbose, &path)?;
                }

                Ok(())
            })?;

        if verbose {
            eprintln!("{}", dir.display());
        }

        remove_dir(dir)?;

        Ok(())
    }

    patterns.into_par_iter().try_for_each(|pattern| {
        glob(pattern)?.par_bridge().try_for_each(|path| {
            let path = path?;

            if !path.is_dir() {
                if verbose {
                    eprintln!("{}", path.display());
                }

                remove_file(&path)?;
            } else {
                rmtree(verbose, &path)?;
            }

            Ok(())
        })
    })
}

type Fallible<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;
