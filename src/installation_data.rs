use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufWriter};
use std::io::{BufReader, Error};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, ExitStatus};
use chrono::prelude::*;

use downloader::{Download, Downloader};

use fs_extra::dir::CopyOptions;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use zip::ZipArchive;

#[derive(Debug)]
pub struct InstalledVersion {
    path: PathBuf,
    commit: String,
    date: DateTime<Utc>,
}

pub struct InstallationsData {
    base_path: PathBuf,
    versions: Vec<InstalledVersion>,
    current_version: usize,
}

impl InstalledVersion {
    pub fn from_path(path: PathBuf) -> Result<InstalledVersion, Error> {
        if !path.is_dir() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "given path is not a dir",
            ));
        };
        let file_name: String = match path.file_name() {
            Some(x) => x.to_string_lossy().to_string(),
            None => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "given path has incorrect file name",
                ))
            }
        };
        // try read metadata
        let date = match helper_read_metadata(&path.join("meta.info")) {
            Ok((hash, date)) => {
                if file_name != hash {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "hash in meta.info does not match dir name"
                    ))
                }
                date
            }
            Err(_) => {
                eprintln!("failed to read date from metadata, using dir creation time");
                path.metadata()?.created()?.into()
            }
        };

        Ok(InstalledVersion {
            path,
            commit: file_name,
            date,
        })
    }

    pub fn source_commit(&self) -> &str {
        &self.commit
    }
}

impl InstallationsData {
    ///
    /// construct new installations data scanning given dir
    pub fn from_dir(base_path: PathBuf) -> Result<InstallationsData, Error> {
        let mut versions = Vec::new();
        let mut current_version = usize::MAX;
        let mut current_path = PathBuf::new();

        match fs::read_dir(&base_path) {
            Ok(dir_iter) => {
                for entry in dir_iter {
                    let entry = match entry {
                        Ok(x) => x,
                        Err(e) => {
                            println!("skipping failed dir entry: {}", e);
                            continue;
                        }
                    };

                    let path = entry.path();
                    match path {
                        // case it's 'current' link
                        path if path.is_symlink() && path.ends_with("current") => {
                            match path.read_link() {
                                Ok(link_target) => {
                                    current_path = if link_target.is_absolute() {
                                        link_target
                                    } else {
                                        if let Some(parent) = path.parent() {
                                            parent.join(link_target)
                                        } else {
                                            eprintln!(
                                                "failed to understand where link is pointing {:?}",
                                                link_target
                                            );
                                            continue;
                                        }
                                    }
                                }
                                Err(_) => {
                                    eprintln!(
                                        "thought {:?} is a link, but cannot read it, skipping",
                                        path
                                    );
                                }
                            }
                        }
                        // case it's a dir
                        path if path.is_dir() => match InstalledVersion::from_path(path) {
                            Ok(info) => {
                                Self::insert_version_sorted_by_date(&mut versions, info);
                            }
                            Err(_) => {
                                eprintln!("'{:?}' does not look like a version", entry.path());
                                continue;
                            }
                        },
                        path => {
                            println!("skipping {:?}", path);
                            continue;
                        }
                    }
                }
            }
            Err(e) => return Err(e),
        }

        println!("curr path {:?}", current_path);
        for (i, ver) in versions.iter().enumerate() {
            if ver.path == current_path {
                current_version = i;
                println!("curr {}", current_version);
            }
        }

        Ok(InstallationsData {
            base_path,
            versions,
            current_version,
        })
    }

    pub fn version(&self, i: usize) -> Option<&InstalledVersion> {
        self.versions.get(i)
    }

    pub fn current_version(&self) -> Option<&InstalledVersion> {
        if self.current_version == usize::MAX {
            None
        } else {
            if let Some(ver) = self.versions.get(self.current_version) {
                Some(ver)
            } else {
                None
            }
        }
    }

    pub fn current_version_index(&self) -> usize {
        self.current_version
    }

