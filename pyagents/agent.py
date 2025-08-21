
# Sample Messages:
# Build {"soul_id":"acsoinnaesoc","block_type":"Tissue","X":0,"Y":-1,"dir":"N","power":50}
# Activate {"soul_id":"acsoinnaesoc","delay": 1, "X":0,"Y":-2,"power":50}

import asyncio
import json
import websockets
import pygame

# ---- Pygame setup ----
TILE_SIZE = 32
GRID_WIDTH = 20   # adjust if your world is bigger
GRID_HEIGHT = 15
SCREEN_WIDTH = TILE_SIZE * GRID_WIDTH
SCREEN_HEIGHT = TILE_SIZE * GRID_HEIGHT

pygame.init()
screen = pygame.display.set_mode((SCREEN_WIDTH, SCREEN_HEIGHT))
pygame.display.set_caption("Critter World")
clock = pygame.time.Clock()

# Simple colors for world cells / critters
WHITE = (255, 255, 255)
BLACK = (0, 0, 0)
RED   = (255, 0, 0)
BLUE  = (0, 0, 255)
GREEN = (0, 255, 0)
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
                # ---- Pump pygame events so window stays alive ----
                for event in pygame.event.get():
                    if event.type == pygame.QUIT:
                        print("Window closed.")
                        pygame.quit()
                        return

                try:
                    msg = await asyncio.wait_for(websocket.recv(), timeout=0.05)
                    if isinstance(msg, bytes):
                        try:
                            data = json.loads(msg.decode("utf-8"))
                            print("Received JSON:", data)
                            render_world(data)
                        except json.JSONDecodeError:
                            print("not valid JSON:", msg)
                    else:
                        print("Received non-JSON message:", msg)
                except asyncio.TimeoutError:
                    # just means no new message, keep looping
                    pass

                await asyncio.sleep(0.01)  # small yield for event loop


        # Run both loops concurrently
        await asyncio.gather(send_loop(), receive_loop())

def render_world(squares):
    screen.fill(BLACK)
    for sq in squares:
        x = sq["x"]
        y = sq["y"]
        content = sq["content"]

        rect = pygame.Rect(x * TILE_SIZE, y * TILE_SIZE, TILE_SIZE, TILE_SIZE)

        if "WorldCell" in content:
            val = content["WorldCell"]
            color = (val, val, val)  # grayscale
            pygame.draw.rect(screen, color, rect)

        elif "CritterCell" in content:
            critter = content["CritterCell"]["kind"]
            if critter == "Eyeball":
                pygame.draw.rect(screen, BLUE, rect)
            elif critter == "Muscle":
                pygame.draw.rect(screen, RED, rect)
            else:
                pygame.draw.rect(screen, GREEN, rect)

    pygame.display.flip()

asyncio.run(agent())