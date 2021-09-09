use std::error::Error;
use std::fs::{read_dir, remove_dir, remove_file};
use std::path::Path;

use clap::{crate_authors, crate_name, crate_version, App, Arg};
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};

fn main() -> Fallible {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(", "))
        .arg(Arg::with_name("DIRS").required(true).multiple(true))
        .arg(
            Arg::with_name("JOBS")
                .short("j")
                .long("jobs")
                .default_value("1"),
        )
        .get_matches();

    let dirs = matches.values_of("DIRS").unwrap().collect::<Vec<_>>();
    let jobs = matches.value_of("JOBS").unwrap().parse::<usize>()?;

    ThreadPoolBuilder::new().num_threads(jobs).build_global()?;

    fn rmtree(dir: &Path) -> Fallible {
        let mut subdirs = Vec::new();
        let mut files = Vec::new();

        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if entry.file_type()?.is_dir() {
                subdirs.push(path);
            } else {
                files.push(path);
            }
        }

        subdirs.into_par_iter().try_for_each(|dir| rmtree(&dir))?;
        files.into_par_iter().try_for_each(remove_file)?;

        remove_dir(dir)?;

        Ok(())
    }

    dirs.into_par_iter()
        .try_for_each(|dir| rmtree(Path::new(dir)))?;

    Ok(())
}

type Fallible<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;
