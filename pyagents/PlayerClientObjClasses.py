import asyncio
import json
import websockets
import pygame
import sys

# Colors
WHITE = (255, 255, 255)
BLACK = (0, 0, 0)
GRAY = (200, 200, 200)
BLUE = (0, 120, 215)

# --- Button Class ---
class Button:
    def __init__(self, x, y, w, h, center, image_paths, callback):
        """
        image_paths: tuple/list of 3 strings (paths or URLs):
            (released, hovered, pressed)
        callback: function to call when button is clicked
        """
        if center:
            x = x - w/2
            y = y - h/2
        self.rect = pygame.Rect(x, y, w, h)
        self.callback = callback
        self.pressed = False
        self.hovered = False

        # Load images from paths
        self.images = []
        for path in image_paths:
            # If path is a URL, you would need requests or urllib to fetch it
            # For local paths:
            img = pygame.image.load(path).convert_alpha()
            # Scale image to button size
            img = pygame.transform.scale(img, (w, h))
            self.images.append(img)

    def draw(self, surface):
        # Choose image based on state
        if self.pressed:
            img = self.images[2]  # pressed
        elif self.hovered:
            img = self.images[1]  # hovered/semi-pressed
        else:
            img = self.images[0]  # released
        surface.blit(img, self.rect.topleft)

    def handle_event(self, event):
        mouse_pos = pygame.mouse.get_pos()
        self.hovered = self.rect.collidepoint(mouse_pos)

        if event.type == pygame.MOUSEBUTTONDOWN and event.button == 1:
            if self.hovered:
                self.pressed = True
        elif event.type == pygame.MOUSEBUTTONUP and event.button == 1:
            if self.pressed and self.hovered:
                self.callback()
            self.pressed = False

# --- Slider Class ---
class Slider:
    def __init__(self, x, y, w, h, initial=1.0, center=False):
        if center:
            x = x - w/2
            y = y - h/2
        self.rect = pygame.Rect(x, y, w, h)
        self.knob_rect = pygame.Rect(x + int(initial * w) - 10, y - 5, 20, h + 10)
        self.dragging = False
        self.value = initial  # 0.0 to 1.0
        pygame.mixer.music.set_volume(self.value)

    def draw(self, surface):
        # Track
        pygame.draw.rect(surface, GRAY, self.rect)
        # Knob
        pygame.draw.rect(surface, BLUE, self.knob_rect)
        # Label
        label = font.render(f"Volume: {int(self.value*100)}%", True, BLACK)
        surface.blit(label, (self.rect.x, self.rect.y - 50))

    def handle_event(self, event):
        if event.type == pygame.MOUSEBUTTONDOWN and event.button == 1:
            if self.knob_rect.collidepoint(event.pos):
                self.dragging = True
        elif event.type == pygame.MOUSEBUTTONUP and event.button == 1:
            self.dragging = False
        elif event.type == pygame.MOUSEMOTION and self.dragging:
            # Move knob within slider bounds
            new_x = max(self.rect.x, min(event.pos[0], self.rect.x + self.rect.w))
            self.knob_rect.x = new_x - self.knob_rect.w // 2
            # Update value
            self.value = (new_x - self.rect.x) / self.rect.w
            pygame.mixer.music.set_volume(self.value)

class text_input_box:
    def __init__(self, x, y, w, h, font_size=32):
        self.rect = pygame.Rect(x, y, w, h)
        self.rect.center = (x, y)
        self.color_inactive = pygame.Color('lightskyblue3')
        self.color_active = pygame.Color('dodgerblue2')
        self.color = self.color_inactive
        self.text = ''
        self.font = pygame.font.Font(None, font_size)
        self.active = False

    def handle_event(self, event):
        if event.type == pygame.MOUSEBUTTONDOWN:
            # Toggle active if clicked inside
            if self.rect.collidepoint(event.pos):
                self.active = not self.active
            else:
                self.active = False
            self.color = self.color_active if self.active else self.color_inactive

        if event.type == pygame.KEYDOWN and self.active:
            if event.key == pygame.K_BACKSPACE:
                self.text = self.text[:-1]
            else:
                self.text += event.unicode

    def draw(self, surface):
        txt_surface = self.font.render(self.text, True, self.color)
        width = max(200, txt_surface.get_width() + 10)
        self.rect.w = width
        surface.blit(txt_surface, (self.rect.x+5, self.rect.y+5))
        pygame.draw.rect(surface, self.color, self.rect, 2)

        
    def get_text(self):
        """Return the current contents of the text box."""
        return self.text

