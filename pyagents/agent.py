import numpy as np

import asyncio
import websockets

import json

async def read_input():
    # Run blocking input() in a background thread
    return await asyncio.to_thread(input, "Enter UserInput command: ")

async def Arcive_agent():
    uri = "ws://localhost:9001"
    async with websockets.connect(uri) as websocket:
        while True:
            user_input = await read_input()
            if user_input.lower() in ("exit", "quit"):
                print("Goodbye!")
                break

            # Example: Build {"soul_id":"acsoinnaesoc","block_type":"Mouth","X":1,"Y":1,"dir":"N","power":50}
            # Example: Activate {"soul_id":"acsoinnaesoc","delay":0,"X":1,"Y":1,"power":50}
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


async def agent():
    uri = "ws://localhost:9001"
    credential = None  # Store credential after login

    async with websockets.connect(uri) as websocket:
        # Step 1: Login first
        username = input("Enter username: ")
        soul_id = input("Enter soul ID: ")

        login_msg = {
            "type": "Login",
            "payload": {
                "username": username,
                "soul_id": soul_id
            }
        }
        await websocket.send(json.dumps(login_msg))
        print(f"Sent login message: {login_msg}")

        # Wait for login response to get credential
        response = await websocket.recv()
        credential = response.strip()
        print(f"Logged in, received credential: {credential}")

        # Now loop for other commands, always attaching credential in payload
        while True:
            user_input = await read_input()
            if user_input.lower() in ("exit", "quit"):
                print("Goodbye!")
                break

            try:
                # parse user input command and optional payload JSON
                type_and_payload = user_input.split(" ", 1)
                msg_type = type_and_payload[0]

                if len(type_and_payload) > 1:
                    payload = json.loads(type_and_payload[1])
                else:
                    payload = {}

                # Replace soul_id with credential for all commands except Login
                if msg_type.lower() != "login":
                    payload["soul_id"] = credential

                msg = {
                    "type": msg_type,
                    "payload": payload
                }
                await websocket.send(json.dumps(msg))
                print(f"Sent: {msg}")

            except Exception as e:
                print(f"Error: {e}\nFormat: <Type> <JSON payload>")

asyncio.run(agent())