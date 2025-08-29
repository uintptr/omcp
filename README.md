[![rust](https://github.com/uintptr/omcp/actions/workflows/rust.yml/badge.svg)](https://github.com/uintptr/omcp/actions/workflows/rust.yml)

# omcp

oxidized mcp

## Dumper

List the tools from the MCP server

```
cargo run --example omcpcli dump-tools --server http://localhost:8123/mcp_server/sse --bearer $TOKEN
```

```json
{
    "jsonrpc": "2.0",
    "id": 2,
    "result": {
        "tools": [
            {
                "description": "Turns on/opens/presses a device or entity. For locks, this performs a 'lock' action. Use for requests like 'turn on', 'activate', 'enable', or 'lock'.",
                "inputSchema": {
                    "properties": {
                        "area": {
                            "type": "string"
                        },
                        "device_class": {
                            "items": {
                                "enum": [
                                    "water",
                                    "gas",
                                    "identify",
                                    "restart",
                                    "update",
                                    "tv",
                                    "speaker",
                                    "receiver",
                                    "awning",
                                    "blind",
                                    "curtain",
                                    "damper",
                                    "door",
                                    "garage",
                                    "gate",
                                    "shade",
                                    "shutter",
                                    "window",
                                    "outlet",
                                    "switch"
                                ],
                                "type": "string"
                            },
                            "type": "array"
                        },
                        "domain": {
                            "items": {
                                "type": "string"
                            },
                            "type": "array"
                        },
                        "floor": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "type": "object"
                },
                "name": "HassTurnOn"
            },
            {
                "description": "Turns off/closes a device or entity. For locks, this performs an 'unlock' action. Use for requests like 'turn off', 'deactivate', 'disable', or 'unlock'.",
                "inputSchema": {
                    "properties": {
                        "area": {
                            "type": "string"
                        },
                        "device_class": {
                            "items": {
                                "enum": [
                                    "water",
                                    "gas",
                                    "identify",
                                    "restart",
                                    "update",
                                    "tv",
                                    "speaker",
                                    "receiver",
                                    "awning",
                                    "blind",
                                    "curtain",
                                    "damper",
                                    "door",
                                    "garage",
                                    "gate",
                                    "shade",
                                    "shutter",
                                    "window",
                                    "outlet",
                                    "switch"
                                ],
                                "type": "string"
                            },
                            "type": "array"
                        },
                        "domain": {
                            "items": {
                                "type": "string"
                            },
                            "type": "array"
                        },
                        "floor": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "type": "object"
                },
                "name": "HassTurnOff"
            },
            {
                "description": "Cancels all timers",
                "inputSchema": {
                    "properties": {
                        "area": {
                            "type": "string"
                        }
                    },
                    "type": "object"
                },
                "name": "HassCancelAllTimers"
            },
            {
                "description": "Add item to a todo list",
                "inputSchema": {
                    "properties": {
                        "item": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "type": "object"
                },
                "name": "HassListAddItem"
            },
            {
                "description": "Complete item on a todo list",
                "inputSchema": {
                    "properties": {
                        "item": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "type": "object"
                },
                "name": "HassListCompleteItem"
            },
            {
                "description": "Query a to-do list to find out what items are on it. Use this to answer questions like 'What's on my task list?' or 'Read my grocery list'. Filters items by status (needs_action, completed, all).",
                "inputSchema": {
                    "properties": {
                        "status": {
                            "default": "needs_action",
                            "description": "Filter returned items by status, by default returns incomplete items",
                            "enum": ["needs_action", "completed", "all"],
                            "type": "string"
                        },
                        "todo_list": {
                            "enum": ["Shopping List"],
                            "type": "string"
                        }
                    },
                    "type": "object"
                },
                "name": "todo_get_items"
            },
            {
                "description": "Provides real-time information about the CURRENT state, value, or mode of devices, sensors, entities, or areas. Use this tool for: 1. Answering questions about current conditions (e.g., 'Is the light on?'). 2. As the first step in conditional actions (e.g., 'If the weather is rainy, turn off sprinklers' requires checking the weather first).",
                "inputSchema": {
                    "properties": {},
                    "type": "object"
                },
                "name": "GetLiveContext"
            }
        ]
    }
}
```
