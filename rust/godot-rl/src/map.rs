use std::rc::Rc;

use godot::{
    classes::{
        Geometry2D,
        class_macros::private::virtuals::Os::{GString, Rect2i, Vector2Axis, Vector2i},
    },
    meta::GodotConvert,
    obj::Singleton,
    prelude::{Export, Var},
};
use rand::{RngExt, rng};

#[derive(Clone, Copy, GodotConvert, Var, Export, Debug, PartialEq, Eq, Hash)]
#[godot(via = GString)]
pub enum TileType {
    Floor,
    Wall,
}

pub trait TileIter {
    fn iter_tiles(&self) -> impl Iterator<Item = Vector2i>;
}

pub struct RectTileIter {
    current: Vector2i,
    rect: Rect2i,
}

impl RectTileIter {
    pub fn from_rect(rect: &Rect2i) -> RectTileIter {
        RectTileIter {
            current: rect.position,
            rect: *rect,
        }
    }
}

impl Iterator for RectTileIter {
    type Item = Vector2i;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rect.contains_point(self.current) {
            let res = Some(self.current);
            if self.current.x + 1 >= self.rect.end().x {
                self.current.y += 1;
                self.current.x = self.rect.position.x;
            } else {
                self.current.x += 1;
            }
            res
        } else {
            None
        }
    }
}

impl TileIter for Rect2i {
    fn iter_tiles(&self) -> impl Iterator<Item = Vector2i> {
        RectTileIter::from_rect(self)
    }
}

pub trait Map {
    fn get_tile_at_position(&self, position: Vector2i) -> Option<TileType>;
    fn get_tiles(&self) -> impl Iterator<Item = (Vector2i, TileType)>;
}

pub struct DiscreteMap {
    size: Vector2i,
    tiles: Vec<Option<TileType>>,
    pub start_tile: Vector2i,
    pub door_locations: Vec<Vector2i>,
}

impl DiscreteMap {
    pub fn generate_random(size: Vector2i, max_rooms: usize) -> DiscreteMap {
        let rect = Rect2i::new(Vector2i::ZERO, size);
        let mut rng = rng();

        let mut tiles = Vec::new();
        tiles.resize_with((size.x * size.y) as usize, || None);

        let min_size = Vector2i::new(5, 5);
        let max_size = Vector2i::new(10, 10);
        let mut rooms = Vec::<Rect2i>::new();
        let mut retries = 0;
        let max_retries = 100;

        while rooms.len() < max_rooms && retries < max_retries {
            let start_x = rng.random_range(2..size.x - min_size.x - 2);
            let start_y = rng.random_range(2..size.y - min_size.y - 2);

            let size_x = rng.random_range(min_size.x..max_size.x);
            let size_y = rng.random_range(min_size.y..max_size.y);

            let room_rect = Rect2i::from_components(start_x, start_y, size_x, size_y);
            if rect.encloses(room_rect) && rooms.iter().all(|r| !r.grow(1).intersects(room_rect)) {
                rooms.push(room_rect);
            } else {
                retries += 1;
                continue;
            }
        }

        for (idx, room) in rooms.iter().enumerate() {
            for tile in room.iter_tiles() {
                tiles[(tile.x + tile.y * rect.size.x) as usize] = Some(TileType::Floor);
            }

            if idx > 0 {
                let last_rect = rooms[idx - 1];

                // start at new room so corridor early exits are do not prevent access
                let mut current = room.center();
                let end = last_rect.center();

                let mut left_start_room = false;

                while !(current == end
                    || (Some(TileType::Floor)
                        == tiles[(current.x + current.y * rect.size.x) as usize]
                        && left_start_room))
                {
                    if !room.contains_point(current) {
                        left_start_room = true
                    }
                    tiles[(current.x + current.y * rect.size.x) as usize] = Some(TileType::Floor);
                    let delta = end - current;
                    let axis = delta.abs().max_axis().unwrap_or(Vector2Axis::X);
                    let step = delta[axis].clamp(-1, 1);
                    let mut stepv = Vector2i::ZERO;
                    stepv[axis] = step;
                    current += stepv;
                }
            }
        }

        let mut wall_tiles = Vec::new();
        for tile in tiles.iter().enumerate().filter_map(|(idx, t)| {
            if let Some(TileType::Floor) = t {
                Some(Vector2i::new(idx as i32 % size.x, idx as i32 / size.x))
            } else {
                None
            }
        }) {
            for offset_tile in [
                Vector2i::new(-1, -1),
                Vector2i::new(0, -1),
                Vector2i::new(1, -1),
                Vector2i::new(-1, 0),
                Vector2i::new(1, 0),
                Vector2i::new(-1, 1),
                Vector2i::new(0, 1),
                Vector2i::new(1, 1),
            ]
            .map(|p| p + tile)
            {
                if offset_tile.x >= 0
                    && offset_tile.x < size.x
                    && offset_tile.y >= 0
                    && offset_tile.y < size.y
                {
                    if tiles[(offset_tile.x + offset_tile.y * rect.size.x) as usize]
                        .is_none_or(|t| t != TileType::Floor)
                    {
                        wall_tiles.push(offset_tile);
                    }
                }
            }
        }

        wall_tiles
            .iter()
            .for_each(|t| tiles[(t.x + t.y * size.x) as usize] = Some(TileType::Wall));

        // calculate possible door locations
        let mut door_locations = Vec::new();
        for room in &rooms {
            // grow room by 1 to include walls
            let room = room.grow(1);
            let edge_positions = (room.position.x..room.end().x)
                .flat_map(|x| {
                    [
                        Vector2i::new(x, room.position.y),
                        Vector2i::new(x, room.end().y - 1),
                    ]
                })
                .chain((room.position.y + 1..room.end().y - 1).flat_map(|y| {
                    [
                        Vector2i::new(room.position.x, y),
                        Vector2i::new(room.end().x - 1, y),
                    ]
                }));

            for pos in edge_positions {
                if let Some(TileType::Floor) = tiles[(pos.x + pos.y * size.x) as usize] {
                    door_locations.push(pos);
                }
            }
        }

        let start_tile = rooms.first().unwrap().center();
        DiscreteMap {
            size,
            tiles,
            start_tile,
            door_locations,
        }
    }
}

impl Map for DiscreteMap {
    fn get_tile_at_position(&self, position: Vector2i) -> Option<TileType> {
        if let Some(to) = self
            .tiles
            .get((position.x + position.y * self.size.x) as usize)
        {
            *to
        } else {
            None
        }
    }

    fn get_tiles(&self) -> impl Iterator<Item = (Vector2i, TileType)> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_some())
            .map(|(idx, t)| {
                (
                    Vector2i::new(idx as i32 % self.size.x, idx as i32 / self.size.x),
                    t.unwrap(),
                )
            })
    }
}
