use std::{path::Path, io, fs};

use rustydav::client;

use crate::{envfuncs, datas::enums};

/// The [`copy_recursively`] function copies all files and directories from a source directory to a
/// destination directory
/// 
/// Arguments:
/// 
/// * `from`: type of [`Path`], the source directory or file that you want to copy recursively. It
/// can be any type that can be converted to a [`Path`], such as a [`String`] or [`&str`].
/// * `to`: type of [`Path`], the destination directory where the files and directories will be
/// copied to. It should implement the [`AsRef<Path>`] trait, which means it can be any type that can be
/// converted to a [`Path`] reference.
/// 
/// Returns:
/// Returns a result, indicating if we ran successfully
pub fn copy_recursively(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&to)?;
    
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_recursively(entry.path(), to.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}


/// [`del_restore_dir`] attempts to delete the `.cache` directory and returns an error
/// message if it fails.
/// 
/// Returns:
/// 
/// [`del_restore_dir`] returns an error message if process fails.
pub fn del_restore_dir() -> Result<(), &'static str>{
    if !Path::new(".cache").exists(){
        return Err("We could not find the .cache directory. This maybe due to it being already deleted in the meantime during execution. This should usually not happen.");
    }

    let remove_dir_result_err = fs::remove_dir_all(".cache").is_err();

    if remove_dir_result_err {
        return Err("Failed removing .cache directory. This maybe due to the directory not being found or having too low permissions.")
    }

    return Ok(());
}


/// [`del_webdav_cache`] deletes a file from a webdav server based on the operating system
/// and branch name.
/// 
/// Arguments:
/// 
/// * `ostype`: type of [`enums::OsType`], which represents the Operating system that's currently being used.
/// 
/// Returns:
/// 
/// The function [`del_webdav_cache`] returns an error message if process fails.
pub fn del_webdav_cache(ostype: enums::OsType) -> Result<(), &'static str>{

    if !Path::new(".cache").exists(){
        return Err("We could not find a folder named .cache.");
        
    }

    let webdav_addr = envfuncs::get_webdavaddr();
    let webdav_client = client::Client::init(&envfuncs::get_webdav_user(), &envfuncs::get_webdav_password());
    let file_name = format!{"{}-{}.zip", ostype.to_string(), envfuncs::get_branch_name()};
    let remove_result = webdav_client.delete(format!("{}/gitcache/{}/{}", webdav_addr, envfuncs::get_projectid(), file_name).as_str()).is_err();

    if remove_result {
        return Err("Encountered an error while removing the file from the webdav server");
    }

    return Ok(());
}