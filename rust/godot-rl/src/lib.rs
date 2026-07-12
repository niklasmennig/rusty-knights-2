use godot::prelude::*;
use hecs::{DynamicBundle, Entity, World};

use crate::{
    components::Position,
    map::{DiscreteMap, Map, TileType},
};

mod components;
mod map;

type EntityId = u32;

struct GodotRLExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRLExtension {}

struct WorldResources {
    map: DiscreteMap,
}

#[derive(GodotClass)]
#[class(base=Node)]
struct RoguelikeWorld {
    world: World,
    resources: WorldResources,
    base: Base<Node>,
}

#[godot_api]
impl INode for RoguelikeWorld {
    fn init(base: Base<Node>) -> Self {
        Self {
            world: World::new(),
            resources: WorldResources {
                map: DiscreteMap::generate_random(Vector2i::new(30, 30)),
            },
            base,
        }
    }

    fn ready(&mut self) {
        godot_print!("RoguelikeWorld initializing...");

        self.create_entity((
            Position {
                position: Vector2i::new(0, 0),
            },
            true,
        ));

        let map = &self.resources.map;
        let tiles = map
            .get_tiles()
            .filter(|(_, ty)| ty == &TileType::Floor)
            .map(|(tile, _)| tile.cast_float())
            .collect::<PackedVector2Array>();
        self.signals()
            .tiles_type_changed()
            .emit(&tiles, TileType::Floor);
    }
}

#[godot_api]
impl RoguelikeWorld {
    #[signal]
    fn visual_entity_created(entity: EntityId, position: Vector2i);
    #[signal]
    fn visual_entity_moved(entity: EntityId, to: Vector2i);
    #[signal]
    fn tiles_type_changed(tiles: PackedVector2Array, tile_type: TileType);

    #[func]
    fn try_move(&mut self, entity: EntityId, delta: Vector2i) {
        let entity = self.get_entity_from_id(entity);
        let (mut pos, _) = self
            .world
            .query_one_mut::<(&mut Position, &bool)>(entity)
            .unwrap();

        pos.position += delta;
        let target_pos = pos.position;
        self.signals()
            .visual_entity_moved()
            .emit(entity.id(), target_pos);
    }

    fn create_entity(&mut self, components: impl DynamicBundle) -> Entity {
        let with_position = components.has::<Position>();
        let entity = self.world.spawn(components);

        if with_position {
            let pos = self.world.get::<&Position>(entity).unwrap().position;
            self.signals()
                .visual_entity_created()
                .emit(entity.id(), pos);
        }

        entity
    }

    fn get_entity_from_id(&self, entity: EntityId) -> Entity {
        unsafe { self.world.find_entity_from_id(entity) }
    }
}
