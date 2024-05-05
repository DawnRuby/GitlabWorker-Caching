use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheType {
    Directory,
    File
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsType {
    Windows,
    Unix,
    Unknown
}

impl ToString for OsType {
    /// converts an [`OsType`] enum variant into a string representation.
    /// 
    /// Returns:
    /// [`String`] value depending on operating system entered
    fn to_string(&self) -> String {
        match self{
            OsType::Windows => {return format!("Windows")}
            OsType::Unix => {return format!("Unix")}
            _ => {std::unimplemented!()}
        }
    }
}

impl OsType{
    /// Returns the OsType based on the enviorement variable .
    /// The function `get_ostype` returns the operating system type based on the platform it is running on.
    /// 
    /// Returns:
    /// returns an [`OsType`] enum value.
    pub fn get_ostype() -> OsType{
        if cfg!(windows) {
            return OsType::Windows;
        }else if cfg!(unix){
            return OsType::Unix;
    
        }else{
            return OsType::Unknown;
        }
    }
}