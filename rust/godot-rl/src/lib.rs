use godot::prelude::*;
use hecs::{DynamicBundle, Entity, World};

use crate::{
    components::{BlocksMovement, Door, OpenState, Player, Position},
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
                map: DiscreteMap::generate_random(Vector2i::new(100, 100), 30),
            },
            base,
        }
    }

    fn ready(&mut self) {
        godot_print!("RoguelikeWorld initializing...");

        self.create_entity((
            Position {
                position: self.resources.map.start_tile,
            },
            Player,
        ));

        for pos in self.resources.map.door_locations.clone() {
            self.create_entity((
                Position { position: pos },
                BlocksMovement,
                OpenState(false),
                Door,
            ));
        }

        for tt in [TileType::Floor, TileType::Wall] {
            let tiles = &self
                .resources
                .map
                .get_tiles()
                .filter(|(_, ty)| ty == &tt)
                .map(|(tile, _)| tile.cast_float())
                .collect::<PackedVector2Array>();
            self.signals().tiles_type_changed().emit(tiles, tt);
        }
    }
}

#[godot_api]
impl RoguelikeWorld {
    #[signal]
    fn visual_entity_created(entity: EntityId, position: Vector2i);
    #[signal]
    fn visual_entity_moved(entity: EntityId, to: Vector2i);
    #[signal]
    fn visual_entity_base_representation_changed(entity: EntityId, base_representation: GString);
    #[signal]
    fn tiles_type_changed(tiles: PackedVector2Array, tile_type: TileType);
    #[signal]
    fn player_assigned(entity: EntityId);

    #[func]
    fn try_move(&mut self, entity: EntityId, delta: Vector2i) {
        let entity = self.get_entity_from_id(entity);
        let (mut pos, _) = self
            .world
            .query_one_mut::<(&mut Position, &Player)>(entity)
            .unwrap();

        let target_pos = pos.position + delta;

        // check for map collision
        if Some(TileType::Floor) != self.resources.map.get_tile_at_position(target_pos) {
            return;
        }

        // check for entity collision
        for (pos, _) in self.world.query::<(&Position, &BlocksMovement)>().iter() {
            if pos.position == target_pos {
                return;
            }
        }

        let (mut pos, _) = self
            .world
            .query_one_mut::<(&mut Position, &Player)>(entity)
            .unwrap();

        pos.position = target_pos;

        self.signals()
            .visual_entity_moved()
            .emit(entity.id(), target_pos);
    }

    fn create_entity(&mut self, components: impl DynamicBundle) -> Entity {
        let with_position = components.has::<Position>();
        let is_player = components.has::<Player>();
        let is_door = components.has::<Door>();
        let entity = self.world.spawn(components);

        if with_position {
            let pos = self.world.get::<&Position>(entity).unwrap().position;
            self.signals()
                .visual_entity_created()
                .emit(entity.id(), pos);
        }

        if is_player {
            self.signals().player_assigned().emit(entity.id());
        }

        self.signals()
            .visual_entity_base_representation_changed()
            .emit(
                entity.id(),
                if is_player {
                    "player"
                } else if is_door {
                    "door"
                } else {
                    "unknown"
                },
            );

        entity
    }

    fn get_entity_from_id(&self, entity: EntityId) -> Entity {
        unsafe { self.world.find_entity_from_id(entity) }
    }
}
