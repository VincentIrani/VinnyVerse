import asyncio
import json
import websockets

async def read_input():
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, input, "> ")

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

        async def send_loop():
            while True:
                user_input = await read_input()
                if user_input.lower() in ("exit", "quit"):
                    print("Goodbye!")
                    break

                try:
                    type_and_payload = user_input.split(" ", 1)
                    msg_type = type_and_payload[0]

                    if len(type_and_payload) > 1:
                        payload = json.loads(type_and_payload[1])
                    else:
                        payload = {}

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

        async def receive_loop():
            while True:
                try:
                    msg = await websocket.recv()
                    if isinstance(msg, bytes):
                        try:
                            data = json.loads(msg.decode('utf-8'))
                            print("Received JSON:", data)
                        except json.JSONDecodeError:
                            print("not valid JSON:", msg)
                    else:
                        print("Received non-JSON message:", msg)
                except websockets.ConnectionClosed:
                    print("Server closed the connection.")
                    break

        # Run both loops concurrently
        await asyncio.gather(send_loop(), receive_loop())

# To run the agent
asyncio.run(agent())

# Sample Messages:
# Build {"soul_id":"acsoinnaesoc","block_type":"Tissue","X":0,"Y":-1,"dir":"N","power":50}
# Activate {"soul_id":"acsoinnaesoc","delay": 1, "X":0,"Y":-2,"power":50}