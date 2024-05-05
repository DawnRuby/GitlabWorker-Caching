use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::{env, fs, str::FromStr, ops::Deref};
use crate::datas::structs::{self, RestoreData};
use crate::datas::enums;
use crate::{envfuncs, helpers};
use crate::zip::zip_dir_recursively;
use normpath::PathExt;
use rustydav::client::{self, Client};

/// Fetches cached values via enviorement variables, moves the files from there to the .cache folder, zips it's contents and then uploads the zipped contents to a
/// webdav server.
/// 
/// Arguments:
/// 
/// * `ostype`: Is of type [`enums::OsType`]. Represents the type of operating system.
/// 
/// Returns:
/// Returns an error code if it fails, indicating the location of an error.
pub fn main(ostype: enums::OsType) -> Result<(),i32>{
    println!("Welcome to the cache upload tool. This will create a cache structure for the specified values.
    To define more items to cache simply create more enviorement variables with cachepath_ or cachefile_ in front of them. 
    We take the local file system into account so full paths are not required.");
    println!("Current directory is: {}", env::current_dir().unwrap().to_str().unwrap());
    let folders = envfuncs::get_env_if_startswith("cachepath_");
    let files = envfuncs::get_env_if_startswith("cachefile_");

    if (folders.len() == 0)  && (files.len() == 0){
        eprintln!("Found 0 directories or files via searching env vars that start with cachepath_ or cachefile_. \n
        Please make sure you cache something when calling this script to reduce length of execution durinb build scripts.")
    }

    if files.is_empty() && folders.is_empty(){
        eprintln!("Could not find any files or folders to restore. Please ensure that at least one enviorement variable starting with the name: cachepath_ or cachefile_ 
        and the value of a path is set.");
        return Err(13);
    }

    let restore_data = generate_storage_data_from_pathstrings(files, folders);

    if restore_data.is_err(){
        eprintln!("Encountered an error while generating the Restore Data. The error was: {}", restore_data.unwrap_err());
        return Err(14);
    }

    let restore_data = restore_data.unwrap();
    println!("Found {} Files and folders overall and created their Data objects. Uploading to server now.", restore_data.len());
    let json_data = serde_json::to_string_pretty(&restore_data).unwrap(); 
    println!("Checking if base & project directory exists on webdav server and creating it if it doesn't now.");
    let create_result = create_webdav_paths();

    if create_result.is_err(){
        eprintln!("Encountered error while attempting to create base and project directory on webdav server. Error was: \n{}", create_result.unwrap_err());
        return Err(15);
    }

    let cpy_files_result = cpy_files_to_cache_dir(json_data, restore_data);

    if cpy_files_result.is_err(){
        eprintln!("Encountered an error while attempting to copy files to the .cache directory. Error was: \n{}", cpy_files_result.unwrap_err());
        return Err(16);
    }


    let zip_cache_result = zip_cache_dir(ostype);

    if zip_cache_result.is_err(){
        eprintln!("Encountered error while attempting to Zip cache folder. Error was: \n{}", zip_cache_result.unwrap_err());
        return Err(17);
    }

    let copyres = upload_zip();

    if copyres.is_err(){
        eprintln!("Encountered an error while attempting to upload the ZIP file to the webdav server. Error was: \n{}", copyres.unwrap_err());
        return Err(18);
    }

    println!("Finished uploading cache to webdav.");
    return Ok(());
}

/// [`generate_storage_data_from_pathstrings`] generates a [`Vec<enums::RestoreData>`] from a list of file and folder paths.
/// 
/// Arguments:
/// 
/// * `files`: A vector of strings representing file paths.
/// * `folders`: A vector of strings representing the paths to folders.
/// 
/// Returns:
/// 
/// Returns a [`Result`] enum. If successful in generating the restore data, 
/// returns [`Vec<enums::RestoreData>`] objects. If there is an error, returns a static string
/// message describing the error.
fn generate_storage_data_from_pathstrings(files: Vec<String>, folders: Vec<String>) -> Result<Vec<RestoreData>, &'static str> {
    let mut restore_data:Vec<structs::RestoreData> = Vec::new();

    if folders.is_empty() && files.is_empty(){
        return Err("Both files and folders were empty. We need data to generate the restoredata objects");
    }

    for folder in folders{
        let restoredir = generate_storage_data_directory_from_pathstr(folder.as_str());

        if let Ok(restoredir) = restoredir {
            restore_data.push(restoredir);
        }else if let Err(restoredir) = restoredir {
            println!("Could not add the folder at:\n{}\n Encountered error: {}", folder,restoredir);
        }
    }

    for file in files{
        let restorefile = generate_storage_data_file_from_pathstr(file.clone());

        if let Ok(restorefile) = restorefile {
            restore_data.push(restorefile);
        }else if let Err(restorefile) = restorefile {
            println!("Could not add the folder at:\n{}\n Encountered error: {}", file,restorefile);
        }
    }

    let numresdata = restore_data.len();

    if numresdata == 0{
        return Err("No Restore data found");
    }

    return Ok(restore_data)
}

/// `cpy_files_to_cache_dir` copies files and directories to a cache directory
/// 
/// Arguments:
/// 
/// * `json_data`: type of <string>. 
/// JSON Data that should be written to restore the cache later on should represent `restore_data`
/// * `restore_data`: type of`Vec<enums::RestoreData>`. 
/// Contains the data we want to restore later on. `json_data` should represent it's contents.
/// 
/// Returns:
/// 
/// The function `cpy_files_to_cache_dir` returns a `Result<(), &'static str>`.
fn cpy_files_to_cache_dir(json_data: String, restore_data: Vec<structs::RestoreData>) -> Result<(), &'static str>{
    if Path::new(".cache").exists(){
        return Err("We already found a folder named .cache. 
        Aborting upload since this may cause issues / conflics. 
        Please make sure you don't have a project in your repository with that name.");
    }

    let failed_create_dir = fs::create_dir_all(".cache").is_err();
    
    if failed_create_dir {
        return Err("Could not create cache dir at .cache. 
        Please ensure we have write permissions in the current directory you work in and that the folder doesn't already exist.");
    }

    let failed_write_json = fs::write(".cache/data.json", json_data).is_err();

    if failed_write_json {
        return Err("Could not write json data to .cache/data.json.");
    }

    let current_directory = env::current_dir().unwrap();

    for restore_obj in restore_data{

        let restore_objpath = format!("{}/{}", restore_obj.restore_to, restore_obj.restore_obj_name);
        let copy_to_path = format!("{}/.cache/{}", current_directory.to_str().unwrap(), restore_obj.restore_obj_name);
        let copy_error: bool;

        match restore_obj.cachetype{
            enums::CacheType::Directory => {copy_error = helpers::copy_recursively(restore_objpath.clone(), copy_to_path.clone()).is_err();}
            enums::CacheType::File => {copy_error = fs::copy(restore_objpath.clone(), copy_to_path.clone()).is_err();}
        }
    
        if copy_error{
            println!("Failed to copy file from {} to {}. This may result in the file not being restored in a later process.", restore_objpath, copy_to_path);
        }
    }

    return Ok(());
}

/// Generates the storage data file from a given file path string.
/// 
/// Arguments:
/// 
/// * `file`:  type of `String` that represents the path to a file.
/// 
/// Returns:
/// 
/// Returns a [`Result`] type. If the function is successful, it returns a [`structs::RestoreData`] object. If there
/// is an error, it returns a [`String`] representing the error error message.
fn generate_storage_data_file_from_pathstr(file: String) -> Result<structs::RestoreData, &'static str> {
    let file_path = Path::new(&file).normalize();

    if file_path.is_err(){
        println!("Could not canonicalize directory please make sure the path is correct. We normally do not recommend caching directories outside of project space.
        \nIf this is intentional you can ignore this error.
        \nFor more information about canonicalization visit: https://doc.rust-lang.org/std/fs/fn.canonicalize.html");

        let file_path = [file].iter().collect();
        return generate_storage_data_file_from_pathbuf(file_path);
    }

    return generate_storage_data_file_from_pathbuf(file_path.unwrap().into_path_buf());
}

/// Generates the storage data file from a given pathbuf if it is a file
/// 
/// Arguments:
/// 
/// * `file`:  type of `String` that represents the path to a file.
/// 
/// Returns:
/// 
/// Returns a [`Result`] type. If the function is successful, it returns a [`structs::RestoreData`] object. If there
/// is an error, it returns a [`String`] representing the error error message.
fn generate_storage_data_file_from_pathbuf(file: PathBuf) ->  Result<structs::RestoreData, &'static str>{
    if !fs::metadata(file.clone()).is_ok(){
        return Result::Err("Could not find directory at location");
    }

    if file.is_dir(){
        return Result::Err("We recieved a folder when we expected a file... please make sure you use the right prefix");
    }

    let parent_path = file.deref().parent();
    
    if parent_path.is_none(){
        return Err("Could not access parent directory. Please validate you're trying to cache a valid folder (and have permissiosn to access the directory to put it back into)");
    }

    let parent_path = parent_path.unwrap().to_str().unwrap();
    let restore_object_name = file.file_name().unwrap().to_os_string().into_string().unwrap();
    let parent_path_string = String::from_str(parent_path).unwrap();
    let res_data: structs::RestoreData = structs::RestoreData { 
        restore_obj_name: restore_object_name, 
        cachetype: enums::CacheType::File,
        restore_to: parent_path_string
    };

    println!("Created storage data for file: {} successfully", res_data.restore_obj_name);
    return Result::Ok(res_data);
}

/// [`generate_storage_data_directory_from_pathstr`] takes a pathstring as an input,
/// validates it, and returns a [`RestoreData`] struct containing information about the directory for
/// caching purposes.
/// 
/// Arguments:
/// 
/// * `directory`: type of "&str" that represents the path to a directory.
/// 
/// Returns:
/// 
/// Returns a [`Result`] type. 
/// If the function is successful, it returns a [`structs::RestoreData`] object. If there is an
/// error, it returns  a [`String`] error message.
fn generate_storage_data_directory_from_pathstr(directory: &str) -> Result<structs::RestoreData, &'static str> {
    let dir_canonicalize_result = Path::new(directory).normalize();

    if dir_canonicalize_result.is_err(){
        println!("Could not canonicalize directory please make sure the path is correct. We normally do not recommend caching directories outside of project space.
        \nIf this is intentional you can ignore this error.
        \nFor more information about canonicalization visit: https://doc.rust-lang.org/std/fs/fn.canonicalize.html");
        let dir_path = [directory].iter().collect();
        return generate_storage_data_directory_from_pathbuf(dir_path);
    }

    return generate_storage_data_directory_from_pathbuf(dir_canonicalize_result.unwrap().into_path_buf());
}

/// Generates the storage data file from a given pathbuf if it is a file
/// 
/// Arguments:
/// 
/// * `file`:  type of `String` that represents the path to a file.
/// 
/// Returns:
/// 
/// Returns a [`Result`] type. If the function is successful, it returns a [`structs::RestoreData`] object. If there
/// is an error, it returns a [`String`] representing the error error message.
fn generate_storage_data_directory_from_pathbuf(dir: PathBuf) ->  Result<structs::RestoreData, &'static str>{
    if !fs::metadata(dir.clone()).is_ok(){
        return Result::Err("Could not find directory");
    }

    if dir.is_file(){
        return Result::Err("We recieved a file when we expected a folder... please make sure you use the right prefix");
    }

    let parentpath = dir.deref().parent();

    if parentpath.is_none(){
        return Err("Could not access parent directory. Please validate you're trying to cache a valid folder (and have permissiosn to access the directory to put it back into)");
    }

    let parent_path = parentpath.unwrap().to_str().unwrap();
    let restore_object_name = dir.file_name().unwrap().to_os_string().into_string().unwrap();
    let parentpathstr = &String::from_str(parent_path).unwrap();
    let res_data: structs::RestoreData = structs::RestoreData { 
        restore_obj_name: restore_object_name, 
        cachetype: enums::CacheType::Directory,  
        restore_to: parentpathstr.to_string() 
    };

    println!("Created storage data for folder: {} successfully", res_data.restore_obj_name);
    return Result::Ok(res_data);
}

/// [`upload_zip`] uploads a zip file to a webdav server using the provided credentials and
/// file path.
/// 
/// Returns:
/// 
/// The function [`upload_zip()`] returns an error message if there is a problem.
fn upload_zip() -> Result<(), &'static str>{
    let webdav_address = envfuncs::get_webdavaddr();
    let webdav_client = client::Client::init(&envfuncs::get_webdav_user(), &envfuncs::get_webdav_password());
    let file = File::open(envfuncs::get_zip_file_name());

    if file.is_err(){
        return Err("Could not open file to read bytes into stream This maybe due to the zip file having been deleted since creation");
    }

    let file = file.unwrap();
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    // Read file into vector.
    let readerr = reader.read_to_end(&mut buffer).is_err();

    if readerr {
        return Err("Could not read zip file. This maybe due to insufficient permissions or because of other similar reasons.");
    }

    let upload_path = format!("{}/gitcache/{}/{}", webdav_address, envfuncs::get_projectid(), envfuncs::get_zip_file_name());
    let upload_result = webdav_client.put(
        buffer, 
        upload_path.as_str()
    );

    if upload_result.is_err(){
        return Err("Encountered an error while attempting to upload the zip file. Program will now exit...");
    }

    println!("All files were uploaded. Program will now exit...");
    return Ok(());
}


