use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};
use world_genetics::genome::Genome;

/// Immutable DNA shared by transactional world forks.
#[derive(Clone, Debug)]
pub struct InheritedGenome {
    genome: Arc<Genome>,
    validation: Arc<OnceLock<std::result::Result<(), &'static str>>>,
}

impl From<Genome> for InheritedGenome {
    fn from(genome: Genome) -> Self {
        Self {
            genome: Arc::new(genome),
            validation: Arc::new(OnceLock::new()),
        }
    }
}
impl Deref for InheritedGenome {
    type Target = Genome;
    fn deref(&self) -> &Genome {
        &self.genome
    }
}
impl PartialEq for InheritedGenome {
    fn eq(&self, other: &Self) -> bool {
        self.genome == other.genome
    }
}
impl InheritedGenome {
    pub fn validate(&self) -> std::result::Result<(), &'static str> {
        *self.validation.get_or_init(|| self.genome.validate())
    }
    pub fn blueprint(&self) -> Genome {
        self.genome.as_ref().clone()
    }
}
impl Serialize for InheritedGenome {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.genome.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for InheritedGenome {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Ok(Genome::deserialize(deserializer)?.into())
    }
}
