extends Sprite2D

func _enter_tree() -> void:
	set_multiplayer_authority(name.to_int())

func _process(delta: float) -> void:
	if !is_multiplayer_authority():
		return
	
	if Input.is_action_pressed("ui_right"):
		global_position.x += 300 * delta
