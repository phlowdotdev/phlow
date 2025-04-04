pub mod loader;

use loader::{Error, Loader};

use crate::cli::ModuleExtension;

pub fn download(main_path: &str) -> Result<Loader, Error> {
    let modules = Loader::load(main_path, &ModuleExtension::Json)?;

    println!("Modules loaded: {:?}", modules);

    Ok(modules)
}

pub fn load(main_path: &str) -> Result<Loader, Error> {
    let modules = Loader::load(main_path, &ModuleExtension::Json)?;

    println!("Modules loaded: {:?}", modules);

    Ok(modules)
}
