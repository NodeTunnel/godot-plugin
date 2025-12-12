class_name RoomInfo

var id: String
var app_id: String
var name: String
var region: String
var players: int
var max_players: int
var created: String
var updated: String

static func from_json(d: Dictionary) -> RoomInfo:
	var r = RoomInfo.new()
	r.id = d.get("id", "")
	r.app_id = d.get("app_id", "")
	r.name = d.get("name", "")
	r.region = d.get("region", "")
	r.players = int(d.get("players", 0))
	r.max_players = int(d.get("max_players", -1))
	r.created = d.get("created", "")
	r.updated = d.get("updated", "")
	return r
