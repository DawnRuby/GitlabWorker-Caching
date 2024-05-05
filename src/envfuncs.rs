use std::{env, process};

use crate::datas::enums;

/// The function [`get_env_if_startswith`] returns a vector of environment variable values if their names
/// start with a given string.
/// 
/// Arguments:
/// 
/// * `name`: type of `&str` that represents the prefix that you want to check for in the enviorement variable names.
/// 
/// Returns:
/// 
/// The function [`get_env_if_startswith`] returns a vector of strings [`Vec<String>`] containing the
/// values of environment variables that start with the specified `name`.
pub fn get_env_if_startswith(name: &str) -> Vec<String>{

    let mut found_variables:Vec<String> = Vec::new();
    for (n,v) in env::vars_os() {
        let env_var = n.to_str().unwrap_or("failed");
 
        //Check if conversion succeeded if it did not, output warning and continue.
        if env_var == "failed"{
            println!("Failed to convert an enviorement variable to a string. 
            Please validate it's name is a valid env var name (UTF-8). 
            Skipping this one.");
            continue;
        } if env_var.starts_with(name) {
             let vstr = v.to_str().expect(format!("Could not convert value of the enviorement variable {env_var} to string.").as_str());
             found_variables.push(vstr.to_string());
        }
    }
 
    return found_variables;
 }

/// [`safe_get_envvar`] retrieves the value of an environment variable and handles errors by
/// printing an error message and optionally exiting the program.
/// 
/// Arguments:
/// 
/// * `envvarval`: type of [`&str`] that represents the name of the environment variable you want to retrieve the value for.
/// * `errmsg`: type of [`&str`] that contains the error message to be printed if the
/// environment variable retrieval fails.
/// * `exitonfail`: type of [`bool`] indicating whether the program should exit if the environment
/// variable is not found or cannot be retrieved. If `exitonfail` is `true`, the program will exit with
/// a status code of 2. If `exitonfail` is `false`, the program will continue execution and return an empty string
/// 
/// Returns:
/// 
/// returns a [`String`] value representing the value of the enviorement variable
fn safe_get_envvar(envvarval: &str, errmsg: &str, exitonfail: bool) -> String{
    let get_envvar_result = env::var(envvarval);

    if get_envvar_result.is_err(){
        if exitonfail{
            eprintln!("{}", errmsg);
            process::exit(2);
        }
        
        println!("{}", errmsg);
        return String::new();
    }

    return get_envvar_result.unwrap();
 }


/// [`get_webdav_user`] returns the value of the `WEBDAVUSER` environment variable, or
/// displays an error message if it is not set.
/// 
/// Returns:
/// 
/// A string value is being returned.
pub fn get_webdav_user() -> String{
    return safe_get_envvar("WEBDAVUSER",
     "WEBDAVUSER env var not set. 
     Please make sure to set one. We currently do not support unsecured webdav servers.", 
     true);
}

/// [`get_webdav_password`] returns the value of the `WEBDAVPASS` environment variable, or
/// displays an error message if it is not set.
/// 
/// Returns:
/// 
/// A string value is being returned.
pub fn get_webdav_password() -> String{
    return safe_get_envvar("WEBDAVPASS", 
    "WEBDAVPASS env var not set. 
    Please make sure to set one. 
    We currently do not support unsecured webdav servers.", true);
}

/// The function [`get_webdavaddr`] returns the value of the `WEBDAVADDR` environment variable, or
/// displays an error message if it is not set.
/// 
/// Returns:
/// 
/// a string value.
pub fn get_webdavaddr() -> String {
    return safe_get_envvar("WEBDAVADDR", 
    "WEBDAVPASS env var not set. 
    Please make sure to set one otherwise we don't know where to connect.", 
    true);
}

/// The function [`get_projectid`] returns the value of the `CI_PROJECT_ID` environment variable, or
/// prompts the user to set it manually if it is not found.
/// 
/// Returns:
/// 
/// a String value.
pub fn get_projectid() -> String{
    return safe_get_envvar("CI_PROJECT_ID", 
    "Could not find CI_PROJECT_ID enviorement variable. 
    Please make sure that you are using a gilab server or set the value manually to a unique value.", 
    true);
}

/// The function [`get_branch_name`] returns the value of the environment variable `CI_COMMIT_BRANCH` or
/// `CI_MERGE_REQUEST_TARGET_BRANCH_NAME`, or a default error message if the variables are not found.
/// 
/// Returns:
/// 
/// a String value.
pub fn get_branch_name() -> String{
    return safe_get_envvar("CI_COMMIT_BRANCH", 
    "Could not find CI_COMMIT_BRANCH or CI_MERGE_REQUEST_TARGET_BRANCH_NAME enviorement variable. 
    Please make sure that you are using a gilab server or set the value manually to a unique value.", 
    true);
}

/// The function [`get_zip_file_name`] returns a string representing the name of a zip file based on the
/// operating system type and branch name.
/// 
/// Returns:
/// 
/// A string containing the zip file name.
pub fn get_zip_file_name() -> String {
    let ostype: enums::OsType;
    if cfg!(windows) {
        ostype = enums::OsType::Windows;
    }else if cfg!(unix){
        ostype = enums::OsType::Unix;

    }else{
        eprintln!("OsType is currently not supported. Exiting with error");
        process::exit(1);
    }

    return format!("{}-{}.zip", ostype.to_string(), get_branch_name());
}