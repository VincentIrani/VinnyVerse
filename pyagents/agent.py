import numpy as np

import asyncio
import websockets

import json

async def read_input():
    # Run blocking input() in a background thread
    return await asyncio.to_thread(input, "Enter UserInput command: ")

async def agent_archive():
    uri = "ws://localhost:9001"
    async with websockets.connect(uri) as websocket:

        while True:
            user_input = await read_input()
            if user_input.lower() in ("exit", "quit"):
                print("Goodbye!")
                break
            elif user_input.strip() == "a":
                msg = {
                    "soul_id": "acsoinnaesoc",
                    "type": "Activate",
                    "payload": {
                        "X": 0,
                        "Y": 0,
                        "power": 100
                    }
                }
                await websocket.send(json.dumps(msg))
                continue
            elif user_input.strip() == "b":
                msg = {
                    "type": "Build",
                    "payload": {
                        "soul_id": "acsoinnaesoc",
                        "block_type": "Mouth",
                        "X": 1,
                        "Y": 1,
                        "dir": "N",
                        "power": 50
                    }
                }
                await websocket.send(json.dumps(msg))
                continue

async def agent():
    uri = "ws://localhost:9001"
    async with websockets.connect(uri) as websocket:
        while True:
            user_input = await read_input()
            if user_input.lower() in ("exit", "quit"):
                print("Goodbye!")
                break

            # Example: Build {"soul_id":"acsoinnaesoc","block_type":"Mouth","X":1,"Y":1,"dir":"N","power":50}
            try:
                type_and_payload = user_input.split(" ", 1)
                msg_type = type_and_payload[0]
                payload = json.loads(type_and_payload[1]) if len(type_and_payload) > 1 else {}

                msg = {
                    "type": msg_type,
                    "payload": payload
                }
                await websocket.send(json.dumps(msg))
                print(f"Sent: {msg}")
            except Exception as e:
                print(f"Error: {e}\nFormat: <Type> <JSON payload>")


asyncio.run(agent())