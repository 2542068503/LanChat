import os
from PIL import Image, ImageDraw, ImageFont

os.makedirs('public/presets', exist_ok=True)

icons = [
    ('word.png', (43, 87, 154), 'W'),
    ('excel.png', (33, 115, 70), 'X'),
    ('powerpoint.png', (183, 71, 42), 'P'),
    ('vscode.png', (0, 101, 169), 'V')
]

for filename, color, text in icons:
    img = Image.new('RGBA', (256, 256), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # Draw rounded rectangle
    d.rounded_rectangle([16, 16, 240, 240], radius=32, fill=color)
    # Try to use a default font or just draw text
    try:
        font = ImageFont.truetype("arial.ttf", 150)
    except IOError:
        font = ImageFont.load_default()
    
    # Calculate text position manually to center it
    text_width = d.textlength(text, font=font)
    text_x = (256 - text_width) / 2
    # Estimate text height since textbbox might vary
    text_y = (256 - 150) / 2 - 20
    
    d.text((text_x, text_y), text, fill="white", font=font)
    img.save(f'public/presets/{filename}')

print("Preset icons generated successfully.")