    pub fn iter_versions(&self) -> impl Iterator<Item = &InstalledVersion> + '_ {
        self.versions.iter()
    }

    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    pub fn make_version_current(&mut self, i: usize) -> Result<(), Error> {
        match self.versions.get(i) {
            Some(ver) => {
                let path_to_current = self.base_path.join("current");
                if path_to_current.is_symlink() || path_to_current.exists() {
                    if let Err(e) = fs::remove_file(&path_to_current) {
                        return Err(Error::new(
                            e.kind(),
                            format!("failed to remove 'current' link: {}", e)
                        ));
                    }
                }
                // try to get a relpath
                let path_to_ver = if let Ok(path) = ver.path.strip_prefix(&self.base_path) {
                    path
                } else {
                    &ver.path
                };

                #[cfg(windows)]
                if let Err(e) = std::os::windows::fs::symlink_dir(path_to_ver, &path_to_current) {
                    return Err(e);
                }

                #[cfg(unix)]
                if let Err(e) = std::os::unix::fs::symlink(path_to_ver, &path_to_current) {
                    // TODO: restore prev current
                    return Err(e);
                }

                self.current_version = i;
                Ok(())
            }
            None => Err(Error::new(std::io::ErrorKind::NotFound, "no such version")),
        }
    }

    ///
    /// download freshest commit from given branch, and makes an "installation" of it
    ///
    /// TODO: give branch
    pub fn download_new_version(&mut self) -> Result<usize, Error> {
        let temp_location = std::env::temp_dir();
        let mut downloader = match Downloader::builder()
            .download_folder(&temp_location)
            .connect_timeout(std::time::Duration::from_secs(90))
            .build()
        {
            Ok(x) => x,
            Err(e) => {
                // TODO: cleanup
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    "could not initialize downloader :(",
                ));
            }
        };

        let mut rng = thread_rng();
        let branch = "dev";
        let url = format!(
            "https://github.com/pedohorse/lifeblood/archive/refs/heads/{}.zip",
            branch
        );
        let temp_filename: PathBuf = PathBuf::from(
            (0..16)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect::<String>(),
        );
        let unzip_location = std::env::temp_dir().join(&temp_filename);
        if let Err(e) = fs::create_dir(&unzip_location) {
            // TODO: cleanup
            return Err(e);
        }

        //
        // download phase
        let downloaded_zip = match downloader
            .download(&[Download::new(&url).file_name(&temp_filename.with_extension("zip"))])
        {
            Ok(results) => {
                let mut path = PathBuf::new();
                for part in results {
                    match part {
                        Ok(summary) => {
                            println!("Ok: downloaded {:?}", summary);
                            path = summary.file_name;
                            break;
                        }
                        Err(e) => {
                            // TODO: cleanup
                            return Err(Error::new(
                                std::io::ErrorKind::Other,
                                format!("download failed: {:?}", e),
                            ));
                        }
                    }
                }
                path
            }
            Err(e) => {
                // TODO: cleanup
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("download failed: {:?}", e),
                ));
            }
        };

        //
        // unpacking phase
        let reader = BufReader::new(match File::open(&downloaded_zip) {
            Ok(x) => x,
            Err(e) => {
                // TODO: cleanup
                return Err(e);
            }
        });
        let mut zip_reader = match ZipArchive::new(reader) {
            Ok(x) => x,
            Err(e) => {
                // TODO: cleanup
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("failed to read back zip file: {}", e),
                ));
            }
        };

        let hash = String::from_utf8_lossy(zip_reader.comment())[..13].to_string();
        let date = match zip_reader.by_index(0) {
            Ok(x) => {
                let zipdate = x.last_modified();
                Utc.with_ymd_and_hms(zipdate.year() as i32, zipdate.month() as u32, zipdate.day() as u32, zipdate.hour() as u32, zipdate.minute() as u32, zipdate.second() as u32).unwrap()
                
            }
            Err(e) => {
                return Err(Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("zip file empty? {}", e),
                ));
            }
        };
        // check if already downloaded
        for (i, ver) in self.versions.iter().enumerate() {
            if ver.commit == hash {
                println!("latest commit already downloaded!");
                return Ok(i);
            }
        }

        if let Err(e) = zip_reader.extract(&unzip_location) {
            // TODO: cleanup
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("unzip failed: {}", e),
            ));
        }

        let do_install_viewer = true;

        let dest_dir = self.base_path.join(&hash);
        self.helper_install(&unzip_location, &dest_dir, do_install_viewer)?;

        // (re)make shortcuts
        Self::helper_make_script_link(&self.base_path.join("lifeblood"), "lifeblood.launch")?;
        if do_install_viewer {
            Self::helper_make_script_link(
                &self.base_path.join("lifeblood_viewer"),
                "lifeblood_viewer.launch",
            )?;
        }

        // save some metadata
        
        helper_save_metadata(&dest_dir.join("meta.info"), &hash, date.into())?;

        //
        // update versions list
        let inserted_index = Self::insert_version_sorted_by_date(&mut self.versions, InstalledVersion {
            path: dest_dir,
            commit: hash,
            date,
        });
        if self.current_version != usize::MAX && inserted_index <= self.current_version {
            self.current_version += 1;
        }

        Ok(inserted_index)
    }

    ///
    /// insert into sorted list and return inserted index
    fn insert_version_sorted_by_date(versions: &mut Vec<InstalledVersion>, ver: InstalledVersion) -> usize {
        // we assume that versions list is always sorted
        match versions.binary_search_by_key(&ver.date, |v| { v.date }) {
            Ok(idx) | Err(idx) => {
                versions.insert(idx, ver);
                idx
            }
        }
    }

    fn helper_install(
        &mut self,
        unzip_location: &Path,
        dest_dir: &Path,
        do_install_viewer: bool,
    ) -> Result<(), Error> {
        if !dest_dir.exists() {
            if let Err(e) = fs::create_dir(&dest_dir) {
                // TODO: cleanup
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("failed to create destination dir: {}", e),
                ));
            }
        }
        let inner_dir = match unzip_location.read_dir() {
            Ok(mut read_dir) => {
                // we expect a single folder inside
                if let Some(Ok(dir)) = read_dir.next() {
                    dir.path()
                } else {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "no dir suitable found in unzip location",
                    ));
                }
            }
            Err(e) => {
                return Err(e);
            }
        };

        // copy modules
        Self::helper_copy_dir(&inner_dir.join("src").join("lifeblood"), dest_dir)?;
        if do_install_viewer {
            Self::helper_copy_dir(&inner_dir.join("src").join("lifeblood_viewer"), dest_dir)?;
        }

        if let Err(e) = fs::copy(inner_dir.join("entry.py"), dest_dir.join("entry.py")) {
            // TODO: cleanup
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("failed to copy unzipped contents: {}", e),
            ));
        }

        // do the venv

        // prepare requirements
        let requirements_path = dest_dir.join("requirements.txt");
        let reqs = Self::helper_get_requirements_from_setupcfg(
            &inner_dir.join("pkg_lifeblood").join("setup.cfg"),
        )?;
        Self::helper_write_strings_to_file(reqs, &requirements_path)?;

        let requirements_path_viewer = dest_dir.join("requirements_viewer.txt");
        let reqs = Self::helper_get_requirements_from_setupcfg(
            &inner_dir.join("pkg_lifeblood_viewer").join("setup.cfg"),
        )?;
        Self::helper_write_strings_to_file(reqs, &requirements_path_viewer)?;

        Self::helper_install_venv(&dest_dir, &requirements_path)?;
        if do_install_viewer {
            Self::helper_install_venv(&dest_dir, &requirements_path_viewer)?;
        }

        Ok(())
    }

    fn helper_copy_dir(src: &Path, dest: &Path) -> Result<(), Error> {
        let copy_options = CopyOptions::new();
        if let Err(e) = fs_extra::dir::copy(src, dest, &copy_options) {
            // TODO: cleanup
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("failed to copy unzipped contents: {}", e),
            ));
        };

        Ok(())
    }

    fn helper_get_requirements_from_setupcfg(path: &Path) -> Result<Vec<String>, Error> {
        let mut config_cfg = String::new();
        match fs::File::open(path) {
            Ok(mut file) => {
                if let Err(e) = file.read_to_string(&mut config_cfg) {
                    // TODO: cleanup
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        format!("failed to open setup.cfg: {}", e),
                    ));
                }
            }
            Err(e) => {
                // TODO: cleanup
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("failed to open setup.cfg: {}", e),
                ));
            }
        };

        // a bit hacky way of getting info from setup.cfg, as it's not really a toml, and i'm not sure about it's syntax
        let mut reqs = Vec::new();
        let mut found_start = false;
        for line in config_cfg.lines() {
            if found_start {
                let line = line.trim();
                if line.len() == 0 || line.starts_with("[") {
                    break;
                }
                reqs.push(line.to_owned());
            } else {
                found_start = line.trim() == "install_requires =";
            }
        }

        Ok(reqs)
    }

    fn helper_write_strings_to_file(strings: Vec<String>, file_path: &Path) -> Result<(), Error> {
        let file = fs::File::create(file_path)?;
        let mut file = BufWriter::new(file);
        for line in strings.into_iter() {
            file.write(line.as_bytes())?;
            file.write(&[b'\n'])?;
        }

        Ok(())
    }

    fn helper_install_venv(dest_dir: &Path, requirements_path: &Path) -> Result<(), Error> {
        macro_rules! check_status {
            ($exit_status:ident) => {
                if !$exit_status.success() {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "python process exited with status: {}",
                            if let Some(ex) = $exit_status.code() {
                                ex
                            } else {
                                -1
                            }
                        ),
                    ));
                };
            };
        }

        // if venv dir is present - skip creating venv
        if !dest_dir.join("venv").exists() {
            let python_command = Self::helper_get_python_command()?;
            let exit_status = match process::Command::new(python_command)
                .current_dir(dest_dir)
                .arg("-m")
                .arg("venv")
                .arg("venv")
                .status()
            {
                Ok(status) => status,
                Err(e) => {
                    return Err(Error::new(e.kind(), format!("error running python: {}", e)));
                }
            };

            check_status!(exit_status);
        }

        // run pip
        let exit_status = match process::Command::new(dest_dir.join("venv").join("bin").join("pip"))
            .current_dir(dest_dir)
            .arg("install")
            .arg("-r")
            .arg(requirements_path)
            .status()
        {
            Ok(status) => status,
            Err(e) => {
                return Err(Error::new(e.kind(), format!("error running pip: {}", e)));
            }
        };
        check_status!(exit_status);

        Ok(())
    }

    fn helper_get_python_command() -> Result<PathBuf, Error> {
        // TODO: do checks, use env variable or smth
        //  propagate errors
        if let Ok(x) = std::env::var("PYTHON_BIN") {
            return Ok(PathBuf::from(x));
        }
        Ok(PathBuf::from("python"))
    }

    fn helper_make_script_link(file_path: &Path, entry_arg: &str) -> Result<(), Error> {
        #[cfg(windows)]
        let file_path = &file_path.with_extension("cmd");

        let contents = if cfg!(unix) {
            format!(
                "#!/bin/sh\n\
                exec `dirname \\`readlink -f $0\\``/current/venv/bin/python -m {} \"$@\"",
                entry_arg
            )
        } else if cfg!(windows) {
            format!(
                "@echo off\n\
                 %~dp0\\current\\bin\\python -m {} %*",
                entry_arg
            )
        } else {
            return Err(Error::new(
                std::io::ErrorKind::Unsupported,
                "unsupported platform",
            ));
        };

        // write
        match fs::File::create(file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write(contents.as_bytes()) {
                    return Err(Error::new(
                        e.kind(),
                        format!("failed to write to shortcut file: {}", e),
                    ));
                }
            }
            Err(e) => {
                return Err(Error::new(
                    e.kind(),
                    format!("failed to create shortcut script: {}", e),
                ));
            }
        };

        #[cfg(unix)]
        {
            // set unix permissions
            let mut perms = match fs::metadata(file_path) {
                Ok(m) => m.permissions(),
                Err(e) => {
                    return Err(Error::new(
                        e.kind(),
                        format!("failed to set permissions on shortcut: {}", e),
                    ));
                }
            };
            perms.set_mode(perms.mode() | 0o111);
            if let Err(e) = fs::set_permissions(file_path, perms) {
                return Err(Error::new(
                    e.kind(),
                    format!("failed to set permissions on shortcut: {}", e),
                ));
            }
        }

        Ok(())
    }

}


