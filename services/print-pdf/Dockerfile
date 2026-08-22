# ============================================
# print-pdf — FastAPI HTML→PDF via Chromium (Playwright)
# ============================================
# 取代 WeasyPrint（2026-06-26）：商用標楷體 DFKai-SB 為 point-matching composite，
# WeasyPrint 嵌入會破字；Chromium 原生處理（像瀏覽器 / Word）→ 標楷體乾淨紮實，
# 且 @page 頁眉頁尾 + counter(page) 新版 Chromium 已支援。
# base image 已含 chromium + 系統依賴。
FROM mcr.microsoft.com/playwright/python:v1.62.0-jammy AS production

# CJK 字型：8 份非 GLP 文件經 base.html 以「Noto Sans CJK TC」名稱引用 → 需系統字型。
# 3 份 GLP 文件（aup_protocol / vet_patrol / vet_patrol_report）主頁排版用 @font-face
# file:///app/fonts/ 自帶 kaiu(標楷體)/times/arial。
# 但 vet_patrol_report 的頁首頁尾走 Chromium 原生 header/footer（獨立 iframe，
# @font-face file:// 載不到）→ 標楷體/Times 必須另裝成系統字型（見下方 fc-cache 步驟）。
ENV TZ=Asia/Taipei
RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        fonts-noto-cjk fonts-noto-core fonts-liberation tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY main.py samples.py floor_plan.py ./
COPY schemas ./schemas
COPY adapters ./adapters
COPY templates ./templates
COPY static ./static
COPY fonts ./fonts

# 頁首頁尾（Chromium 原生 header/footer iframe）需系統字型才引用得到標楷體/Times
# （file:// @font-face 在該受限 iframe 載不到）。將 GLP 商業字型註冊進 fontconfig，
# main.py 的 header/footer template 即可用 family name「標楷體」「Times New Roman」引用。
# （實測：裝進系統後 fc-match「標楷體」→ 真 kaiu.ttf、「Times New Roman」→ 真 times.ttf，
#   贏過容器預設別名 WenQuanYi / Liberation Serif。）
RUN mkdir -p /usr/share/fonts/truetype/drpig && \
    cp /app/fonts/kaiu.ttf /app/fonts/times.ttf /app/fonts/timesbd.ttf \
       /app/fonts/timesi.ttf /app/fonts/timesbi.ttf /usr/share/fonts/truetype/drpig/ && \
    fc-cache -f >/dev/null

# 非 root 執行（playwright image 內建 pwuser）。/app 須可寫（render 暫存 .html）。
RUN chown -R pwuser:pwuser /app
USER pwuser

ENV PORT=9200 TZ=Asia/Taipei
EXPOSE 9200

HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=3 \
  CMD python -c "import urllib.request,sys,os;p=os.getenv('PORT','9200');sys.exit(0 if urllib.request.urlopen(f'http://127.0.0.1:{p}/healthz').status==200 else 1)" || exit 1

CMD ["sh", "-c", "uvicorn main:app --host 0.0.0.0 --port ${PORT}"]
