use crate::{body::BodyGenome, neural::NeuralGenome};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Genome {
    photosynthetic: bool,
    can_ingest: bool,
    initial_weights: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyGenome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brain: Option<NeuralGenome>,
}

impl Genome {
    pub fn new(
        photosynthetic: bool,
        can_ingest: bool,
        initial_weights: Vec<f64>,
    ) -> Result<Self, &'static str> {
        let genome = Self {
            photosynthetic,
            can_ingest,
            initial_weights,
            body: None,
            brain: None,
        };
        genome.validate()?;
        Ok(genome)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.initial_weights.iter().all(|value| value.is_finite()) {
            return Err("inherited weights must be finite");
        }
        if let Some(body) = &self.body {
            body.validate()?;
            if let Some(circle) = &body.circle {
                if self.photosynthetic != circle.photosynthetic
                    || self.can_ingest != circle.can_ingest
                {
                    return Err("capability flags disagree with circle traits");
                }
            } else if self.photosynthetic
                != body
                    .modules
                    .iter()
                    .any(|m| m.kind == crate::body::ModuleKind::Photosynthetic)
                || self.can_ingest
                    != body
                        .modules
                        .iter()
                        .any(|m| m.kind == crate::body::ModuleKind::Mouth)
            {
                return Err("capability flags disagree with body modules");
            }
        }
        if let Some(brain) = &self.brain {
            let body = self
                .body
                .as_ref()
                .ok_or("neural organism requires a body blueprint")?;
            brain.validate(&body.ports()?)?;
        }
        Ok(())
    }

    pub fn photosynthetic(&self) -> bool {
        self.body.as_ref().map_or(self.photosynthetic, |b| {
            if let Some(circle) = &b.circle {
                return circle.photosynthetic;
            }
            b.modules
                .iter()
                .any(|m| m.kind == crate::body::ModuleKind::Photosynthetic)
        })
    }
    pub fn refresh_capabilities(&mut self) {
        if let Some(body) = &self.body {
            if let Some(circle) = &body.circle {
                self.photosynthetic = circle.photosynthetic;
                self.can_ingest = circle.can_ingest;
                return;
            }
            self.photosynthetic = body
                .modules
                .iter()
                .any(|m| m.kind == crate::body::ModuleKind::Photosynthetic);
            self.can_ingest = body
                .modules
                .iter()
                .any(|m| m.kind == crate::body::ModuleKind::Mouth);
        }
    }
    pub fn can_ingest(&self) -> bool {
        self.body.as_ref().map_or(self.can_ingest, |b| {
            if let Some(circle) = &b.circle {
                return circle.can_ingest;
            }
            b.modules
                .iter()
                .any(|m| m.kind == crate::body::ModuleKind::Mouth)
        })
    }
    pub fn initial_weights(&self) -> &[f64] {
        &self.initial_weights
    }
}
