extends PanelContainer

var room_info: RoomInfo

@onready var display_name = $HBoxContainer/VBoxContainer/Name
@onready var players = $HBoxContainer/VBoxContainer/Players
@onready var id = $HBoxContainer/VBoxContainer/Id
@onready var region = $HBoxContainer/VBoxContainer/Region

func _ready() -> void:
	if room_info:
		display_name.text = room_info.name
		players.text = str(room_info.players) + "/" + str(room_info.max_players) + " players"
		id.text = "id: " + room_info.id
		region.text = "region: " + room_info.region
