"""
產生透明底 favicon，並依淺色/深色背景使用不同 logo：
- favicon-light.ico：深色 logo、透明底（用於淺色背景）
- favicon-dark.ico：淺色 logo、透明底（用於深色背景）
"""
from PIL import Image
import os

IMAGES_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_DIR = os.path.join(IMAGES_DIR, "..", "frontend", "public")
SIZES = [(16, 16), (32, 32), (48, 48)]


def make_transparent(img: Image.Image, bg_color_threshold=40, is_dark_bg=True) -> Image.Image:
    """
    將背景變透明。
    is_dark_bg: True 表示原圖是深色底，要保留淺色部分；False 表示原圖是淺色底，要保留深色部分。
    """
    img = img.convert("RGBA")
    data = img.load()
    w, h = img.size

    for y in range(h):
        for x in range(w):
            r, g, b, a = data[x, y]
            gray = 0.299 * r + 0.587 * g + 0.114 * b

            if is_dark_bg:
                # 深色底圖：深色像素（背景）→ 透明
                if gray < 80 and a > 200:
                    data[x, y] = (r, g, b, 0)
            else:
                # 淺色底圖：淺色像素（背景）→ 透明
                if gray > 200 and a > 200:
                    data[x, y] = (r, g, b, 0)

    return img


def save_favicon(img: Image.Image, filename: str):
    img.save(os.path.join(OUTPUT_DIR, filename), format="ICO", sizes=SIZES)
    print(f"  OK: {filename}")


def main():
    # 1. 淺色 logo 版（原圖深色底）→ 給深色背景用
    print("產生 favicon-dark.ico（淺色 logo、透明底，用於深色背景）...")
    img_dark = Image.open(os.path.join(IMAGES_DIR, "pigmodel logo dark.png"))
    img_dark = make_transparent(img_dark, is_dark_bg=True)
    save_favicon(img_dark, "favicon-dark.ico")

    # 2. 深色 logo 版（原圖淺色底）→ 給淺色背景用
    print("產生 favicon-light.ico（深色 logo、透明底，用於淺色背景）...")
    img_light = Image.open(os.path.join(IMAGES_DIR, "pigmodel logo.png"))
    img_light = make_transparent(img_light, is_dark_bg=False)
    save_favicon(img_light, "favicon-light.ico")

    # 3. 預設 fallback（與 light 相同，多數瀏覽器分頁為淺色）
    print("產生 favicon.ico（預設 fallback）...")
    save_favicon(img_light, "favicon.ico")

    print("\n完成！")


if __name__ == "__main__":
    main()
