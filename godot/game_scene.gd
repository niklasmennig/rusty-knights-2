extends Node

var visual_entity_scene = preload("res://visual_entity.tscn")

var visual_entities : Dictionary[int, VisualEntity]

var player_entity = -1

@export var player_camera : Camera2D

func _input(event: InputEvent) -> void:
    var dir = Vector2i.ZERO
    if event.is_action_pressed("up", true):
        dir = Vector2i.UP
    elif event.is_action_pressed("down", true):
        dir = Vector2i.DOWN
    elif event.is_action_pressed("left", true):
        dir = Vector2i.LEFT
    elif event.is_action_pressed("right", true):
        dir = Vector2i.RIGHT
        
    if dir != Vector2i.ZERO:
        %RoguelikeWorld.try_move(player_entity, dir)

func _grid_to_world(position: Vector2i) -> Vector2:
    return %TileMap.to_global(%TileMap.map_to_local(position))
    

func _on_roguelike_world_visual_entity_created(entity: int, position: Vector2i) -> void:
    print("created visual entity {0} at {1}".format([entity, position]))
    var e = visual_entity_scene.instantiate() as VisualEntity
    e.position = _grid_to_world(position)
    add_child.call_deferred(e)
    visual_entities[entity] = e

func _on_roguelike_world_visual_entity_moved(entity: int, to: Vector2i) -> void:
    print("entity {0} moved to {1} ".format([entity, to]))
    var e = visual_entities[entity]
    e.position = _grid_to_world(to)


func _on_roguelike_world_tiles_type_changed(tiles: PackedVector2Array, tile_type: String) -> void:
    var tile = Vector2i(0,0)
    if tile_type == "Floor":
        tile = Vector2i(1,0)
    elif tile_type == "Wall":
        tile = Vector2i(4,18)
    for t in tiles:
        %TileMap.set_cell(t, 0, tile)


func _on_roguelike_world_player_assigned(entity: int) -> void:
    print("player entity assigned")
    player_entity = entity
    player_camera.reparent(visual_entities[player_entity])
    player_camera.call_deferred("set_position", Vector2.ZERO)


func _on_roguelike_world_visual_entity_base_representation_changed(entity: int, base_representation: String) -> void:
    var e = visual_entities[entity]
    match base_representation:
        "player":
            e.set_texture(preload("res://textures/entities/player.tres"))
        "door":
            e.set_texture(preload("res://textures/entities/door.tres"))
        _:
            e.set_texture(preload("res://textures/entities/unknown.tres"))
