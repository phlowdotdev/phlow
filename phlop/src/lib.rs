use loader::Loader;

mod loader;

pub enum Error {
    LoaderError(loader::LoaderError),
}

pub fn download(main_path: Option<String>) -> Result<(), Error> {
    let modules = Loader::load(main_path).map_err(Error::LoaderError)?;

    println!("Modules loaded: {:?}", modules.modules);

    Ok(())
}
