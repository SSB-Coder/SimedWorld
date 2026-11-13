use crate::genome::Genome;

pub fn inherit(parent: &Genome) -> Genome {
    parent.clone()
}
