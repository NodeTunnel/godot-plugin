extends Node

const RELEASE_URL = "https://api.github.com/repos/NodeTunnel/godot-plugin/releases/latest"

var http := HTTPRequest.new()
var plugin_version: String

func _init() -> void:
	add_child(http)

func check_update(current: String) -> void:
	plugin_version = current
	var err = http.request(RELEASE_URL)
	if err != OK:
		return
	
	http.request_completed.connect(_handle_res)

func _handle_res(result, response_code, headers, body: PackedByteArray):
	if response_code != 200:
		return
	
	var json = JSON.parse_string(body.get_string_from_utf8())
	if json == null:
		return
	
	var latest: String = json.get("tag_name", "")
	
	if latest:
		var res = _compare(plugin_version, latest)
		
		if res == -1:
			print(plugin_version)
			print("[NodeTunnel] v%s available! (Currently on: v)" % latest, plugin_version)

func _compare(v1: String, v2: String) -> int:
	v1 = v1.split("_", true, 1)[0]
	v2 = v2.split("_", true, 1)[0]
	
	var versions_1 := v1.split(".")
	var versions_2 := v2.split(".")
	
	for i in max(versions_1.size(), versions_2.size()):
		var v1v := int(versions_1[i]) if i < versions_1.size() else 0
		var v2v := int(versions_2[i]) if i < versions_2.size() else 0
		
		if v1v > v2v:
			return 1
		elif v1v < v2v:
			return -1
	
	return 0
