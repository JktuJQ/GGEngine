//! `datacore::assets` submodule provides traits and structs that encapsulate work with assets.
//!
//! It defines traits like [`FromFile`] and [`ToFile`] that are implemented on structs which are
//! either serializable or deserializable.
//! You can find more about data formats that `ggengine` provides in [`AssetFormat`] enum.
//!
//! `ggengine` serializes `Rust`-side data by using Concise Binary Object Representation format.
//! It is encouraged to read the docs to find out about other types of data.
//!

use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, create_dir_all, File},
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

/// [`FromFile`] trait is implemented on objects that can be restored from file data (deserialized).
///
/// There is an auto implementation on all types that implement `serde::Deserialize` and there are also
/// several manual implementations on data formats that are not deserializable by `serde` (audio, images and fonts).
///
pub trait FromFile {
    /// Deserializes object from file.
    ///
    fn from_file(filename: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized;
}
impl<T: for<'a> Deserialize<'a>> FromFile for T {
    /// Restores data from given file.
    ///
    /// This function returns an error if file does not exist or if data is not recoverable.
    ///
    fn from_file(filename: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(filename)?;
        serde_cbor::from_reader(file)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Wrong data format"))
    }
}
/// [`ToFile`] trait is implemented on objects that can be saved to file (serialized).
///
/// There is an auto implementation on all types that implement `serde::Serialized` and there is
/// also manual implementation for `Image`.
///
/// To hold up certain constraints, this trait is not implemented on types such as `Sound`, `Music` and `PartialFont`,
/// because those are fully external to `ggengine` and `ggengine` cannot change them,
/// so it's pointless to serialize data that can only be retrieved externally or
/// to serialize objects that are initialized from data that is serializable.
///
pub trait ToFile {
    /// Serializes object to file.
    ///
    fn to_file(&self, filename: impl AsRef<Path>) -> Result<(), Error>;
}
impl<T: Serialize> ToFile for T {
    /// Saves data to file.
    ///
    /// This implementation will create a file if it does not exist, and will truncate it if it does.
    /// All manual implementations should follow this principle for uniformity.
    ///
    /// This function fails if file creation or truncation fails or if data is not
    /// serializable by CBOR.
    ///
    fn to_file(&self, filename: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(filename)?;
        serde_cbor::to_writer(file, self)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Wrong data format"))
    }
}

/// [`AssetFormat`] trait registers types of assets by reserving a folder for them.
///
/// `ggengine` provides few implementors, which are quite common.
/// Implementation of [`AssetFormat`] on [`AudioAssetFormat`] means that
/// `ggengine` will store data which is marked as audio in the folder with the name 'audio'.
///
pub trait AssetFormat {
    /// Name of folder in which assets of this format will be stored.
    ///
    fn format_folder(&self) -> &'static str;
}
/// [`AudioAssetFormat`] represents `ggengine`'s audio data.
/// Its folder is 'audio'.
///
#[derive(Clone, Copy, Debug)]
pub struct AudioAssetFormat;
impl AssetFormat for AudioAssetFormat {
    fn format_folder(&self) -> &'static str {
        "audio"
    }
}
/// [`ImageAssetFormat`] represents `ggengine`'s image data.
/// Its folder is 'images'.
///
#[derive(Debug, Clone, Copy)]
pub struct ImageAssetFormat;
impl AssetFormat for ImageAssetFormat {
    fn format_folder(&self) -> &'static str {
        "images"
    }
}
/// [`FontAssetFormat`] represents `ggengine`'s font data.
/// Its folder is 'fonts'.
///
#[derive(Debug, Clone, Copy)]
pub struct FontAssetFormat;
impl AssetFormat for FontAssetFormat {
    fn format_folder(&self) -> &'static str {
        "fonts"
    }
}
/// [`ConfigAssetFormat`] represents config data.
/// Its folder is 'configs'.
///
#[derive(Debug, Clone, Copy)]
pub struct ConfigAssetFormat;
impl AssetFormat for ConfigAssetFormat {
    fn format_folder(&self) -> &'static str {
        "configs"
    }
}

