use std::{env, process};

use crate::datas::enums;
mod datas;
mod create_cache;
mod download_cache;
mod restore_cache;
mod envfuncs;
mod zip;
mod helpers;

fn main() { 
    let launch_args: Vec<String> = env::args().collect();
    let os_type = enums::OsType::get_ostype();

    if os_type ==  enums::OsType::Unknown{
        eprint!("Detected Unknown / Unsupported operating system. Aborting...");
        process::exit(1);
    }

    if launch_args.is_empty(){
        eprintln!("Please enter an arg to specify the operation you'd like to do. For more information use /help");
        process::exit(2);
    }

    if launch_args.contains(&format!("/help")) {
        println!("Welcome to the gitlab caching tool. This will allow you to cache directories and restore them.");
        println!("Caching is usually done through specifying enviorement variables. I may add more options in the future if I feel the need.");
        println!("");
        println!("++++++++++++++");
        println!("Required env-vars:");
        println!("We read a few values which are required for the restore process");
        println!("These values include: ");
        println!("WEBDAVUSER: The user to use for Webdav authentication. I recommend storing this in a secured variable on the gitlab server");
        println!("WEBDAVPASS: The password to use for Webdav authentication. I currently do not support unauthenticated webdav storage neither do we recommend it");
        println!("WEBDAVADDR: The web address base to use to store Data (e.g.: https://example.com");
        println!("CI_PROJECT_ID: Usually a default value set by gitlab itself. See more here: https://docs.gitlab.com/ee/ci/variables/predefined_variables.html");
        println!("CI_COMMIT_BRANCH: Usually a default value set by gitlab itself. See more here: https://docs.gitlab.com/ee/ci/variables/predefined_variables.html");
        println!("++++++++++++++");
        println!("++++++++++++++");
        println!("Specifing Cache:");
        println!("For caching folders via enviorement variables specify them this way: cachepath_<VARNAME> where <VARNAME> is the unique name for the variable you wanted to specify.");
        println!("For caching files via enviorement variables specify them this way: cachefile_<VARNAME> where <VARNAME> is the unique name for the variable you wanted to specify");
        println!("The value of the enviorement variables specifies the path where we copy it.");
        println!("An example of this is:\n Variable name is: cachepath_homedir.\n Variable value is: /home/myawsomeuser");
        println!("++++++++++++++");
        println!("++++++++++++++");
        println!("Args:");
        println!("Use these arguments to specify the operation you intend to do. These arguments follow the executable e.g: <execname>.exe /backup");
        println!("/backup          || Backs up all values of enviorement variables with the right name. May not be called at the same time as /download or /restore (process will exit after finishing this)");
        println!("/help            || Shows this help menu. Overrides all other instructions. (process will exit after finishing this)");
        println!("/download        || Download the zip and extracts it into the .cache folder. Does not restore the files. (May execute other things after running this. Apart from /backup)");
        println!("/restore         || Restores all files from the .cache folder to the correct locations and then deletes the .cache folder (Process will exit after finishing this)");
        println!("/rmlocalcache    || Deletes the .cache folder (Process will exit after finishing this)");
        println!("/rmremcache      || Deletes the remote cache folder on the webdav directory (Process will exit after finishing this)");
        println!("++++++++++++++");
    }


    if launch_args.contains(&format!("/backup")) && (launch_args.contains(&format!("download")) ||launch_args.contains(&format!("restore"))){
        eprintln!("Cannot backup and restore or backup and download at the same time. Please operate the two in a seperate call. 
        \nUsually you also may not wanna do the two at the same time");
        process::exit(2);
    }

    if launch_args.contains(&format!("/download")){
        let upload_res = download_cache::main(os_type);

        if upload_res.is_err(){
            //Ensure cache is deleted
            let _ = helpers::del_restore_dir();
            process::exit(upload_res.unwrap_err());
        }
    }

    if launch_args.contains(&format!("/rmlocalcache")){
        let del_restore_dir_has_err = helpers::del_restore_dir().is_err();

        if del_restore_dir_has_err{
            eprintln!("Encountered an error while attempting to remove .cache folder");
            process::exit(5);
        }

        process::exit(0);
    }

    if launch_args.contains(&format!("/rmremcache")){
        let del_webdav_cache_has_err = helpers::del_webdav_cache(os_type).is_err();

        if del_webdav_cache_has_err{
            eprintln!("Encountered an error while attempting to remove remote cache folder");
            process::exit(4);
        }

        process::exit(0);
    }

    if launch_args.contains(&format!("/backup")){
        let upload_res = create_cache::main(os_type);

        if upload_res.is_err(){
            eprintln!("Encountered an error while attempting to upload the files to the folder");
            //Ensure cache is deleted so we can upload it later.
            let _ = helpers::del_restore_dir();
            process::exit(upload_res.unwrap_err());
        }

        process::exit(0);
    }

    if launch_args.contains(&format!("/restore")){
        let upload_res = restore_cache::main();

        if upload_res.is_err(){
            eprintln!("Encountered an error while attempting to restore the files to the folder");
            //Ensure cache is deleted so we can upload it later.
            let _ = helpers::del_restore_dir();
            process::exit(upload_res.unwrap_err());
        }

        process::exit(0);
    }

    eprintln!("Could not find valid aguments. 
    \nIf you don't know which arguments are available / how to use the program, execute the argument /help");

    eprintln!("Reached in arguments were:");
    for launch_arg in launch_args{
        eprintln!("{}", launch_arg);
    }
}