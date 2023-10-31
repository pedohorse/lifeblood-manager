use chrono::prelude::*;
use fs_extra::file;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufWriter};
use std::io::{BufReader, Error};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, ExitStatus};

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
    has_viewer: bool,
}

pub struct InstallationsData {
    base_path: PathBuf,
    versions: Vec<InstalledVersion>,
    current_version: usize,
}

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
                        "hash in meta.info does not match dir name",
                    ));
                }
                date
            }
            Err(_) => {
                eprintln!("failed to read date from metadata, using dir creation time");
                path.metadata()?.created()?.into()
            }
        };

        // check viewer
        let has_viewer = path.join("lifeblood_viewer").exists();

        Ok(InstalledVersion {
            path,
            commit: file_name,
            date,
            has_viewer,
        })
    }

    pub fn source_commit(&self) -> &str {
        &self.commit
    }

    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
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
                            format!("failed to remove 'current' link: {}", e),
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
    /// TODO: provide option to get viewer too
    pub fn download_new_version(&mut self) -> Result<usize, Error> {
        macro_rules! noop {
            ($($t:tt)*) => {};
        }

        macro_rules! wraperr {
            ($($phase:literal)|+, $call:expr, $cleanup:ident!) => {
                match $call {
                    Ok(x) => x,
                    Err(e) => {

                        $cleanup!($($phase)|+);

                        return Err(Error::new(
                            e.kind(),
                            format!("{} failed: {}", concat!($($phase, ": "),+), e)
                        ))
                    }
                }
            };
            ($($phase:literal)|+, $call:expr) => {
                wraperr!($($phase)|+, $call, noop!);
            }
        }

        let mut cleanups: Vec<Box<dyn FnOnce() -> Result<(), Error>>> = Vec::new();
        //                Vec<impl Fn() -> Result<(), Error>>

        macro_rules! cleanup {
            ($($preverr:literal)?) => {
                for (i, cleanup) in cleanups.into_iter().enumerate() {
                    println!("cleaning up: {}", i);
                    wraperr!($($preverr |)?"cleanup phase", cleanup());
                }
            };
        }
        let do_install_viewer = true; // TODO make into an arg

        let temp_location = std::env::temp_dir();

        //
        // download phase
        let downloaded_zip = wraperr!(
            "download phase",
            Self::helper_download(&temp_location),
            cleanup!
        );
        let cleanup_downloaded_zip = downloaded_zip.clone();
        cleanups.push(Box::new(|| -> Result<(), Error> {
            println!("removing: {:?}", cleanup_downloaded_zip);
            fs::remove_file(cleanup_downloaded_zip)
        }));

        let unzip_location = downloaded_zip.with_extension("");
        if let Err(e) = fs::create_dir(&unzip_location) {
            // cleanup
            cleanup!();
            return Err(Error::new(
                e.kind(),
                format!("failed to create temp directory: {}", e),
            ));
        }
        let cleanup_unzip_location = unzip_location.clone();
        cleanups.push(Box::new(move || -> Result<(), Error> {
            println!("removing: {:?}", cleanup_unzip_location);
            fs::remove_dir_all(cleanup_unzip_location)
        }));

        //
        // unpacking phase
        let (hash, date) = wraperr!(
            "unpack phase",
            Self::helper_unpack(&downloaded_zip, &unzip_location),
            cleanup!
        );
        // removing dir already added to cleanup

        // check if already installed, maybe without viewer
        for (i, ver) in self.versions.iter().enumerate() {
            if ver.commit == hash && (ver.has_viewer || !do_install_viewer) {
                println!("latest commit already downloaded!");
                cleanup!();
                return Ok(i);
            }
        }

        // install
        let dest_dir = self.base_path.join(&hash);
        wraperr!("install phase", self.helper_install(&unzip_location, &dest_dir, do_install_viewer), cleanup!);

        // println!("imitating error!");
        // cleanup!();
        // return Err(Error::new(std::io::ErrorKind::Other, "foo test!"));

        // (re)make shortcuts
        Self::helper_make_script_link(&self.base_path.join("lifeblood"), "")?;
        if do_install_viewer {
            Self::helper_make_script_link(&self.base_path.join("lifeblood_viewer"), "viewer")?;
        }

        // save some metadata

        helper_save_metadata(&dest_dir.join("meta.info"), &hash, date.into())?;

        //
        // update versions list
        let inserted_index = Self::insert_version_sorted_by_date(
            &mut self.versions,
            InstalledVersion {
                path: dest_dir,
                commit: hash,
                date,
                has_viewer: do_install_viewer,
            },
        );
        if self.current_version != usize::MAX && inserted_index <= self.current_version {
            self.current_version += 1;
        }

        Ok(inserted_index)
    }

    ///
    /// insert into sorted list and return inserted index
    fn insert_version_sorted_by_date(
        versions: &mut Vec<InstalledVersion>,
        ver: InstalledVersion,
    ) -> usize {
        // we assume that versions list is always sorted
        match versions.binary_search_by_key(&ver.date, |v| v.date) {
            Ok(idx) | Err(idx) => {
                versions.insert(idx, ver);
                idx
            }
        }
    }

    /// helper func
    /// download latest commit
    fn helper_download(download_location: &Path) -> Result<PathBuf, Error> {
        let mut downloader = match Downloader::builder()
            .download_folder(&download_location)
            .connect_timeout(std::time::Duration::from_secs(90))
            .build()
        {
            Ok(x) => x,
            Err(e) => {
                // nothing to cleanup
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

        // download
        let target_filepath = temp_filename.with_extension("zip");
        let downloaded_zip =
            match downloader.download(&[Download::new(&url).file_name(&target_filepath)]) {
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
                                // cleanup
                                if target_filepath.exists() {
                                    fs::remove_file(target_filepath)?;
                                }
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
                    // cleanup
                    if target_filepath.exists() {
                        fs::remove_file(target_filepath)?;
                    }
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        format!("download failed: {:?}", e),
                    ));
                }
            };

        Ok(downloaded_zip)
    }

    /// helper func
    /// unpack zip
    fn helper_unpack(
        zip_file: &Path,
        unzip_location: &Path,
    ) -> Result<(String, DateTime<Utc>), Error> {
        let reader = BufReader::new(match File::open(zip_file) {
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
                Utc.with_ymd_and_hms(
                    zipdate.year() as i32,
                    zipdate.month() as u32,
                    zipdate.day() as u32,
                    zipdate.hour() as u32,
                    zipdate.minute() as u32,
                    zipdate.second() as u32,
                )
                .unwrap()
            }
            Err(e) => {
                return Err(Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("zip file empty? {}", e),
                ));
            }
        };

        if let Err(e) = zip_reader.extract(unzip_location) {
            // TODO: cleanup
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("unzip failed: {}", e),
            ));
        }

        Ok((hash, date))
    }

    /// helper func
    /// "install" the whole thing
    /// create all dirs, venv, copy stuff, etc
    fn helper_install(
        &mut self,
        unzip_location: &Path,
        dest_dir: &Path,
        do_install_viewer: bool,
    ) -> Result<(), Error> {
        let mut existing_dest: Option<PathBuf> = None;

        macro_rules! wraperr {
            ($text:literal, $call:expr) => {
                match $call {
                    Ok(x) => x,
                    Err(e) => {
                        error_cleanup!();
                        return Err(Error::new(e.kind(), format!("{} failed: {}", $text, e)));
                    }
                }
            };
        }

        macro_rules! error_cleanup {
            () => {
                if dest_dir.exists() {
                    fs::remove_dir_all(&dest_dir)?; // sloppy error report
                }
                match existing_dest {
                    Some(path) => {
                        fs::rename(&path, &dest_dir)?; // sloppy error report
                    }
                    None => (),
                }
            };
        }

        if dest_dir.exists() {
            let tmp_dest_dir = dest_dir.with_file_name(format!(
                "__{}",
                dest_dir.file_name().unwrap().to_str().unwrap()
            )); // may error...
            if tmp_dest_dir.exists() {
                fs::remove_dir_all(&tmp_dest_dir)?;
            }
            fs::rename(&dest_dir, &tmp_dest_dir)?;
            existing_dest = Some(tmp_dest_dir);
        }

        if let Err(e) = fs::create_dir(&dest_dir) {
            error_cleanup!();
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create destination dir: {}", e),
            ));
        }

        let inner_dir = match unzip_location.read_dir() {
            Ok(mut read_dir) => {
                // we expect a single folder inside
                if let Some(Ok(dir)) = read_dir.next() {
                    dir.path()
                } else {
                    error_cleanup!();
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "no dir suitable found in unzip location",
                    ));
                }
            }
            Err(e) => {
                error_cleanup!();
                return Err(Error::new(
                    e.kind(),
                    format!("error reading unpack dir: {}", e),
                ));
            }
        };

        // copy modules
        wraperr!(
            "copying modules",
            Self::helper_copy_dir(&inner_dir.join("src").join("lifeblood"), dest_dir)
        );
        if do_install_viewer {
            wraperr!(
                "copying modules",
                Self::helper_copy_dir(&inner_dir.join("src").join("lifeblood_viewer"), dest_dir)
            );
        }

        if let Err(e) = fs::copy(inner_dir.join("entry.py"), dest_dir.join("entry.py")) {
            error_cleanup!();
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("failed to copy unzipped contents: {}", e),
            ));
        }

        // do the venv

        // prepare requirements
        let requirements_path = dest_dir.join("requirements.txt");
        let reqs = wraperr!(
            "getting reqs",
            Self::helper_get_requirements_from_setupcfg(
                &inner_dir.join("pkg_lifeblood").join("setup.cfg"),
            )
        );
        wraperr!(
            "writing reqs",
            Self::helper_write_strings_to_file(reqs, &requirements_path)
        );

        let requirements_path_viewer = dest_dir.join("requirements_viewer.txt");
        let reqs = wraperr!(
            "getting reqs",
            Self::helper_get_requirements_from_setupcfg(
                &inner_dir.join("pkg_lifeblood_viewer").join("setup.cfg"),
            )
        );
        wraperr!(
            "writing reqs",
            Self::helper_write_strings_to_file(reqs, &requirements_path_viewer)
        );

        wraperr!(
            "installing to venv",
            Self::helper_install_venv(&dest_dir, &requirements_path)
        );
        if do_install_viewer {
            wraperr!(
                "installing to venv",
                Self::helper_install_venv(&dest_dir, &requirements_path_viewer)
            );
        }

        // all good, cleanup temp dir if used
        if let Some(path) = existing_dest {
            if let Err(e) = fs::remove_dir_all(&path) {
                eprintln!("failed to remove remporary dir {:?}, that is not important for the install, but please do it manually: {}", path, e);
            }
        }

        Ok(())
    }

    /// helper func
    /// copy dirs, with overwriting
    fn helper_copy_dir(src: &Path, dest: &Path) -> Result<(), Error> {
        let mut copy_options = CopyOptions::new();
        copy_options.overwrite = true;
        if let Err(e) = fs_extra::dir::copy(src, dest, &copy_options) {
            // TODO: cleanup
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("failed to copy unzipped contents: {}", e),
            ));
        };

        Ok(())
    }

    /// helper func
    /// somewhat parse setup.cfg and get requirements
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
                // skip deps on self
                if !line.starts_with("lifeblood") {
                    reqs.push(line.to_owned());
                }
            } else {
                let line = line.trim();
                found_start = line == "install_requires="
                    || line.starts_with("install_requires ") && line.ends_with("=");
            }
        }

        Ok(reqs)
    }

    /// helper func
    /// just write strings as we like it
    fn helper_write_strings_to_file(strings: Vec<String>, file_path: &Path) -> Result<(), Error> {
        let file = fs::File::create(file_path)?;
        let mut file = BufWriter::new(file);
        for line in strings.into_iter() {
            file.write(line.as_bytes())?;
            file.write(&[b'\n'])?;
        }

        Ok(())
    }

    /// helper func
    /// install venv phase
    fn helper_install_venv(dest_dir: &Path, requirements_path: &Path) -> Result<(), Error> {
        // if venv dir is present - skip creating venv
        if !dest_dir.join("venv").exists() {
            if let Some(python_command) = Self::helper_get_python_command() {
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
            } else {
                // python not found, but we know what to do on windows in this case
                if cfg!(windows) {
                    Self::helper_prepare_windows_venv(dest_dir)?
                } else {
                    return Err(Error::new(
                        std::io::ErrorKind::NotFound,
                        "python binary not found",
                    ));
                }
            }
        }

        // run pip
        let exit_status =
            match process::Command::new(dest_dir.join("venv").join("bin").join("python"))
                .current_dir(dest_dir)
                .arg("-m")
                .arg("pip")
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

    /// helper func
    /// download single file, that's it
    fn helper_download_single_file(
        url: &str,
        dest_dir: &Path,
        filename: Option<&str>,
        download_stage_name: &str,
    ) -> Result<PathBuf, Error> {
        let mut downloader = match Downloader::builder()
            .download_folder(&dest_dir)
            .connect_timeout(std::time::Duration::from_secs(90))
            .build()
        {
            Ok(x) => x,
            Err(e) => {
                // nothing to cleanup
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "could not initialize downloader for {:?} :( : {}",
                        download_stage_name, e
                    ),
                ));
            }
        };

        let mut download = Download::new(url);
        if let Some(filename) = filename {
            download = download.file_name(&dest_dir.join(filename));
        }

        let downloaded_file = match downloader.download(&[download]) {
            Ok(results) => {
                let mut path = PathBuf::new();
                for part in results {
                    match part {
                        Ok(summary) => {
                            println!("Ok: {}: downloaded {:?}", download_stage_name, summary);
                            path = summary.file_name;
                            break;
                        }
                        Err(e) => {
                            // cleanup
                            return Err(Error::new(
                                std::io::ErrorKind::Other,
                                format!("download {} failed: {:?}", download_stage_name, e),
                            ));
                        }
                    }
                }
                path
            }
            Err(e) => {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("download {} failed: {:?}", download_stage_name, e),
                ))
            }
        };

        Ok(downloaded_file)
    }

    /// helper func
    /// gets embedded python for windows case
    fn helper_prepare_windows_venv(dest_dir: &Path) -> Result<(), Error> {
        let pyver = "3.10.9"; // TODO: do not hardcode
        let pycode = "3.10";
        //

        let pyzip = Self::helper_download_single_file(
            &format!(
                "https://www.python.org/ftp/python/{}/python-{}-embed-win32.zip",
                pyver, pyver
            ),
            &dest_dir,
            None,
            "get python embedded",
        )?;

        let venv_path = dest_dir.join("venv");
        if !venv_path.exists() {
            fs::create_dir(&venv_path)?;
        }
        let venv_bin_path = venv_path.join("bin");
        if !venv_bin_path.exists() {
            fs::create_dir(&venv_bin_path)?;
        }
        Self::helper_unpack(&pyzip, &venv_bin_path)?;

        // get pip.pyz
        let getpip = Self::helper_download_single_file(
            "https://bootstrap.pypa.io/get-pip.py",
            &venv_bin_path,
            None,
            "get pip",
        )?;

        // write pyvenv
        fs::write(
            venv_path.join("pyvenv.cfg"),
            "include-system-site-packages = false",
        )?;
        // write special _pth file
        fs::write(
            venv_bin_path.join(format!("python{}._pth", pycode)),
            "import site",
        )?;

        // now run get-pip.py script
        let exit_status = match process::Command::new(venv_bin_path.join("python"))
            .current_dir(dest_dir)
            .arg(getpip)
            .status()
        {
            Ok(status) => status,
            Err(e) => {
                return Err(Error::new(e.kind(), format!("error running python: {}", e)));
            }
        };
        check_status!(exit_status);

        Ok(())
    }

    /// helper func
    /// find python executable
    fn helper_get_python_command() -> Option<PathBuf> {
        // TODO: do checks, use env variable or smth
        //  propagate errors
        let pypath = if let Ok(x) = std::env::var("PYTHON_BIN") {
            PathBuf::from(x)
        } else {
            PathBuf::from("python")
        };

        if let Err(_) = process::Command::new(&pypath).arg("--version").status() {
            // we don't care about result, just that it ran or no
            return None;
        }

        Some(pypath)
    }

    /// helper func
    /// make common lifeblood link files
    /// used to create lifeblood, lifeblood_viewer
    fn helper_make_script_link(file_path: &Path, entry_arg: &str) -> Result<(), Error> {
        #[cfg(windows)]
        let file_path = &file_path.with_extension("cmd");

        let contents = if cfg!(unix) {
            format!(
                "#!/bin/sh\n\
                cwd=`dirname \\`readlink -f $0\\``\n\
                exec $cwd/current/venv/bin/python $cwd/current/entry.py {} \"$@\"",
                entry_arg
            )
        } else if cfg!(windows) {
            format!(
                "@echo off\n\
                 %~dp0\\current\\bin\\python %~dp0\\current\\entry.py {} %*",
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

/// helper func
/// save metadata file
fn helper_save_metadata(
    info_file_path: &Path,
    commit_info: &str,
    date: DateTime<Utc>,
) -> Result<(), Error> {
    let mut file = match fs::File::create(info_file_path) {
        Ok(file) => BufWriter::new(file),
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("failed to create metadata file: {}", e),
            ));
        }
    };

    write!(file, "1\n")?;
    write!(file, "{}\n", commit_info)?;
    write!(file, "{:?}", date)?;

    Ok(())
}

/// helper func
/// read back previously saved metadata file
fn helper_read_metadata(info_file_path: &Path) -> Result<(String, DateTime<Utc>), Error> {
    let mut file = match fs::File::open(info_file_path) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            return Err(Error::new(
                e.kind(),
                format!("failed to create metadata file: {}", e),
            ));
        }
    };

    let mut buf = String::new();
    file.read_line(&mut buf)?;
    if buf.trim() != "1" {
        return Err(Error::new(
            std::io::ErrorKind::Other,
            format!("unknown metadata format version: {}", buf),
        ));
    };

    buf.clear();
    file.read_line(&mut buf)?;
    let commit = buf.trim().to_owned();

    buf.clear();
    file.read_line(&mut buf)?;
    let date: DateTime<Utc> = if let Ok(x) = buf.trim().parse() {
        x
    } else {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "incorrect date format in metadata",
        ));
    };

    Ok((commit, date))
}
