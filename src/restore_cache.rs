use std::{fs::{self, File}, path::Path, io::Read};
use crate::{datas::{structs::{self, RestoreData}, enums}, helpers, envfuncs};

/// The main function restores the .cache folder to the locations indicated by the data.json file.
/// 
/// Returns:
/// `main` returns an error code if process fails indicating where failure happened.
pub fn main() -> Result<(), i32>{
    println!("Welcome to the caching restore tool.
    \nThis will restore your .cache folder to the locations indicated by the data.json file.");

    if !Path::new(".cache").exists(){
        eprintln!("We could not find the .cache file. Please ensure that the folder is at the correct location.");
        return Err(51);
    }

    if !Path::new(".cache/data.json").exists(){
        eprintln!("The .cache folder does not contain a data.json file. This is needed to restore the items to the locations they're supposed to be.");
        return Err(52);
    }

    let folders = envfuncs::get_env_if_startswith("cachepath_");
    let files = envfuncs::get_env_if_startswith("cachefile_");

    if (folders.len() == 0)  && (files.len() == 0){
        eprintln!("Found 0 directories or files via searching env vars that start with cachepath_ or cachefile_. 
        \nPlease make sure something is part of the cachepath_ or cachefile_ enviorement variables so we know what to restore.");
        return Err(21);
    }

    let json_file = File::open(".cache/data.json");

    if json_file.is_err(){
        eprintln!("Could not read json file. This maybe because it's corrupt or we have invalid permissions. Please check your permissions and try again");
        return Err(53);
    }

    let mut json_file = json_file.unwrap();
    let mut data = String::new();
    let data_read_is_err = json_file.read_to_string(&mut data).is_err();

    if data_read_is_err {
        eprintln!("Could not read json file. This maybe because it's corrupt or we have invalid permissions. Please check your permissions and try again");
        return Err(54);
    }

    let seralized_data = serde_json::from_str(data.as_str());

    if seralized_data.is_err(){
        eprintln!("Encountered an issue while attempting to deserialize data.json file");
        return Err(55);
    }

    let seralized_data: Vec<structs::RestoreData> = seralized_data.unwrap();

    if seralized_data.is_empty(){
        eprintln!("Encountered an issue while attempting to deserialize data.json file");
    }

    println!("Found .cache and data.json file both of which are valid. Starting restore process");
    let restore_res = restore_data(seralized_data, folders, files);

    if restore_res.is_err(){
        eprintln!("Encountered an unrecoverable error during restore process. 
        \nError was: {}", restore_res.unwrap_err());
        return Err(56);
    }

    let del_restore_dir_res = helpers::del_restore_dir();

    if del_restore_dir_res.is_err(){
        eprintln!("Encountered an issue attempting to delete the restore directory. 
        \nExiting with an error to prevent issues creating cache.
        \nError was: {}", del_restore_dir_res.unwrap_err());
        return Err(57);
    }

    return Ok(());
}


/// `restore_data` takes in a [`Vec<RestoreData>`], as well as vectors of folder
/// and file paths, and attempts to restore the data to the designated locations. 
/// Compares the `res_data_vec` with `restore_folder_paths` and `restore_file_paths` to determine if we should restore the item or not.
/// 
/// Arguments:
/// 
/// * `res_data_vec`: type of [`Vec<RestoreData>`] which contains information
/// about the data to be restored, such as the cache type (directory or file), the restore destination,
/// and the name of the object to be restored.
/// * `restore_folder_paths`: type of [`Vec<String>`] representing the paths to the folders where the data
/// should be restored.
/// * `restore_file_paths`: type of [`Vec<String>`] representing the paths to the files that need to be
/// restored.
/// 
/// Returns:
/// Returns an error message if we failed.
pub fn restore_data(res_data_vec: Vec<RestoreData>, restore_folder_paths: Vec<String>, restore_file_paths: Vec<String>) -> Result<(), &'static str>{
    if res_data_vec.is_empty(){
        return Err("The array you entered is empty. Please give us an array with at least one function.");
    }

    let mut error_count = 0;

    for restore_obj in res_data_vec.clone(){
        match restore_obj.cachetype{
            enums::CacheType::Directory => {     
                //Check if our Item is in the enviorement variables. If not skip.
                if !restore_folder_paths.contains(&format!("{}/{}", restore_obj.restore_to, restore_obj.restore_obj_name)){
                    continue;
                }            

                let restore_file_res = restore_folder(restore_obj.clone());

                if restore_file_res.is_err() {
                   println!("Encountered an error attempting to restore File named: {}", restore_obj.restore_obj_name);
                   println!("The Error was:
                            \n{}", restore_file_res.unwrap_err());
                   error_count = error_count +1;
                }
            }
            enums::CacheType::File => {

                //Check if our Item is in the enviorement variables. If not skip.
                if !restore_file_paths.contains(&format!("{}/{}", restore_obj.restore_to, restore_obj.restore_obj_name)){
                    continue;
                }

                 let restore_file_res = restore_file(restore_obj.clone());

                 if restore_file_res.is_err() {
                    println!("Encountered an error attempting to restore File named: {}", restore_obj.restore_obj_name);
                    println!("The Error was:
                             \n{}", restore_file_res.unwrap_err());
                    error_count = error_count +1;
                 }
            }
        }
    }

    println!("Restored {} objects to all designated locations. Encountered {} Errors along the way.", res_data_vec.len(), error_count);

    return Ok(());
}


/// [`restore_folder`] function restores a folder from the .cache directory to the location indicated by the `restore_data` object.
/// 
/// Arguments:
/// 
/// * `restore_data`: type of [`RestoreData`], which tells us which folder to restore and where to restore it to
/// 
/// Returns:
/// 
/// [`restore_file`] returns an error message if restore process fails.
pub fn restore_folder(restore_data: RestoreData) -> Result<(), &'static str> {
    if restore_data.cachetype != enums::CacheType::Directory{
        return Err("Attempted to copy invalid restore type. Please make sure restore types match");
    }

    let restore_dir = restore_data.restore_to.as_str();

    if !Path::new(restore_dir).exists(){
        return Err("We could not find the restore folder to restore this object into. The folder did not seem to exist. 
        This maybe due to the operating system of this file being different and doesn't indicate a direct problem.");
    }

    let copy_from = format!(".cache/{}", restore_data.restore_obj_name);
    let copy_to = format!("{}/{}", restore_data.restore_to, restore_data.restore_obj_name);
    let copy_res_is_err = helpers::copy_recursively(copy_from, copy_to).is_err();

    if copy_res_is_err {
        return Err("Encounted an error while attempting to restore directory. 
        This maybe due to insufficient permissions or because the directory was not at the expected location to copy from.");
    }

    println!("Restored Folder named: {} to this location: {}", restore_data.restore_obj_name, restore_data.restore_to);
    return Ok(());
}

/// [`restore_file`] function restores a file from the .cache directory to the directory indicated by the `restore_data` object.
/// 
/// Arguments:
/// 
/// * `restore_data`: type of [`RestoreData`], which tells us which object to restore and where to restore it to
/// 
/// Returns:
/// 
/// [`restore_file`] returns an error message if restore process fails.
pub fn restore_file(restore_data: RestoreData) -> Result<(), &'static str> {
    if restore_data.cachetype != enums::CacheType::File{
        return Err("Attempted to copy invalid restore type. Please make sure restore types match");
    }

    let restore_dir = restore_data.restore_to.as_str();

    if !Path::new(restore_dir).exists(){
        return Err("We could not find the restore folder to restore this object into. The folder did not seem to exist. 
        This maybe due to the operating system of this file being different and doesn't indicate a direct problem.");
    }

    let copy_from = format!(".cache/{}", restore_data.restore_obj_name);
    let copy_to = format!("{}/{}", restore_data.restore_to, restore_data.restore_obj_name);
    let copy_is_err = fs::copy(copy_from,copy_to).is_err();

    if copy_is_err {
        return Err("Encounted an error while attempting to copy the folder. 
        This maybe due to insufficient permissions or because the file was not at the expected location to copy from.");
    }
    
    println!("Restored File named: {} to this location: {}", restore_data.restore_obj_name, restore_data.restore_to);
    return Ok(());
}