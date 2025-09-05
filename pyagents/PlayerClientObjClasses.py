import asyncio
import json
import websockets
import pygame
import sys
from enum import Enum


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

        self.length = w

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
        width = max(self.length, txt_surface.get_width() + 10)
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

class InputSelector:
    class Mode(Enum):
        ACTIVATE = 0
        BUILD = 1
    class Tissue(Enum):
        NULL = 0
        TISSUE = 1
        MUSCLE_N = 2; MUSCLE_S = 3; MUSCLE_E = 4; MUSCLE_W = 5
        MOUTH_N = 6; MOUTH_S = 7; MOUTH_E = 8; MOUTH_W = 9
        EYE_D_N = 10; EYE_D_S = 11; EYE_D_E = 12; EYE_D_W = 13; EYE_C = 14
        BUTT_N = 15; BUTT_S = 16; BUTT_E = 17; BUTT_W = 18
        ARMOR = 19
        ANCHOR = 23

    def __init__(self, x, y, default_index=0):
        
        self.mode = InputSelector.Mode.ACTIVATE # Initialize mode to ACTIVATE

        self.x_pos = x
        self.y_pos = y

        self.b1_rect = pygame.Rect(x, y, 75, 20)  # Main box position
        self.b1_pressed = False 
        self.b1_hovered = False

        self.build_tissue_rect = pygame.Rect(x + 100, y, 50, 50)  # Build Tissue box position
        self.build_tissue_hovered = False
        self.build_tissue_pressed = False

        self.x_label = pygame.font.SysFont(None, 18).render("X:", True, BLACK)
        self.x_box = text_input_box(x+200, y+10, 40, 20, font_size=18)
        self.y_label = pygame.font.SysFont(None, 18).render("Y:", True, BLACK)
        self.y_box = text_input_box(x+270, y+10, 40, 20, font_size=18)

        self.power_box = text_input_box(x+300, y+10, 40, 20, font_size=18)

        self.current_tissue = InputSelector.Tissue.NULL # Initialize tissue to NULL

        #Importing first button images
        self.img_b1_A_released = pygame.image.load(r"assets\Buttons\Input_Button\activate_released.png").convert_alpha()
        self.img_b1_A_hovered = pygame.image.load(r"assets\Buttons\Input_Button\activate_hovered.png").convert_alpha()
        self.img_b1_A_pressed = pygame.image.load(r"assets\Buttons\Input_Button\activate_pressed.png").convert_alpha()
        self.img_b1_B_released = pygame.image.load(r"assets\Buttons\Input_Button\build_released.png").convert_alpha()
        self.img_b1_B_hovered = pygame.image.load(r"assets\Buttons\Input_Button\build_hovered.png").convert_alpha()
        self.img_b1_B_pressed = pygame.image.load(r"assets\Buttons\Input_Button\build_pressed.png").convert_alpha()
        
        # Importing Build Tissue Images
        self.img_tis_null = pygame.image.load(r"assets\Buttons\Input_Button\build_pressed.png").convert_alpha()
        self.img_tis_tissue = pygame.image.load(r"assets\Critter_Squares\Tissue\Tissue_0.png").convert_alpha()
        self.img_tis_muscle = pygame.image.load(r"assets\Critter_Squares\Muscle\Muscle_0_d.png").convert_alpha()
        self.img_tis_mouth = pygame.image.load(r"assets\Critter_Squares\Mouth\Mouth_0.png").convert_alpha()
        self.img_tis_eye_c = pygame.image.load(r"assets\Critter_Squares\Eye\Eye_0_c.png").convert_alpha()
        self.img_tis_eye_d = pygame.image.load(r"assets\Critter_Squares\Eye\Eye_0_d.png").convert_alpha()
        self.img_tis_butt = pygame.image.load(r"assets\Critter_Squares\Butt\Butt_0_d.png").convert_alpha()
        self.img_tis_armor = pygame.image.load(r"assets\Critter_Squares\Armor\Armor_0.png").convert_alpha()
        self.img_tis_anchor = pygame.image.load(r"assets\Critter_Squares\Anchor\Anchor_0_d.png").convert_alpha()



    def handle_event(self, event):
        
        mouse_pos = pygame.mouse.get_pos()
        self.b1_hovered = self.b1_rect.collidepoint(mouse_pos)

        if event.type == pygame.MOUSEBUTTONDOWN and event.button == 1:
            if self.b1_hovered:
                self.b1_pressed = True
        elif event.type == pygame.MOUSEBUTTONUP and event.button == 1:
            if self.b1_pressed and self.b1_hovered:
                if self.mode == InputSelector.Mode.ACTIVATE:
                    self.mode = InputSelector.Mode.BUILD
                else:
                    self.mode = InputSelector.Mode.ACTIVATE
            self.b1_pressed = False

        self.x_box.handle_event(event)
        self.y_box.handle_event(event)

        match self.mode:
            case InputSelector.Mode.ACTIVATE:
                pass
            case InputSelector.Mode.BUILD:
                self.build_tissue_hovered = self.build_tissue_rect.collidepoint(mouse_pos)
                if event.type == pygame.MOUSEBUTTONDOWN and event.button == 1:
                    if self.build_tissue_hovered:
                        self.build_tissue_pressed = True
                elif event.type == pygame.MOUSEBUTTONUP and event.button == 1:
                    if self.build_tissue_pressed and self.build_tissue_hovered:
                        # Cycle through tissue types
                        if self.current_tissue.value == len(InputSelector.Tissue) - 2:
                            next_tissue_value = 0
                        else:
                            next_tissue_value = (self.current_tissue.value + 1) % len(InputSelector.Tissue)
                        self.current_tissue = InputSelector.Tissue(next_tissue_value)
                    self.build_tissue_pressed = False

                

    def draw(self, surface):
        match self.mode:
            case InputSelector.Mode.ACTIVATE:
                if self.b1_pressed:
                    img = self.img_b1_A_pressed
                elif self.b1_hovered:
                    img = self.img_b1_A_hovered
                else:
                    img = self.img_b1_A_released
            case InputSelector.Mode.BUILD:
                if self.b1_pressed:
                    img = self.img_b1_B_pressed
                elif self.b1_hovered:
                    img = self.img_b1_B_hovered
                else:
                    img = self.img_b1_B_released

                match self.current_tissue:
                    case InputSelector.Tissue.NULL:
                        tis_img = self.img_tis_null
                    case InputSelector.Tissue.TISSUE:
                        tis_img = self.img_tis_tissue
                    case InputSelector.Tissue.MUSCLE_N:
                        tis_img = pygame.transform.rotate(self.img_tis_muscle, 0)
                    case InputSelector.Tissue.MUSCLE_S:
                        tis_img = pygame.transform.rotate(self.img_tis_muscle, 180)
                    case InputSelector.Tissue.MUSCLE_E:
                        tis_img = pygame.transform.rotate(self.img_tis_muscle, 270)
                    case InputSelector.Tissue.MUSCLE_W:
                        tis_img = pygame.transform.rotate(self.img_tis_muscle, 90)
                    case InputSelector.Tissue.MOUTH_N:
                        tis_img = pygame.transform.rotate(self.img_tis_mouth, 0)
                    case InputSelector.Tissue.MOUTH_S:
                        tis_img = pygame.transform.rotate(self.img_tis_mouth, 180)
                    case InputSelector.Tissue.MOUTH_E:
                        tis_img = pygame.transform.rotate(self.img_tis_mouth, 270)
                    case InputSelector.Tissue.MOUTH_W:
                        tis_img = pygame.transform.rotate(self.img_tis_mouth, 90)
                    case InputSelector.Tissue.EYE_D_N:
                        tis_img = pygame.transform.rotate(self.img_tis_eye_d, 0)
                    case InputSelector.Tissue.EYE_D_S:
                        tis_img = pygame.transform.rotate(self.img_tis_eye_d, 180)
                    case InputSelector.Tissue.EYE_D_E:
                        tis_img = pygame.transform.rotate(self.img_tis_eye_d, 270)
                    case InputSelector.Tissue.EYE_D_W:
                        tis_img = pygame.transform.rotate(self.img_tis_eye_d, 90)
                    case InputSelector.Tissue.EYE_C:
                        tis_img = self.img_tis_eye_c
                    case InputSelector.Tissue.BUTT_N:
                        tis_img = pygame.transform.rotate(self.img_tis_butt, 0)
                    case InputSelector.Tissue.BUTT_S:
                        tis_img = pygame.transform.rotate(self.img_tis_butt, 180)
                    case InputSelector.Tissue.BUTT_E:
                        tis_img = pygame.transform.rotate(self.img_tis_butt, 270)
                    case InputSelector.Tissue.BUTT_W:
                        tis_img = pygame.transform.rotate(self.img_tis_butt, 90)
                    case InputSelector.Tissue.ARMOR:
                        tis_img = self.img_tis_armor
                    case InputSelector.Tissue.ANCHOR:
                        tis_img = self.img_tis_anchor

                # Scale tissue image to fit box
                tis_img = pygame.transform.scale(tis_img, (50, 50))
                surface.blit(tis_img, self.build_tissue_rect.topleft)
        

        surface.blit(self.x_label, (self.x_pos + 160, self.y_pos + 3))
        surface.blit(self.y_label, (self.x_pos + 230, self.y_pos + 3))


        self.x_box.draw(surface)
        self.y_box.draw(surface)

        surface.blit(img, self.b1_rect.topleft)
