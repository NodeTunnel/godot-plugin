extends Node

const DEFAULT_DB = "http://127.0.0.1:8090"

var http = HTTPRequest.new()

func _ready() -> void:
	add_child(http)

func fetch_rooms(app_id: String, db_url = DEFAULT_DB) -> Array[RoomInfo]:
	var http = HTTPRequest.new()
	add_child(http)

	var headers = ["X-App-Id: " + app_id]
	http.request(db_url + "/api/collections/rooms/records", headers)

	var response = await http.request_completed
	http.queue_free()

	var body = response[3]
	var json = JSON.parse_string(body.get_string_from_utf8())

	var rooms: Array[RoomInfo] = []
	for item in json.get("items", []):
		rooms.append(RoomInfo.from_json(item))
	return rooms
