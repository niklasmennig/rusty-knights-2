use std::rc::Rc;

use godot::{
    classes::class_macros::private::virtuals::Os::{GString, Rect2i, Vector2i},
    meta::GodotConvert,
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
    fn get_tiles(&self) -> impl Iterator<Item = (Vector2i, TileType)>;
}

pub struct DiscreteMap {
    size: Vector2i,
    tiles: Vec<Option<TileType>>,
}

impl DiscreteMap {
    pub fn generate_random(size: Vector2i) -> DiscreteMap {
        let rect = Rect2i::new(Vector2i::ZERO, size);
        let mut rng = rng();

        let mut tiles = Vec::new();
        tiles.resize_with((size.x * size.y) as usize, || None);

        let min_size = Vector2i::new(5, 5);
        let max_size = Vector2i::new(10, 10);
        let mut rooms = Vec::<Rect2i>::new();
        let mut retries = 0;
        let max_retries = 100;

        while rooms.len() < 10 && retries < max_retries {
            let start_x = rng.random_range(1..size.x - min_size.x - 1);
            let start_y = rng.random_range(1..size.y - min_size.y - 1);

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
                let (min_x, max_x) = (
                    room.center().x.min(last_rect.center().x),
                    room.center().x.max(last_rect.center().x),
                );
                let (min_y, max_y) = (
                    room.center().y.min(last_rect.center().y),
                    room.center().y.max(last_rect.center().y),
                );

                for x in min_x..=max_x {
                    tiles[(x + min_y * rect.size.x) as usize] = Some(TileType::Floor)
                }
                for y in min_y..=max_y {
                    tiles[(min_x + y * rect.size.x) as usize] = Some(TileType::Floor)
                }
            }
        }

        DiscreteMap { size, tiles }
    }
}

impl Map for DiscreteMap {
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
