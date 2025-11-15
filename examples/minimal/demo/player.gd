extends Sprite2D

func _enter_tree() -> void:
	set_multiplayer_authority(name.to_int())

func _process(delta: float) -> void:
	if Input.is_action_pressed("ui_right"):
		global_position.x += 100 * delta
