import sys
import subprocess
import math

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("[LOG] Pillow no esta instalado. Instalando de forma automatizada...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "Pillow"])
    from PIL import Image, ImageDraw

def draw_hexagon(draw, center, r, outline_color, width):
    cx, cy = center
    points = []
    for i in range(6):
        angle = math.radians(i * 60 - 30)
        x = cx + r * math.cos(angle)
        y = cy + r * math.sin(angle)
        points.append((x, y))
    draw.polygon(points, outline=outline_color, width=width)

size = 512
img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

red_crimson = (255, 0, 60, 255)
white_pure = (255, 255, 255, 255)
shadow_dark = (15, 15, 15, 200)

draw_hexagon(draw, (256, 256), 220, red_crimson, width=18)

draw.polygon([(150, 140), (150, 372), (346, 256)], fill=shadow_dark)

draw.polygon([(140, 130), (140, 362), (336, 246)], fill=red_crimson)

draw.polygon([(240, 170), (240, 342), (336, 256)], fill=white_pure)

img.save("app-icon.png", "PNG")
print("[LOG SUCCESS] Imagen 'app-icon.png' generada con exito en 512x512 (Transparente).")