/// [`create_webdav_paths`] checks if the required directories exist in a WebDAV server and
/// creates them if necessary.
/// 
/// Returns:
/// 
/// The function [`create_webdav_paths()`] returns a [`Result<(), &'static str>`].
fn create_webdav_paths() -> Result<(), &'static str>{
    let webdav_addr = envfuncs::get_webdavaddr();
    let webdav_client = client::Client::init(&envfuncs::get_webdav_user(), &envfuncs::get_webdav_password());
    let base_folder_structure = webdav_client.list(format!("{}/gitcache", webdav_addr).as_str(), "1");

    if base_folder_structure.as_ref().is_err() {
        return Err("Encountered a problem trying to retrieve webdav folder structure.");
    }

    let folder_structure = base_folder_structure.unwrap();

    if folder_structure.status() != http::StatusCode::FOUND && folder_structure.status() != http::StatusCode::OK {
        println!("Could not find root directory of folder structure. Creating it now.");
        let make_base_dir_res = webdav_client.mkcol(format!("{}/gitcache", webdav_addr).as_str());

        if make_base_dir_res.is_err() {
            return Err("Could not make base directory. Please ensure your access credentials are correct.");
        }

        let make_base_dir_res = make_base_dir_res.unwrap();
        let make_base_dir_statuscode = make_base_dir_res.status();

        match make_base_dir_statuscode{
            http::StatusCode::FORBIDDEN => { return Err("Server returned status FORBIDDEN while creating gitcache folder. Please make sure you have the ability to create and upload files on the webdav server.");}
            http::StatusCode::ACCEPTED | http::StatusCode::OK  | http::StatusCode::CREATED => { println!("Gitcache folder was created on the server."); }
            http::StatusCode::CONFLICT => { return Err("Gitcache folder seems to already exist (recieved CONFLICT). But we couldn't read it before. This could be due to insufficient read permissions inside the base folder."); }
            _ => { return Err("Response contained unknown / unhandled status code.")}
        }

        let result = make_webdav_project_path(webdav_client);
        if result.is_err(){
            return result;
        }

        println!("All required Webdav directories were created or already existed");
        return Ok(());
    }

    let body_text = folder_structure.text();

    if body_text.is_err(){
        return Err("Could not read message body when attempting to fetch folder structure.");
    }

    let body_text = body_text.unwrap();

    if !body_text.contains(format!("<a href=\"{}/\">", envfuncs::get_projectid()).as_str()){
        println!("Project directory does not exist yet. Creating it now.");
        return make_webdav_project_path(webdav_client);
    }

    println!("Cache storage location was checked and alredy existed.");
    return Ok(())
}

