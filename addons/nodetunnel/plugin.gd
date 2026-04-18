@tool
extends EditorPlugin

const RELEASE_URL: String = "https://api.github.com/repos/NodeTunnel/godot-plugin/releases/latest"

var http: HTTPRequest


func _enter_tree() -> void:
	if not http:
		http = HTTPRequest.new()
		add_child(http)
		http.request_completed.connect(_handle_res)

	http.request(RELEASE_URL)


func _exit_tree() -> void:
	if http: http.queue_free()


func _handle_res(result: int, response_code: int, _headers: PackedStringArray, body: PackedByteArray) -> void:
	if result != HTTPRequest.RESULT_SUCCESS or response_code != HTTPClient.RESPONSE_OK:
		return

	var json: Dictionary = JSON.parse_string(body.get_string_from_utf8())
	if not json: return
	var plugin_version := get_plugin_version()
	var latest: String = json.get("tag_name", "")

	if not latest.is_empty() and _compare(plugin_version, latest) < 0:
		print("[NodeTunnel] v%s available! (Current: v%s)" % [latest, plugin_version])


func _compare(v1: String, v2: String) -> int:
	var a := v1.get_slice("_", 0).split(".")
	var b := v2.get_slice("_", 0).split(".")

	for i in maxi(a.size(), b.size()):
		var n1 := a[i].to_int() if i < a.size() else 0
		var n2 := b[i].to_int() if i < b.size() else 0
		if n1 != n2:
			return -1 if n1 < n2 else 1
	return 0
