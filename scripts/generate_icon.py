#!/usr/bin/env python3
"""Generate CrabChat app icons in PNG, ICO, and ICNS formats.

Produces a simple, recognizable crab silhouette in orange/red tones
on a dark background — suitable for terminal app branding.

Requirements: Python 3, Pillow
Tools used: iconutil (macOS), Pillow for ICO
"""

import math
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw

ASSETS_DIR = Path(__file__).resolve().parent.parent / "assets"

# Colors
BG_COLOR = (30, 30, 40)           # Dark blue-gray background
CRAB_BODY = (220, 80, 40)         # Orange-red crab body
CRAB_DARK = (170, 55, 25)         # Darker shade for depth
CRAB_HIGHLIGHT = (240, 130, 70)   # Highlight
CLAW_COLOR = (200, 65, 35)        # Claw color
EYE_WHITE = (240, 240, 240)       # Eye whites
EYE_PUPIL = (20, 20, 30)          # Eye pupils
CHAT_COLOR = (100, 200, 255)      # Chat bubble accent (light blue)


def draw_ellipse(draw, cx, cy, rx, ry, fill, outline=None):
    """Draw an ellipse centered at (cx, cy) with radii rx, ry."""
    draw.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=fill, outline=outline)


def draw_crab(img, size=512):
    """Draw a stylized crab icon onto the image."""
    draw = ImageDraw.Draw(img)
    s = size  # shorthand

    # --- Background: rounded rectangle feel via full fill + circle ---
    draw.rectangle([0, 0, s, s], fill=BG_COLOR)

    # Subtle circular vignette behind crab
    draw_ellipse(draw, s // 2, s // 2, int(s * 0.42), int(s * 0.42), fill=(40, 40, 55))

    cx, cy = s // 2, int(s * 0.52)  # Center of crab body, slightly below center

    # --- Legs (6 legs, 3 per side) ---
    leg_origins = [
        (0.18, 0.48), (0.14, 0.55), (0.13, 0.63),  # left
    ]
    for lx_frac, ly_frac in leg_origins:
        # Left leg
        x1 = int(cx - s * 0.15)
        y1 = int(s * ly_frac)
        x2 = int(s * lx_frac)
        y2 = int(s * (ly_frac + 0.08))
        draw.line([x1, y1, x2, y2], fill=CRAB_DARK, width=max(s // 50, 3))
        # small foot
        draw.line([x2, y2, int(x2 - s * 0.02), int(y2 + s * 0.03)],
                  fill=CRAB_DARK, width=max(s // 60, 2))

        # Right leg (mirror)
        x1_r = int(cx + s * 0.15)
        x2_r = int(s * (1 - lx_frac))
        draw.line([x1_r, y1, x2_r, y2], fill=CRAB_DARK, width=max(s // 50, 3))
        draw.line([x2_r, y2, int(x2_r + s * 0.02), int(y2 + s * 0.03)],
                  fill=CRAB_DARK, width=max(s // 60, 2))

    # --- Claws (two big claws on top-sides) ---
    # Left claw arm
    claw_arm_w = max(s // 30, 4)
    # Arm segment
    la_x1, la_y1 = int(cx - s * 0.16), int(cy - s * 0.04)
    la_x2, la_y2 = int(cx - s * 0.28), int(cy - s * 0.16)
    draw.line([la_x1, la_y1, la_x2, la_y2], fill=CLAW_COLOR, width=claw_arm_w)
    # Upper arm to claw
    la_x3, la_y3 = int(cx - s * 0.32), int(cy - s * 0.26)
    draw.line([la_x2, la_y2, la_x3, la_y3], fill=CLAW_COLOR, width=claw_arm_w)
    # Claw (pincer) - two arcs
    claw_r = int(s * 0.055)
    # Top pincer
    draw_ellipse(draw, int(la_x3 - claw_r * 0.3), int(la_y3 - claw_r * 0.5),
                 claw_r, int(claw_r * 0.7), fill=CRAB_BODY)
    # Bottom pincer
    draw_ellipse(draw, int(la_x3 + claw_r * 0.3), int(la_y3 + claw_r * 0.3),
                 claw_r, int(claw_r * 0.6), fill=CRAB_BODY)

    # Right claw arm (mirror)
    ra_x1, ra_y1 = int(cx + s * 0.16), int(cy - s * 0.04)
    ra_x2, ra_y2 = int(cx + s * 0.28), int(cy - s * 0.16)
    draw.line([ra_x1, ra_y1, ra_x2, ra_y2], fill=CLAW_COLOR, width=claw_arm_w)
    ra_x3, ra_y3 = int(cx + s * 0.32), int(cy - s * 0.26)
    draw.line([ra_x2, ra_y2, ra_x3, ra_y3], fill=CLAW_COLOR, width=claw_arm_w)
    claw_r = int(s * 0.055)
    draw_ellipse(draw, int(ra_x3 + claw_r * 0.3), int(ra_y3 - claw_r * 0.5),
                 claw_r, int(claw_r * 0.7), fill=CRAB_BODY)
    draw_ellipse(draw, int(ra_x3 - claw_r * 0.3), int(ra_y3 + claw_r * 0.3),
                 claw_r, int(claw_r * 0.6), fill=CRAB_BODY)

    # --- Body (main oval) ---
    body_rx = int(s * 0.20)
    body_ry = int(s * 0.15)
    draw_ellipse(draw, cx, cy, body_rx, body_ry, fill=CRAB_BODY)

    # Body highlight (smaller oval, slightly up-left)
    highlight_rx = int(body_rx * 0.6)
    highlight_ry = int(body_ry * 0.5)
    draw_ellipse(draw, int(cx - body_rx * 0.15), int(cy - body_ry * 0.25),
                 highlight_rx, highlight_ry, fill=CRAB_HIGHLIGHT)

    # Body shell lines (subtle arcs for texture)
    for i in range(3):
        arc_y = int(cy - body_ry * 0.1 + i * body_ry * 0.35)
        arc_x1 = int(cx - body_rx * 0.5)
        arc_x2 = int(cx + body_rx * 0.5)
        draw.arc([arc_x1, arc_y, arc_x2, int(arc_y + body_ry * 0.3)],
                 start=0, end=180, fill=CRAB_DARK, width=max(s // 200, 1))

    # --- Eyes (on stalks) ---
    eye_r = int(s * 0.03)
    # Left eye stalk
    le_base_x, le_base_y = int(cx - s * 0.08), int(cy - body_ry + s * 0.01)
    le_top_x, le_top_y = int(cx - s * 0.11), int(cy - body_ry - s * 0.06)
    draw.line([le_base_x, le_base_y, le_top_x, le_top_y],
              fill=CRAB_DARK, width=max(s // 60, 3))
    draw_ellipse(draw, le_top_x, le_top_y, eye_r, eye_r, fill=EYE_WHITE)
    draw_ellipse(draw, le_top_x + 1, le_top_y, int(eye_r * 0.55), int(eye_r * 0.55),
                 fill=EYE_PUPIL)

    # Right eye stalk
    re_base_x, re_base_y = int(cx + s * 0.08), int(cy - body_ry + s * 0.01)
    re_top_x, re_top_y = int(cx + s * 0.11), int(cy - body_ry - s * 0.06)
    draw.line([re_base_x, re_base_y, re_top_x, re_top_y],
              fill=CRAB_DARK, width=max(s // 60, 3))
    draw_ellipse(draw, re_top_x, re_top_y, eye_r, eye_r, fill=EYE_WHITE)
    draw_ellipse(draw, re_top_x - 1, re_top_y, int(eye_r * 0.55), int(eye_r * 0.55),
                 fill=EYE_PUPIL)

    # --- Chat bubble (small speech bubble to the upper-right) ---
    bub_cx = int(cx + s * 0.22)
    bub_cy = int(cy - s * 0.22)
    bub_rx = int(s * 0.09)
    bub_ry = int(s * 0.065)
    draw_ellipse(draw, bub_cx, bub_cy, bub_rx, bub_ry, fill=CHAT_COLOR)
    # Bubble tail (small triangle pointing to crab)
    tail_pts = [
        (int(bub_cx - bub_rx * 0.5), int(bub_cy + bub_ry * 0.7)),
        (int(bub_cx - bub_rx * 1.0), int(bub_cy + bub_ry * 1.5)),
        (int(bub_cx - bub_rx * 0.0), int(bub_cy + bub_ry * 0.9)),
    ]
    draw.polygon(tail_pts, fill=CHAT_COLOR)
    # Three dots inside bubble
    dot_r = int(s * 0.012)
    for i in range(3):
        dx = int(bub_cx - s * 0.03 + i * s * 0.03)
        draw_ellipse(draw, dx, bub_cy, dot_r, dot_r, fill=BG_COLOR)

    return img


def generate_png(size=512):
    """Generate the main PNG icon."""
    img = Image.new("RGBA", (size, size), BG_COLOR)
    draw_crab(img, size)
    out = ASSETS_DIR / "icon.png"
    img.save(out, "PNG")
    print(f"Generated {out} ({size}x{size})")
    return img


def generate_ico(base_img):
    """Generate Windows .ico with multiple sizes."""
    sizes = [16, 32, 48, 64, 128, 256]
    out = ASSETS_DIR / "icon.ico"
    # Pillow ICO save requires passing sizes as the target icon sizes
    # and the image will be resized to each. We save from the largest image.
    base_rgba = base_img.convert("RGBA")
    base_rgba.save(out, format="ICO", sizes=[(sz, sz) for sz in sizes])
    print(f"Generated {out} (sizes: {sizes})")


def generate_icns(base_img):
    """Generate macOS .icns using iconutil."""
    if shutil.which("iconutil") is None:
        print("iconutil not found — skipping .icns generation (macOS only)")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        iconset_dir = Path(tmpdir) / "icon.iconset"
        iconset_dir.mkdir()

        # Required sizes for iconutil
        icon_sizes = {
            "icon_16x16.png": 16,
            "icon_16x16@2x.png": 32,
            "icon_32x32.png": 32,
            "icon_32x32@2x.png": 64,
            "icon_128x128.png": 128,
            "icon_128x128@2x.png": 256,
            "icon_256x256.png": 256,
            "icon_256x256@2x.png": 512,
            "icon_512x512.png": 512,
        }

        for name, sz in icon_sizes.items():
            resized = base_img.resize((sz, sz), Image.LANCZOS)
            resized.save(iconset_dir / name, "PNG")

        out = ASSETS_DIR / "icon.icns"
        subprocess.run(
            ["iconutil", "-c", "icns", str(iconset_dir), "-o", str(out)],
            check=True,
        )
        print(f"Generated {out}")


def main():
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    base_img = generate_png(512)
    generate_ico(base_img)
    generate_icns(base_img)
    print("Done! All icons generated in assets/")


if __name__ == "__main__":
    main()
