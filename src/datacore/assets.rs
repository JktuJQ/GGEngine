//! `datacore::assets` submodule provides traits and structs that encapsulate work with assets.
//!
//! It defines traits like [`FromFile`] and [`ToFile`] that are implemented on structs which are
//! either serializable or deserializable, but those traits are also implemented on `Sound`, `Image` etc.
//! for which `serde`'s serialization and deserialization is not applicable.
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
    fn from_file(filename: impl AsRef<Path>) -> Result<Self, Error> {
        let file: File = File::open(filename)?;
        serde_cbor::from_reader(file)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Wrong data format"))
    }
}

/// [`ToFile`] trait is implemented on objects that can be saved to file (serialized).
///
/// There is an auto implementation on all types that implement `serde::Serialized`.
///
/// There are also several implementations on data formats that are not deserializable by `serde` (audio, images and fonts),
/// but for audio and fonts they are no-op (and are implemented only for uniformity)
/// since it's pointless to serialize data that can only be retrieved externally or
/// for objects that are initialized from data that is serializable.
///
pub trait ToFile {
    /// Serializes object to file.
    ///
    fn to_file(&self, filename: impl AsRef<Path>) -> Result<(), Error>;
}
impl<T: Serialize> ToFile for T {
    /// Saves data to file.
    ///
    fn to_file(&self, filename: impl AsRef<Path>) -> Result<(), Error> {
        let file: File = File::create(filename)?;
        serde_cbor::to_writer(file, self)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Wrong data format"))
    }
}

/// [`AssetFormat`] enum lists all kinds of data that `datacore` supports.
///
/// That includes audio, image and font formats. `AssetFormat::Other` case is left for data that is
/// native to `Rust` side (structs that are serialized via `serde`) or data that does not fit for other categories.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AssetFormat {
    /// Audio data.
    ///
    Audio,
    /// Image data.
    ///
    Image,
    /// Font data.
    ///
    Font,
    /// `Rust`-side data.
    ///
    Other,
}
impl AssetFormat {
    /// List of all [`AssetFormat`] variants.
    ///
    const FORMATS: [AssetFormat; 4] = [Self::Audio, Self::Image, Self::Font, Self::Other];

    /// Converts format to corresponding name (name of directory in which those formats are stored).
    ///
    fn to_str(self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Image => "images",
            Self::Font => "fonts",
            _ => "",
        }
    }
}
/// [`AssetMetadata`] struct represents metadata of any asset: it should have filename and specific format.
///
#[derive(Clone, Debug)]
pub struct AssetMetadata {
    /// Name of a loaded asset file.
    ///
    pub filename: PathBuf,
    /// Format of the asset.
    ///
    pub format: AssetFormat,
}

/// [`AssetManager`] struct manages directory by converting it to (or treating it as) a nice storage for game assets.
///
/// Initializing [`AssetManager`] in a directory automatically creates several new directories (if they are not present):
/// 'audio/', 'images/' and 'fonts/'. Assets with corresponding `AssetFormat`s will be saved in those directories, while others are
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
    fn full_path(&self, data: AssetMetadata) -> PathBuf {
        self.root_directory
            .as_path()
            .join(data.format.to_str())
            .join(data.filename)
    }

    /// Initializes [`AssetManager`] in a directory.
    ///
    /// ```rust, no_run
    /// # use ggengine::datacore::assets::AssetManager;
    /// let manager: AssetManager = AssetManager::initialize_at("assets").expect("Filename should be correct");
    /// ```
    ///
    pub fn initialize_at(path: impl AsRef<Path>) -> Result<Self, Error> {
        if !path.as_ref().is_dir() {
            create_dir_all(&path)?;
        }
        for format in AssetFormat::FORMATS {
            create_dir(path.as_ref().join(format.to_str()).as_path())?;
        }

        Ok(AssetManager {
            root_directory: path.as_ref().to_path_buf(),
        })
    }

    /// Save asset using its metadata.
    ///
    /// ```rust, no_run
    /// # use ggengine::datacore::assets::{AssetManager, AssetMetadata, AssetFormat};
    /// # use std::path::PathBuf;
    /// let manager: AssetManager = AssetManager::initialize_at("assets").expect("Filename should be correct");
    ///
    /// let asset: String = String::from("data");
    /// manager.save_asset(AssetMetadata {
    ///     filename: PathBuf::from("asset.data"),
    ///     format: AssetFormat::Other,
    /// }, &asset).expect("Metadata should be correct");
    /// ```
    ///
    pub fn save_asset<T: ToFile>(&self, data: AssetMetadata, asset: &T) -> Result<(), Error> {
        asset.to_file(self.full_path(data).as_path())
    }
    /// Loads asset using its metadata.
    ///
    /// ```rust, no_run
    /// # use ggengine::datacore::assets::{AssetManager, AssetMetadata, AssetFormat};
    /// # use std::path::PathBuf;
    /// let manager: AssetManager = AssetManager::initialize_at("assets").expect("Filename should be correct");
    ///
    /// let asset: String = manager.load_asset(AssetMetadata {
    ///     filename: PathBuf::from("asset.data"),
    ///     format: AssetFormat::Other,
    /// }).expect("Metadata should be correct");
    /// ```
    ///
    pub fn load_asset<T: FromFile>(&self, data: AssetMetadata) -> Result<T, Error> {
        T::from_file(self.full_path(data).as_path())
    }
}
