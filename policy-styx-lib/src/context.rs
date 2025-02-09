use std::collections::HashMap;

use uuid::Uuid;

use crate::dataset::PcdDataset;

pub struct PcdContext {
    /// The PCD dataset registry.
    pub(crate) data_registry: HashMap<Uuid, PcdDataset>,
}
