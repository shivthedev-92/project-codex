//! Known devices: the built-in manifests plus any in `<home>/manifests`.

use std::fs;
use std::path::Path;

use anyhow::{Result, bail};

use crate::manifest::Manifest;

const BUILTIN: &[(&str, &str)] = &[("washer.toml", include_str!("../manifests/washer.toml"))];

#[derive(Debug, Clone)]
pub struct Registry {
    manifests: Vec<Manifest>,
}

impl Registry {
    pub fn builtin() -> Self {
        let manifests = BUILTIN
            .iter()
            .map(|(name, source)| {
                Manifest::from_toml(source)
                    .unwrap_or_else(|error| panic!("built-in manifest {name} is invalid: {error}"))
            })
            .collect();
        Registry { manifests }
    }

    /// Built-ins plus every `*.toml` in `dir`. A user manifest replaces a
    /// built-in with the same id. Broken files are skipped with a warning so
    /// one typo does not lock you out of every device.
    pub fn load(dir: &Path) -> (Self, Vec<String>) {
        let mut registry = Registry::builtin();
        let mut warnings = Vec::new();
        let Ok(entries) = fs::read_dir(dir) else {
            return (registry, warnings);
        };
        let mut paths: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        paths.sort();
        for path in paths {
            let parsed = fs::read_to_string(&path)
                .map_err(anyhow::Error::from)
                .and_then(|source| Manifest::from_toml(&source));
            match parsed {
                Ok(manifest) => registry.insert(manifest),
                Err(error) => warnings.push(format!("skipped {}: {error:#}", path.display())),
            }
        }
        (registry, warnings)
    }

    fn insert(&mut self, manifest: Manifest) {
        match self
            .manifests
            .iter_mut()
            .find(|known| known.device.id == manifest.device.id)
        {
            Some(known) => *known = manifest,
            None => self.manifests.push(manifest),
        }
    }

    pub fn all(&self) -> &[Manifest] {
        &self.manifests
    }

    pub fn get(&self, id: &str) -> Result<&Manifest> {
        match self
            .manifests
            .iter()
            .find(|manifest| manifest.device.id == id)
        {
            Some(manifest) => Ok(manifest),
            None => {
                let known: Vec<&str> = self
                    .manifests
                    .iter()
                    .map(|m| m.device.id.as_str())
                    .collect();
                bail!(
                    "no device named {id:?}; known devices: {}",
                    known.join(", ")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_manifests_are_valid() {
        let registry = Registry::builtin();
        assert!(registry.get("washer").is_ok());
        assert!(registry.get("toaster").is_err());
    }

    #[test]
    fn user_manifests_add_override_and_warn() {
        let dir = tempfile::tempdir().unwrap();
        let plug = r#"
            schema = "spellbook.manifest/0.1"
            [device]
            id = "plug"
            name = "Smart plug"
            kind = "power.smart_plug"
            [transport]
            kind = "mqtt"
            target = "tele/plug/SENSOR"
        "#;
        fs::write(dir.path().join("plug.toml"), plug).unwrap();
        fs::write(
            dir.path().join("washer.toml"),
            plug.replace("\"plug\"", "\"washer\""),
        )
        .unwrap();
        fs::write(dir.path().join("broken.toml"), "schema = 1").unwrap();

        let (registry, warnings) = Registry::load(dir.path());
        assert_eq!(registry.get("plug").unwrap().transport.kind, "mqtt");
        assert_eq!(registry.get("washer").unwrap().transport.kind, "mqtt");
        assert_eq!(registry.all().len(), 2);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("broken.toml"));
    }
}
