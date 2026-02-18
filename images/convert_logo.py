"""
將 pigmodel logo 的白色背景改為 #0F172A，有顏色的部分改為白色。
改良版：讓有顏色的部分更加明亮清晰。
"""
from PIL import Image
import numpy as np

# 載入圖片
img = Image.open(r"d:\Coding\ipig_system\images\pigmodel logo.png").convert("RGBA")
data = np.array(img, dtype=np.float64)

# 目標背景色
bg = np.array([15, 23, 42], dtype=np.float64)  # #0F172A

rgb = data[:, :, :3]
alpha = data[:, :, 3]

# 計算每個像素的亮度
gray = 0.299 * rgb[:,:,0] + 0.587 * rgb[:,:,1] + 0.114 * rgb[:,:,2]

# 計算色飽和度
max_c = np.max(rgb, axis=2)
min_c = np.min(rgb, axis=2)
saturation = np.where(max_c > 0, (max_c - min_c) / max_c, 0)

# 白色/淺色區域的遮罩（高亮度、低飽和度）
is_light = (gray > 200) & (saturation < 0.15)

# 完全透明的像素
is_transparent = alpha < 10

# 有顏色的像素
colored_mask = (~is_light) & (~is_transparent)

# 建立新圖片
new_rgb = np.zeros_like(rgb)
new_alpha = np.full_like(alpha, 255.0)

# 透明像素 → 背景色
new_rgb[is_transparent] = bg

# 白色/淺色像素 → 背景色
light_mask = is_light & (~is_transparent)
new_rgb[light_mask] = bg

# 有顏色的像素 → 白色
# 使用更強烈的對比：原始亮度越低（越深），新亮度越高（越白）
# 對深色部分（如藍色線條），要讓它們變成純白
inv_gray = 255.0 - gray[colored_mask]

# 使用 Gamma 校正增強對比度，讓深色部分更白
inv_norm = inv_gray / 255.0
# Gamma < 1 會讓暗部提亮
gamma = 0.3
inv_norm_corrected = np.power(inv_norm, gamma)

for c in range(3):
    new_rgb[colored_mask, c] = 255.0 * inv_norm_corrected + bg[c] * (1.0 - inv_norm_corrected)

# 保留原始 alpha
new_alpha[colored_mask] = alpha[colored_mask]

# 組合
new_data = np.zeros_like(data)
new_data[:,:,:3] = np.clip(new_rgb, 0, 255)
new_data[:,:,3] = np.clip(new_alpha, 0, 255)

result = Image.fromarray(new_data.astype(np.uint8), 'RGBA')
result.save(r"d:\Coding\ipig_system\images\pigmodel logo dark.png")
print("完成！已儲存 pigmodel logo dark.png")
