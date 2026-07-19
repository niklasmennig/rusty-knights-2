class_name VisualEntity
extends Node2D

func set_texture(texture: Texture2D):
    %Sprite2D.texture = texture

func set_color(color: Color):
    %Sprite2D.modulate = color
