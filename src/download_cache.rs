use std::{env, fs::{self}, io::Write, path::Path};
use rustydav::client;
use crate::{envfuncs, datas::enums, zip};


/// Downloads the correct zip file depending on [`os_type`] reached in and unzips it's contents to the .cache folder.
/// 
/// Arguments:
/// 
/// * `os_type`: type of [`enums::OsType`]. Used to determine the current operating system that's being used
/// 
/// Returns:
/// Returns an error code if there was a problem, indicating location of error.
pub fn main(os_type: enums::OsType) -> Result<(), i32>{
    println!("Welcome to the caching download tool. 
    This tool will download your cache to the .cache folder.");
    println!("Current directory is: {}", env::current_dir().unwrap().to_str().unwrap());

    let branch_name = envfuncs::get_branch_name();
    let download_file_result = download_files_from_webdav(os_type, branch_name);

    if download_file_result.as_deref().is_err(){
        eprintln!("Encountered an error / warning while downloading / creating file. \nError was:{}", download_file_result.clone().unwrap_err());

        //Return error code 0 here if the message contains "Could not find the file on the server." since we want to make sure that we don't exit the program just because of that error
        if download_file_result.unwrap_err().contains("Could not find the file on the server.") {
            return Err(0);
        }


        return Err(22);
    }
    
    let file_name = download_file_result.unwrap();
    let unzip_result = unzip_and_del(file_name);

    if unzip_result.is_err(){
        eprintln!("Encountered an error / warning while trying to unzip file. \nError was:{}", unzip_result.unwrap_err());
        return Err(23);
    }

    return Ok(());
}

/// [`download_files_from_webdav`] downloads a zip file from a webdav server. Zip file is determined based on [`os_type`]
/// 
/// Arguments:
/// 
/// * `os_type`: type of [`enums::OsType`], used to determine the current OS you're using to know which zip file to download
/// * `branch_name`: type of [`String`] that represents the name of the branch in a Git repository. It is used to construct the path for the cache to be stored.
/// 
/// Returns:
/// 
/// [`download_files_from_webdav`] if successful returns a [`String`] representing the file name of the downloaded file, 
/// and if there was an error returns [`&'static str`] representing the error message.
fn download_files_from_webdav(os_type: enums::OsType, branch_name: String) -> Result<String, &'static str>{
    if Path::new(".cache").exists(){
        return Err("We already found a folder named .cache. Aborting download since this may cause issues / conflics. Please make sure you don't have a project in your repository with that name.");
    }

    let failed_create_dir = fs::create_dir_all(".cache").is_err();

    if failed_create_dir {
        return Err("Could not create cache dir at .cache. Please ensure we have write permissions in the current directory you work in and that the folder doesn't already exist. This is an unrecoverable error aborting program.");
    }

    let webdav_address = envfuncs::get_webdavaddr();
    let webdav_client = client::Client::init(&envfuncs::get_webdav_user(), &envfuncs::get_webdav_password());
    let file_name = format!{"{}-{}.zip", os_type.to_string(), envfuncs::get_branch_name()};
    let download_result = webdav_client.get(format!("{}/gitcache/{}/{}", webdav_address, envfuncs::get_projectid(), file_name).as_str());


    if download_result.as_ref().is_err() {
        return Err("Encountered an error downloading cache file from server.");
    }

    let download_result = download_result.unwrap();
    let download_result_status_code = download_result.status();

    match download_result_status_code{
        http::StatusCode::FORBIDDEN => { 
            return Err("Server returned status FORBIDDEN while trying to create project sub-folder. 
                        Please make sure you have the ability to create and upload files on the webdav server.");
        }
        http::StatusCode::NOT_FOUND => { 
            return Err("Could not find the file on the server. Exiting here with non 0 exit code since this probably means you just haven't uploaded the cache yet. 
            If this happens after the cache files were created please contact a system administrator");
        }
        http::StatusCode::ACCEPTED | http::StatusCode::OK | http::StatusCode::CREATED => { 
            println!("Project sub-folder was created on the server."); 
        }
        http::StatusCode::CONFLICT => { 
            return Err("Project sub-folder seems to already exist (recieved CONFLICT). 
                        But we couldn't read it before. This could be due to insufficient read permissions inside the gitcache folder.")
        } 
        _ => { 
            return Err("Response contained unknown / unhandled status code.")
        }
    }

    let file_name = format!(".cache/{}-{}.zip", os_type.to_string(), branch_name);
    let file = fs::OpenOptions::new()
    .create(true)
    .write(true)
    .open(file_name.clone());


    if file.is_err(){
        return Err("Encountered an error while attempting to create file. This maybe due to insufficient permissions");
    }

    let mut file = file.unwrap();
    let file_buffer = download_result.bytes();

    if file_buffer.is_err(){
        return Err("Could not unwrap bytes of file. This may be because the file is empty or did not download properly.")
    }

    let file_buffer = file_buffer.unwrap();
    let file_write_result = file.write_all(file_buffer.as_ref());

    if file_write_result.is_err(){
        return Err("Encountered an error while writing the downloaded bytes to the file. This may indicate insufficient permissions to edit files.");
    }

    return Ok(file_name);
}


/// `unzip_and_del` unzips a file, moves it to a cache folder, and then deletes the original zip file.
/// 
/// Arguments:
/// 
/// * `file_name`: type of [`String`], representing the name of the file to be unzipped and deleted.
/// 
/// Returns:
/// 
/// The function [`unzip_and_del`] returns an error message if there was a problem.
fn unzip_and_del(file_name: String) -> Result<(), &'static str>{
    let unzip_has_error = zip::unzip_file(file_name.clone()).is_err();

    if unzip_has_error {
        return Err("Encountered an error while attempting to unzip the directory.");
    }

    println!("Unzipped file to .cache folder.");

    let remove_cache_has_error = fs::remove_file(file_name).is_err();

    if remove_cache_has_error {
        println!("Encountered an error while attempting to delete the cache zip file. 
        Ignoring but this should not happen and maybe due to a permission error");
    }

    println!("Deleted cache zip file.");
    return Ok(());
}