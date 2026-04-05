use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{config::NoiseRecipe2, grid::GridRequest2, image::NoiseImageSettings};

/// A loadable noise configuration asset.
///
/// Can be loaded from `.noise.ron` or `.noise.json` files via the Bevy asset server.
///
/// ```ron
/// (
///     recipe: Fbm(
///         source: Perlin((seed: (42))),
///         config: (octaves: 6, base_frequency: 1.5, lacunarity: 2.0, gain: 0.5, amplitude: 1.0),
///     ),
///     grid: (size: (256, 256), space: (min: (-2.0, -2.0), max: (2.0, 2.0))),
/// )
/// ```
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct NoiseAsset {
    pub recipe: NoiseRecipe2,
    #[serde(default)]
    pub grid: GridRequest2,
    #[serde(default)]
    pub image: NoiseImageSettings,
}

#[derive(Default, bevy::reflect::TypePath)]
pub struct NoiseRonAssetLoader;

impl AssetLoader for NoiseRonAssetLoader {
    type Asset = NoiseAsset;
    type Settings = ();
    type Error = NoiseAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(NoiseAssetLoaderError::Io)?;
        let asset: NoiseAsset = ron::de::from_bytes(&bytes).map_err(NoiseAssetLoaderError::Ron)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["noise.ron"]
    }
}

#[derive(Default, bevy::reflect::TypePath)]
pub struct NoiseJsonAssetLoader;

impl AssetLoader for NoiseJsonAssetLoader {
    type Asset = NoiseAsset;
    type Settings = ();
    type Error = NoiseAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(NoiseAssetLoaderError::Io)?;
        let asset: NoiseAsset =
            serde_json::from_slice(&bytes).map_err(NoiseAssetLoaderError::Json)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["noise.json"]
    }
}

#[derive(Debug)]
pub enum NoiseAssetLoaderError {
    Io(std::io::Error),
    Ron(ron::error::SpannedError),
    Json(serde_json::Error),
}

impl std::fmt::Display for NoiseAssetLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Ron(err) => write!(f, "RON error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
        }
    }
}

impl std::error::Error for NoiseAssetLoaderError {}