class DropUp:
    def __init__(self, x, y, w, h, default_index=0):
        self.rect = pygame.Rect(x, y, w, h)  # Main box position (bottom)
        self.font = pygame.font.SysFont(None, 18)
        self.options = options
        self.selected_index = default_index
        self.expanded = False
        self.color_inactive = pygame.Color('lightskyblue3')
        self.color_active = pygame.Color('dodgerblue2')
        self.option_height = h
        self.color = self.color_inactive

    def handle_event(self, event):
        if event.type == pygame.MOUSEBUTTONDOWN:
            if self.rect.collidepoint(event.pos):
                self.expanded = not self.expanded
            elif self.expanded:
                for i, option in enumerate(self.options):
                    # Options go upward
                    option_rect = pygame.Rect(
                        self.rect.x,
                        self.rect.y - (i+1)*self.option_height,
                        self.rect.w,
                        self.option_height
                    )
                    if option_rect.collidepoint(event.pos):
                        self.selected_index = i
                        self.expanded = False
                        break
                else:
                    self.expanded = False

    def draw(self, surface):
        # Draw main box
        pygame.draw.rect(surface, self.color_active if self.expanded else self.color_inactive, self.rect)
        selected_text = self.font.render(self.options[self.selected_index], True, pygame.Color('black'))
        surface.blit(selected_text, (self.rect.x + 5, self.rect.y + 5))

        # Draw drop-up options
        if self.expanded:
            for i, option in enumerate(self.options):
                option_rect = pygame.Rect(
                    self.rect.x,
                    self.rect.y - (i+1)*self.option_height,
                    self.rect.w,
                    self.option_height
                )
                pygame.draw.rect(surface, self.color_inactive, option_rect)
                option_text = self.font.render(option, True, pygame.Color('black'))
                surface.blit(option_text, (option_rect.x + 5, option_rect.y + 5))

    def get_selected(self):
        return self.options[self.selected_index]

class CommandSelect:
    def __init__(self, x, y, font, default_index=0):
        self.rect = pygame.Rect(x, y, 75, 20)  # Main box position (bottom)
        self.font = pygame.font.SysFont(None, 18)
        self.options = ["Activate", "Build"]
        self.selected_index = default_index
        self.expanded = False
        self.color_inactive = pygame.Color('lightskyblue3')
        self.color_active = pygame.Color('dodgerblue2')
        self.option_height = 20
        self.color = self.color_inactive

    def handle_event(self, event):
        if event.type == pygame.MOUSEBUTTONDOWN:
            if self.rect.collidepoint(event.pos):
                self.expanded = not self.expanded
            elif self.expanded:
                for i, option in enumerate(self.options):
                    # Options go upward
                    option_rect = pygame.Rect(
                        self.rect.x,
                        self.rect.y - (i+1)*self.option_height,
                        self.rect.w,
                        self.option_height
                    )
                    if option_rect.collidepoint(event.pos):
                        self.selected_index = i
                        self.expanded = False
                        break
                else:
                    self.expanded = False
            
            if self.selected_index == 0:
                X_rect = pygame.Rect(self.rect.x + 50, self.rect.y, 30, 20)
                Y_rect = pygame.Rect(self.rect.x + 75, self.rect.y, 30, 20)

    def draw(self, surface):
        # Draw main box
        pygame.draw.rect(surface, self.color_active if self.expanded else self.color_inactive, self.rect)
        selected_text = self.font.render(self.options[self.selected_index], True, pygame.Color('black'))
        surface.blit(selected_text, (self.rect.x + 5, self.rect.y + 5))

        # Draw drop-up options
        if self.expanded:
            for i, option in enumerate(self.options):
                option_rect = pygame.Rect(
                    self.rect.x,
                    self.rect.y - (i+1)*self.option_height,
                    self.rect.w,
                    self.option_height
                )
                pygame.draw.rect(surface, self.color_inactive, option_rect)
                option_text = self.font.render(option, True, pygame.Color('black'))
                surface.blit(option_text, (option_rect.x + 5, option_rect.y + 5))

            if self.selected_index == 0:
                X_rect = pygame.draw.rect(surface, self.color_active, (self.rect.x + 50, self.rect.y, 30, 20))
                Y_rect = pygame.draw.rect(surface, self.color_active, (self.rect.x + 75, self.rect.y, 30, 20))


    def get_selected(self):
        return self.options[self.selected_index]