fn helper_save_metadata(info_file_path: &Path, commit_info: &str, date: DateTime<Utc>) -> Result<(), Error> {
    let mut file = match fs::File::create(info_file_path) {
        Ok(file) => {
            BufWriter::new(file)
        }
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("failed to create metadata file: {}", e)
            ));
        }
    };

    write!(file, "1\n")?;
    write!(file, "{}\n", commit_info)?;
    write!(file, "{:?}", date)?;

    Ok(())
}

fn helper_read_metadata(info_file_path: &Path) -> Result<(String, DateTime<Utc>), Error> {
    let mut file = match fs::File::open(info_file_path) {
        Ok(file) => {
            BufReader::new(file)
        }
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("failed to create metadata file: {}", e)
            ));
        }
    };

    let mut buf = String::new();
    file.read_line(&mut buf)?;
    if buf.trim() != "1" {
        return Err(Error::new(std::io::ErrorKind::Other, format!("unknown metadata format version: {}", buf)));
    };
    
    buf.clear();
    file.read_line(&mut buf)?;
    let commit = buf.trim().to_owned();

    buf.clear();
    file.read_line(&mut buf)?;
    let date: DateTime<Utc> = if let Ok(x) = buf.trim().parse() { x } else {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "incorrect date format in metadata"));
    };

    Ok((commit, date))
}