/// [`AssetManager`] struct manages directory by converting it to (or treating it as) a nice storage for game assets.
///
/// Initializing [`AssetManager`] in a directory creates several new directories (if they are not present):
/// with corresponding `AssetFormat`s will be saved in those directories, while others are
/// saved in root directory.
///
/// [`AssetManager`] provides nice API by encapsulating filesystem work, and it should be used as a main
/// construct for managing game assets.
///
#[derive(Debug)]
pub struct AssetManager {
    /// Directory that is being handled by [`AssetManager`].
    ///
    root_directory: PathBuf,
}
impl AssetManager {
    /// Constructs full path for asset using its metadata.
    ///
    fn full_path(&self, filename: impl AsRef<Path>, format: impl AssetFormat) -> PathBuf {
        self.root_directory
            .as_path()
            .join(format.format_folder())
            .join(filename)
    }

    /// Initializes [`AssetManager`] in a directory.
    ///
    /// ```rust, no_run
    /// # use ggengine::datacore::assets::{AssetManager, AssetFormat, AudioAssetFormat, ImageAssetFormat, FontAssetFormat, ConfigAssetFormat};
    /// let manager: AssetManager = AssetManager::initialize_at(
    ///     "assets",
    ///     &[
    ///         AudioAssetFormat.format_folder(),
    ///         ImageAssetFormat.format_folder(),
    ///         FontAssetFormat.format_folder(),
    ///         ConfigAssetFormat.format_folder()
    ///     ]
    /// ).expect("Filename should be correct");
    /// ```
    ///
    pub fn initialize_at(path: impl AsRef<Path>, formats: &[&'static str]) -> Result<Self, Error> {
        if !path.as_ref().is_dir() {
            create_dir_all(&path)?;
        }
        for format in formats {
            create_dir(path.as_ref().join(format).as_path())?;
        }

        Ok(AssetManager {
            root_directory: path.as_ref().to_path_buf(),
        })
    }

    /// Save asset using its metadata.
    ///
    /// ```rust, no_run
    /// # use ggengine::datacore::assets::{AssetManager, AssetFormat, ConfigAssetFormat};
    /// # use std::path::PathBuf;
    /// let manager: AssetManager = AssetManager::initialize_at(
    ///     "assets",
    ///     &[ConfigAssetFormat.format_folder()]
    /// ).expect("Filename should be correct");
    ///
    /// let asset: String = String::from("data");
    /// manager.save_asset(
    ///     PathBuf::from("asset.data"),
    ///     ConfigAssetFormat,
    ///     &asset)
    /// .expect("Metadata should be correct");
    /// ```
    ///
    pub fn save_asset<T: ToFile>(
        &self,
        filename: impl AsRef<Path>,
        format: impl AssetFormat,
        asset: &T,
    ) -> Result<(), Error> {
        asset.to_file(self.full_path(filename, format).as_path())
    }
    /// Loads asset using its metadata.
    ///
    /// ```rust, no_run
    /// # use ggengine::datacore::assets::{AssetManager, AssetFormat, ConfigAssetFormat};
    /// # use std::path::PathBuf;
    /// let manager: AssetManager = AssetManager::initialize_at(
    ///     "assets",
    ///     &[ConfigAssetFormat.format_folder()]
    /// ).expect("Filename should be correct");
    ///
    /// let asset: String = manager.load_asset(
    ///     PathBuf::from("asset.data"),
    ///     ConfigAssetFormat,
    /// ).expect("Metadata should be correct");
    /// ```
    ///
    pub fn load_asset<T: FromFile>(
        &self,
        filename: impl AsRef<Path>,
        format: impl AssetFormat,
    ) -> Result<T, Error> {
        T::from_file(self.full_path(filename, format).as_path())
    }
}
