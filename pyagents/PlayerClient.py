# This is the start of a VinnyVerse Player Client

import asyncio
import json
import websockets
import pygame
import sys
import uuid
import threading

import PlayerClientObjClasses as ocs

pygame.init()

# --- Display setup ---
def get_desktop_resolution():
    info = pygame.display.Info()
    return info.current_w, info.current_h

def set_display_mode(mode="windowed"):
    global screen
    if mode == "windowed":
        screen = pygame.display.set_mode((800, 600))  # normal window
    elif mode == "borderless":
        w, h = get_desktop_resolution()
        screen = pygame.display.set_mode((w, h), pygame.NOFRAME)  # no borders
    elif mode == "fullscreen":
        w, h = get_desktop_resolution()
        screen = pygame.display.set_mode((w, h), pygame.FULLSCREEN)  # exclusive fullscreen

# Start in windowed mode
set_display_mode("windowed")
pygame.display.set_caption("Pygame Menu with Display Options")

# Colors
WHITE = (255, 255, 255)
BLACK = (0, 0, 0)
GRAY = (200, 200, 200)
BLUE = (0, 120, 215)

# Fonts
font = pygame.font.SysFont(None, 48)

# --- State Machine ---
state = "main_menu"
def set_state(new_state):
    global state
    state = new_state

# --- Buttons for different screens ---
main_menu_buttons = [
    ocs.Button(screen.get_size()[0]/2, screen.get_size()[1]/2, 100, 100, True, (r"assets\Buttons\Play_Button\play_released.png", r"assets\Buttons\Play_Button\play_hovered.png", r"assets\Buttons\Play_Button\play_pressed.png"), lambda: on_login_pressed()),
    ocs.Button(100, 50, 100, 30, True, (r"assets\Buttons\Settings_Button\settings_released.png", r"assets\Buttons\Settings_Button\settings_hovered.png", r"assets\Buttons\Settings_Button\settings_pressed.png"), lambda: set_state("settings")),
    ocs.Button(screen.get_size()[0] - 50, 50, 30, 30, True, (r"assets\Buttons\Quit_Button\quit_released.png", r"assets\Buttons\Quit_Button\quit_hovered.png", r"assets\Buttons\Quit_Button\quit_pressed.png"), lambda: sys.exit()),
]

settings_buttons = [
    ocs.Button(100, 200, 250, 60, True, (r"assets\Buttons\Windowed_Button\windowed_released.png", r"assets\Buttons\Windowed_Button\windowed_hovered.png", r"assets\Buttons\Windowed_Button\windowed_pressed.png"), lambda: set_display_mode("windowed")),
    ocs.Button(100, 300, 250, 60, True, (r"assets\Buttons\Borderless_Button\borderless_released.png", r"assets\Buttons\Borderless_Button\borderless_hovered.png", r"assets\Buttons\Borderless_Button\borderless_pressed.png"), lambda: set_display_mode("borderless")),
    ocs.Button(100, 400, 250, 60, True, (r"assets\Buttons\Fullscreen_Button\fullscreen_released.png", r"assets\Buttons\Fullscreen_Button\fullscreen_hovered.png", r"assets\Buttons\Fullscreen_Button\fullscreen_pressed.png"), lambda: set_display_mode("fullscreen")),
    ocs.Button(screen.get_size()[0] - 50, 50, 30, 30, True, (r"assets\Buttons\Back_Button\back_released.png", r"assets\Buttons\Back_Button\back_hovered.png", r"assets\Buttons\Back_Button\back_pressed.png"), lambda: set_state("main_menu")),
]

gameplay_buttons = [
    ocs.Button(screen.get_size()[0] - 50, 50, 30, 30, True, (r"assets\Buttons\Back_Button\back_released.png", r"assets\Buttons\Back_Button\back_hovered.png", r"assets\Buttons\Back_Button\back_pressed.png"), lambda: set_state("main_menu")),
    ocs.Button(screen.get_size()[0] - 100, 50, 30, 30, True, (r"assets\Buttons\Send_Button\send_released.png", r"assets\Buttons\Send_Button\send_hovered.png", r"assets\Buttons\Send_Button\send_pressed.png"), lambda: send_command()),
]

volume_slider = ocs.Slider(100, screen.get_size()[1] - 100, 300, 20, initial=1.0)

username_box = ocs.text_input_box(screen.get_size()[0]/2, screen.get_size()[1]/2 + 100, 200, 30)
soulid_box = ocs.text_input_box(screen.get_size()[0]/2, screen.get_size()[1]/2 + 150, 200, 30)

input_button = ocs.InputSelector(100, screen.get_size()[1] - 150)

# --- Screen drawing functions ---
def main_menu():
    screen.fill(BLACK)
    title = font.render("Main Menu", True, WHITE)
    screen.blit(title, (screen.get_width()//2 - title.get_width()//2, 100))
    for button in main_menu_buttons:
        button.draw(screen)
    username_box.draw(screen)
    soulid_box.draw(screen)

def settings_menu():
    screen.fill(GRAY)
    title = font.render("Settings", True, BLACK)
    screen.blit(title, (screen.get_width()//2 - title.get_width()//2, 100))
    for button in settings_buttons:
        button.draw(screen)
    volume_slider.draw(screen)

def gameplay():
    screen.fill((0, 100, 200))
    title = font.render("Game Running...", True, WHITE)
    screen.blit(title, (screen.get_width()//2 - title.get_width()//2, 100))
    for button in gameplay_buttons:
        button.draw(screen)
    input_button.draw(screen)

def is_valid_uuid(u):
    try:
        val = uuid.UUID(u)  # attempt to parse
        return True
    except ValueError:
        return False

async def listener(username, soulID):
    session_credential = ""
    uri = "ws://localhost:9001"
    async with websockets.connect(uri) as websocket:
        login_message = {
            "type": "Login",
            "payload": {
                "username": username,
                "soul_id": soulID
            }
        }
        await websocket.send(json.dumps(login_message))
        response = await websocket.recv()

        if is_valid_uuid(response.strip()):
            print(f"Logged in user: {username} with Soul ID: {soulID}")
            session_credential = response.strip()
            set_state("gameplay")
        else:
            print(f"Recieved: {response.strip()}")

        while state == "gameplay":
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
                    pass

def send_command(command):
    command = command_box.get_text()
    if state == "gameplay":
        asyncio.run(websocket.send(json.dumps(command)))

def render_world(data):
    # Implement your world rendering logic here
    pass

def on_login_pressed():
    username = username_box.get_text()
    soulID = soulid_box.get_text()
    # run async login in a background thread
    threading.Thread(target=lambda: asyncio.run(listener(username, soulID))).start()

# --- Main loop ---
running = True
while running:
    
    # Event handling
    for event in pygame.event.get():
        
        #Killing the program if the 
        if event.type == pygame.QUIT:
            running = False
        
        match state:
            case "main_menu":
                for button in main_menu_buttons:
                    button.handle_event(event)
                username_box.handle_event(event)
                soulid_box.handle_event(event)
            case "settings":
                for button in settings_buttons:
                    button.handle_event(event)
                volume_slider.handle_event(event)
            case "gameplay":
                for button in gameplay_buttons:
                    button.handle_event(event)
                input_button.handle_event(event)

    # Draw the current state
    match state:
        case "main_menu":
            main_menu()
        case "settings":
            settings_menu()
        case "gameplay":
            gameplay()

    pygame.display.flip()

pygame.quit()
sys.exit()