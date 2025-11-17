class_name RelayDiscovery
extends RefCounted

var registry_url: String
var _http_requests: Array[HTTPRequest] = []

func _init(url: String):
	registry_url = url.trim_suffix("/")

# Find the best relay by latency
func find_best_relay() -> Dictionary:
	var relays = await _fetch_relays()
	if relays.is_empty():
		return {}
	
	var best_relay = {}
	var best_latency = INF
	
	for relay in relays:
		var latency = await _ping_relay(relay.health_url)
		if latency < best_latency:
			best_latency = latency
			best_relay = relay
	
	return best_relay

# Find which relay hosts a specific room
func find_room(room_id: String) -> Dictionary:
	var http = HTTPRequest.new()
	Engine.get_main_loop().root.add_child(http)
	
	var url = "%s/api/collections/rooms/records?filter=(room_id='%s')" % [registry_url, room_id]
	var err = http.request(url)
	if err != OK:
		http.queue_free()
		return {}
	
	var result = await http.request_completed
	http.queue_free()
	
	var response_code = result[1]
	var body = result[3]
	
	if response_code != 200:
		return {}
	
	var json = JSON.parse_string(body.get_string_from_utf8())
	if json == null or not json.has("items") or json.items.is_empty():
		return {}
	
	var room = json.items[0]
	
	# Fetch the relay details
	if room.has("expand") and room.expand.has("relay"):
		return room.expand.relay
	
	# If not expanded, fetch relay separately
	if room.has("relay"):
		return await _fetch_relay_by_id(room.relay)
	
	return {}

func _fetch_relays() -> Array:
	var http = HTTPRequest.new()
	Engine.get_main_loop().root.add_child(http)
	
	var url = "%s/api/collections/relays/records" % registry_url
	var err = http.request(url)
	if err != OK:
		http.queue_free()
		return []
	
	var result = await http.request_completed
	http.queue_free()
	
	var response_code = result[1]
	var body = result[3]
	
	if response_code != 200:
		return []
	
	var json = JSON.parse_string(body.get_string_from_utf8())
	if json == null or not json.has("items"):
		return []
	
	return json.items

func _fetch_relay_by_id(relay_id: String) -> Dictionary:
	var http = HTTPRequest.new()
	Engine.get_main_loop().root.add_child(http)
	
	var url = "%s/api/collections/relays/records/%s" % [registry_url, relay_id]
	var err = http.request(url)
	if err != OK:
		http.queue_free()
		return {}
	
	var result = await http.request_completed
	http.queue_free()
	
	var response_code = result[1]
	var body = result[3]
	
	if response_code != 200:
		return {}
	
	return JSON.parse_string(body.get_string_from_utf8())

func _ping_relay(health_url: String) -> float:
	var http = HTTPRequest.new()
	Engine.get_main_loop().root.add_child(http)
	
	var start_time = Time.get_ticks_msec()
	var err = http.request(health_url)
	if err != OK:
		http.queue_free()
		return INF
	
	var result = await http.request_completed
	http.queue_free()
	
	var response_code = result[1]
	if response_code != 200:
		return INF
	
	return Time.get_ticks_msec() - start_time
