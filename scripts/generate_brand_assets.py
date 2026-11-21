import os
from PIL import Image, ImageDraw, ImageFilter

def create_brand_icon():
    # Render at high resolution (1024x1024) with supersampling, then downscale for crisp anti-aliasing
    scale = 4
    size = 256 * scale # 1024x1024
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)

    # Colors from Design System Token
    # bg_canvas = (23, 27, 32)
    # bg_surface = (34, 40, 48)
    # border_default = (57, 66, 76)
    # text_muted = (127, 138, 149)
    # accent_lime = (197, 230, 106)
    # status_success = (85, 190, 145)

    # Squircle Container
    padding = 10 * scale
    radius = 56 * scale
    draw.rounded_rectangle(
        [padding, padding, size - padding, size - padding],
        radius=radius,
        fill=(23, 27, 32, 255),
        outline=(57, 66, 76, 255),
        width=int(4 * scale)
    )

    # Back Device (PC / Host display)
    back_x1 = 44 * scale
    back_y1 = 70 * scale
    back_w = 114 * scale
    back_h = 146 * scale
    draw.rounded_rectangle(
        [back_x1, back_y1, back_x1 + back_w, back_y1 + back_h],
        radius=int(18 * scale),
        fill=(26, 32, 38, 255),
        outline=(127, 138, 149, 140),
        width=int(11 * scale)
    )

    # Back device top speaker/indicator
    draw.line(
        [(back_x1 + 35 * scale, back_y1 + 18 * scale), (back_x1 + 65 * scale, back_y1 + 18 * scale)],
        fill=(127, 138, 149, 100),
        width=int(6 * scale)
    )

    # Front Device (Offset Android device)
    front_x1 = 104 * scale
    front_y1 = 40 * scale
    front_w = 112 * scale
    front_h = 148 * scale
    draw.rounded_rectangle(
        [front_x1, front_y1, front_x1 + front_w, front_y1 + front_h],
        radius=int(18 * scale),
        fill=(34, 40, 48, 255),
        outline=(197, 230, 106, 255),
        width=int(12 * scale)
    )

    # Front device top speaker/notch
    draw.line(
        [(front_x1 + 34 * scale, front_y1 + 18 * scale), (front_x1 + 78 * scale, front_y1 + 18 * scale)],
        fill=(197, 230, 106, 200),
        width=int(7 * scale)
    )

    # Bridge Node (Connection Point)
    node_cx = 178 * scale
    node_cy = 130 * scale
    r_outer = 18 * scale
    r_inner = 8 * scale

    # Outer glow
    draw.ellipse(
        [node_cx - r_outer, node_cy - r_outer, node_cx + r_outer, node_cy + r_outer],
        fill=(197, 230, 106, 255)
    )
    # Inner cutout
    draw.ellipse(
        [node_cx - r_inner, node_cy - r_inner, node_cx + r_inner, node_cy + r_inner],
        fill=(23, 27, 32, 255)
    )

    # Status LED dot (ADB Active Connected green)
    led_cx = 66 * scale
    led_cy = 194 * scale
    led_r = 7 * scale
    draw.ellipse(
        [led_cx - led_r, led_cy - led_r, led_cx + led_r, led_cy + led_r],
        fill=(85, 190, 145, 255)
    )

    # Resize with high quality antialiasing
    final_512 = im.resize((512, 512), Image.Resampling.LANCZOS)
    final_256 = im.resize((256, 256), Image.Resampling.LANCZOS)
    final_64 = im.resize((64, 64), Image.Resampling.LANCZOS)
    final_32 = im.resize((32, 32), Image.Resampling.LANCZOS)
    final_16 = im.resize((16, 16), Image.Resampling.LANCZOS)

    assets_dir = r"d:\CE\scrdeck\assets"
    os.makedirs(assets_dir, exist_ok=True)
    
    logo_png_path = os.path.join(assets_dir, "logo.png")
    final_512.save(logo_png_path, "PNG")
    print(f"Saved {logo_png_path}")

    # Also save to src-tauri/icons
    tauri_icons_dir = r"d:\CE\scrdeck\src-tauri\icons"
    final_256.save(os.path.join(tauri_icons_dir, "icon.png"), "PNG")
    
    # Generate multi-resolution .ico
    ico_path = os.path.join(tauri_icons_dir, "icon.ico")
    final_256.save(ico_path, format="ICO", sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print(f"Saved {ico_path} and tauri icon.png")

if __name__ == "__main__":
    create_brand_icon()
