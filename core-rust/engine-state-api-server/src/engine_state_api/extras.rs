use crate::prelude::*;

use super::*;

/// A lister of entities.
pub struct EngineEntityLister<'s, S> {
    database: &'s S,
}

/// Basic information about an entity (i.e. read from a DB index, for listing purposes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySummary {
    pub node_id: NodeId,
    pub creation_id: CreationId,
    pub blueprint_id: Option<BlueprintId>, // only present for Object entities
}

impl<'s, S: EntityListingIndex> EngineEntityLister<'s, S> {
    /// Creates an instance reading from the given database.
    pub fn new(database: &'s S) -> Self {
        Self { database }
    }

    /// Returns an iterator of entities having one of the given [`EntityType`]s, starting from the
    /// given [`CreationId`] (or its successor, if it does not exist), in the [`CreationId`]'s
    /// natural order (ascending).
    pub fn iter_created_entities(
        &self,
        entity_types: impl Iterator<Item = EntityType>,
        from_creation_id: Option<&CreationId>,
    ) -> Result<impl Iterator<Item = EntitySummary> + 's, EngineStateBrowsingError> {
        Ok(entity_types
            .map(|entity_type| {
                self.database
                    .get_created_entity_iter(entity_type, from_creation_id)
            })
            .kmerge_by(|(a_creation_id, _), (b_creation_id, _)| a_creation_id < b_creation_id)
            .map(Self::to_entity_summary))
    }

    /// Returns an iterator of entities having the given [`BlueprintId`], starting from the given
    /// [`CreationId`] (or its successor, if it does not exist), in the [`CreationId`]'s natural
    /// order (ascending).
    pub fn iter_blueprint_entities(
        &self,
        blueprint_id: &BlueprintId,
        from_creation_id: Option<&CreationId>,
    ) -> Result<impl Iterator<Item = EntitySummary> + 's, EngineStateBrowsingError> {
        Ok(self
            .database
            .get_blueprint_entity_iter(blueprint_id, from_creation_id)
            .map(Self::to_entity_summary))
    }

    /// Converts a database index entry into an [`EntitySummary`].
    fn to_entity_summary(db_entry: (CreationId, EntityBlueprintId)) -> EntitySummary {
        let (creation_id, entity_blueprint_id) = db_entry;
        let EntityBlueprintIdV1 {
            node_id,
            blueprint_id,
        } = entity_blueprint_id;
        EntitySummary {
            node_id,
            creation_id,
            blueprint_id,
        }
    }
}
