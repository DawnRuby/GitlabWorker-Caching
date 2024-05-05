use std::io::{prelude::*, self};
use std::io::{Seek, Write};
use std::iter::Iterator;
use std::ops::Deref;
use zip::result::ZipError;
use zip::write::FileOptions;
use std::fs::{File, self};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// [`zip_dir`] takes an iterator of directory entries, a prefix, a writer, and a compression
/// method, and creates a zip archive by adding files and directories from the iterator to the writer.
/// 
/// Arguments:
/// 
/// * `it`: type of [`&mut dyn Iterator<Item = DirEntry>`], an iterator representing the items to put into the zip. 
/// * `prefix`: type of [`String`], that represents the common prefix that should be
/// stripped from the file paths before adding them to the zip archive. This is useful when you want to
/// create a zip archive that contains files from a specific directory, but you don't want the directory
/// structure to be included in
/// * `writer`: type of [`T`] that implements the [`Write`] and [`Seek`] traits. 
/// It represents the output stream where the zip file will be written to. It could be any type
/// that implements these traits, such as a [`File`] or a [`TcpStream`].
/// * `method`: type of [`zip::CompressionMethod`] and is used to specify the
/// compression method to be used when creating the zip file. The [`zip::CompressionMethod`] enum provides
/// different compression methods such as `Stored`, `Deflated`, `Bzip2`, etc.
/// 
/// Returns:
/// If the zipping of the directory completed successfuly.
fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod
) -> zip::result::ZipResult<()> where T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}



/// [`zip_dir_recursively`] recursively zips a directory and its contents into a destination
/// file using the specified compression method.
/// 
/// Arguments:
/// 
/// * `src_dir`: type of [`&str`], that represents the source directory from which you
/// want to recursively zip all files and subdirectories.
/// * `dst_file`: type of [`&str`], representing the path and name of the destination file where the zipped
/// directory will be created.
/// * `method`: type of function [`zip::CompressionMethod`]. It is used to specify the compression method to be used when creating the
/// zip file. The [`zip::CompressionMethod`] enum provides different compression methods such as `Stored`, `Deflated, `Bzip2`, etc.
/// 
/// Returns:
/// If the zipping of the directory completed successfuly.
pub fn zip_dir_recursively(src_dir: &str,dst_file: &str,method: zip::CompressionMethod,) 
    -> zip::result::ZipResult<()> {

    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}

/// [`unzip_file`] takes a file name as input, attempts to unzip the file, and returns a
/// result indicating success or failure.
/// 
/// Arguments:
/// 
/// * `fname`: type of [`String`] that represents the file name or path of the zip file
/// that you want to unzip.
/// 
/// Returns:
/// Returns an error message, containing the error that was hit during the unzip process
pub fn unzip_file(fname: String) -> Result<(), &'static str> {

    let file_name_cannonicalize_result = fs::canonicalize(fname.clone());

    if file_name_cannonicalize_result.is_err(){
        return Result::Err("Could not canonicalize file please make sure the path is correct. 
        For more information about canonicalization visit: https://doc.rust-lang.org/std/fs/fn.canonicalize.html");
    }

    let file_pathbuf = file_name_cannonicalize_result.ok().unwrap();

    if !fs::metadata(file_pathbuf.clone()).is_ok(){
        return Result::Err("Could not find directory at zip location");
    }

    let parent_path = file_pathbuf.deref().parent();

    if parent_path.is_none(){
        return Result::Err("Could not find parentpath, parentpath returned nothing");
    }

    let parent_path = parent_path.unwrap();

    let file = fs::File::open(fname).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => parent_path.join(path.to_owned()),
            None => continue,
        };
        
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    return Ok(());
}