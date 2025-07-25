use std::fs;
use std::io::prelude::*;
use zip::write::SimpleFileOptions;

pub struct ZipTool {

}

impl super::ProcelioCLITool for ZipTool {
    fn command(&self) -> &'static str {
        "zip"
    }

    fn usage(&self) {
        println!("path/to/folder");
        println!("    Creates a zip of all the files in given folder");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
}

fn zip_recursive<T: Write + std::io::Seek> (
    path: &std::path::Path,
    root: &std::path::Path,
    zipper: &mut zip::ZipWriter<T>) {
    let options = SimpleFileOptions::default()
        .unix_permissions(0o755);
    let name = path.strip_prefix(root).unwrap().to_str().unwrap().to_owned();

    let meta = std::fs::metadata(path).unwrap().is_dir();
    if meta {
        let dir = std::fs::read_dir(path).unwrap();
        zipper.add_directory(name, options).unwrap();
        for elem in dir {
            zip_recursive(&elem.unwrap().path(), root, zipper);
        }
    } else if &name != "manifest.json" {
        zipper.start_file(name, options).unwrap();
        let mut f = std::fs::File::open(path).unwrap();
        let mut bytes = [0u8; 512];
        loop {
            let size = f.read(&mut bytes);
            match size {
                Err(e) => {panic!("{}", e); },
                Ok(0) => {break;}
                Ok(e) => {zipper.write_all(&bytes[0..e]).unwrap();}
            };
        }    
    }
}


fn zip_dir<T: Write + Seek>(
    path: &std::path::Path,
    writer: T,
) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipWriter::new(writer);
    let manifest = path.join("manifest.json");
    if std::fs::metadata(&manifest).is_ok() {
        zip.start_file("manifest.json".to_owned(), SimpleFileOptions::default().unix_permissions(0o755)).unwrap();
        let mut bytes = Vec::new();
        std::fs::File::open(manifest).unwrap().read_to_end(&mut bytes).unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip_recursive(path, path, &mut zip);
    zip.finish()?;
    Result::Ok(())
}

fn tool_impl(args: Vec<String>) {
    let mut args = args.into_iter();
    let folder = args.next().unwrap_or("--help".to_owned());

    if !fs::metadata(&folder).unwrap().is_dir() {
        println!("Error: must be directory");
        return;
    }
    let folder = std::path::PathBuf::from(folder);
    let mut path = folder.clone();
    let name = args.next().unwrap_or(path.file_name().unwrap().to_str().unwrap().to_string());

    path.set_file_name(format!("{}.zip", name));
    println!("H {:?}", &path);
    let writer = std::fs::File::create(&path).unwrap();
    zip_dir(&folder, writer).unwrap();
}