/// [`make_webdav_project_path`] creates a project directory on a webdav server and handles
/// different status codes returned by the server.
/// 
/// Arguments:
/// 
/// * `webdav_client`: type of [`Client`], which is used to make requests to a WebDav Server. 
/// 
/// Returns:
/// Returns a [`&'static str`] error message if there was an
/// error during the creation process.
fn make_webdav_project_path(webdav_client: Client) -> Result<(), &'static str>{

    let make_project_dir_result = webdav_client.mkcol(format!("{}/gitcache/{}", envfuncs::get_webdavaddr(), envfuncs::get_projectid()).as_str());

    if make_project_dir_result.is_err() {
        return Err("Could not make base directory. Please ensure your access credentials are correct.");
    }

    let make_project_dir_result = make_project_dir_result.unwrap();
    let make_project_dir_status_code = make_project_dir_result.status();

    match make_project_dir_status_code{
        http::StatusCode::FORBIDDEN => { 
            return Err("Server returned status FORBIDDEN while trying to create project sub-folder. 
            Please make sure you have the ability to create and upload files on the webdav server.");
        }
        http::StatusCode::ACCEPTED | http::StatusCode::OK | http::StatusCode::CREATED => { 
            println!("Project sub-folder was created on the server."); 
        }
        http::StatusCode::CONFLICT => { 
            return Err("Project sub-folder seems to already exist (recieved CONFLICT). But we couldn't read it before. 
            This could be due to insufficient read permissions inside the gitcache folder."); 
        }
        _ => { 
            return Err("Response contained unknown / unhandled status code.")
        }
    }

    println!("Project directory was created.");
    return Ok(());
}

/// [`zip_cache_dir`] zips the contents of the ".cache/" directory recursively and zips it up, 
/// with the name based on the operating system type and branch name.
/// 
/// Arguments:
/// 
/// * `os_type`: type  of [`enums::OsType`]. It represents the operating system type.
/// 
/// Returns:
/// An error message if zipping the directory failed.
fn zip_cache_dir(os_type: enums::OsType) -> Result<(), &'static str>{
    let branch_name = envfuncs::get_branch_name();
    let dest_file = envfuncs::get_zip_file_name();

    let zip_result = zip_dir_recursively(".cache/", dest_file.as_str(), zip::CompressionMethod::Stored);

    if zip_result.is_err(){
        return Err("Failed to zip up file. This may be because of insufficient permissions or a folder being moved during the zipping operation.");
    }

    println!("Zipped all files and put them into {}-{}.zip", os_type.to_string(), branch_name);
    return Ok(());
}