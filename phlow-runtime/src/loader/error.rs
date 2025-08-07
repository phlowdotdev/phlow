use phlow_sdk::valu3;
use std::fmt::Display;
use zip::result::ZipError;

pub struct ModuleError {
    pub module: String,
}

impl Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module: {}", self.module)
    }
}

pub enum Error {
    VersionNotFound(ModuleError),
    ModuleLoaderError(String),
    ModuleNotFound(String),
    StepsNotDefined,
    LibLoadingError(libloading::Error),
    LoaderErrorJsonValu3(valu3::Error),
    LoaderErrorScript(serde_yaml::Error),
    GetFileError(reqwest::Error),
    FileCreateError(std::io::Error),
    ZipErrorError(ZipError),
    BufferError(reqwest::Error),
    CopyError(std::io::Error),
    MainNotFound(String),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::VersionNotFound(err) => write!(f, "Version not found: {}", err),
            Error::ModuleLoaderError(err) => write!(f, "Module loader error: {}", err),
            Error::StepsNotDefined => write!(f, "Steps not defined"),
            Error::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            Error::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
            Error::LoaderErrorJsonValu3(err) => write!(f, "Json Valu3 error: {:?}", err),
            Error::LoaderErrorScript(err) => write!(f, "Script error: {:?}", err),
            Error::GetFileError(err) => write!(f, "Get file error: {:?}", err),
            Error::FileCreateError(err) => write!(f, "File create error: {:?}", err),
            Error::BufferError(err) => write!(f, "Buffer error: {:?}", err),
            Error::CopyError(err) => write!(f, "Copy error: {:?}", err),
            Error::ZipErrorError(err) => write!(f, "Zip error: {:?}", err),
            Error::MainNotFound(err) => write!(f, "Main not found: {:?}", err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::VersionNotFound(err) => write!(f, "Version not found: {}", err),
            Error::ModuleLoaderError(err) => write!(f, "Module loader error: {}", err),
            Error::StepsNotDefined => write!(f, "Steps not defined"),
            Error::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            Error::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
            Error::LoaderErrorJsonValu3(err) => write!(f, "Json Valu3 error: {:?}", err),
            Error::LoaderErrorScript(err) => write!(f, "Yaml error: {:?}", err),
            Error::GetFileError(err) => write!(f, "Get file error: {:?}", err),
            Error::FileCreateError(err) => write!(f, "File create error: {:?}", err),
            Error::BufferError(err) => write!(f, "Buffer error: {:?}", err),
            Error::CopyError(err) => write!(f, "Copy error: {:?}", err),
            Error::ZipErrorError(err) => write!(f, "Zip error: {:?}", err),
            Error::MainNotFound(err) => write!(f, "Main not found: {:?}", err),
        }
    }
}
