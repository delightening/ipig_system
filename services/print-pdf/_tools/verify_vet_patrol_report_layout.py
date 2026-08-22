"""一次性驗證腳本：巡場報告 PDF 的一頁化 + 圖說兩行（2026-08-07）。

不是 runtime 依賴，也不進 CI。用途是在改 `templates/vet_patrol_report.html`
（字級/行高/padding 階梯、簽名區分頁行為、圖說結構）之後，對著真的 Chromium
render 出來的 PDF 驗證頁數與內容，而不是靠讀 CSS 推論。

用法：先起一個掛上修改後檔案的拋棄式容器，再指向它：

    rtk docker run --rm -d --name ipig-print-pdf-verify -p 127.0.0.1:9211:9200 \
      -v .../main.py:/app/main.py:ro -v .../templates:/app/templates:ro \
      -v .../adapters:/app/adapters:ro -v .../schemas:/app/schemas:ro \
      ipig_system-print-pdf
    python _tools/verify_vet_patrol_report_layout.py http://127.0.0.1:9211

驗證完記得收掉：`rtk docker stop ipig-print-pdf-verify`（帶 --rm，stop 即移除）。
"""

from __future__ import annotations

import base64
import io
import json
import sys
import urllib.request

from pypdf import PdfReader

ENDPOINT = "/render-vet-patrol-report/from-report-data"

# 1x1 紅點 PNG（內容不重要，只要是合法影像讓 adapter 的 Pillow 壓縮走得過去）
_DOT = base64.b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
)
DOT_URL = "data:image/png;base64," + base64.b64encode(_DOT).decode()


def cat(label: str, entries: list[tuple[str, str, str]]) -> dict:
    return {
        "label": label,
        "observation": "",
        "suggestion": "",
        "follow_up": "",
        "entries": [{"observation": o, "suggestion": s, "follow_up": f} for o, s, f in entries],
        "photos": [],
    }


def payload(pig_entries: list[tuple[str, str, str]], groups: list[dict] | None = None) -> dict:
    return {
        "vet_name": "使用者A",
        "companion": "使用者B",
        "patrol_date": "2026-07-20",
        "patrol_date_display": "2026年07月20日",
        "categories": [
            cat("豬隻狀況", pig_entries),
            cat("防疫及消毒計畫", [("全場定期清洗消毒（每週一次，週三）。分娩舍消毒噴霧罐已補。", "", "")]),
            cat("病歷紀錄", []),
            cat(
                "其他",
                [
                    (
                        "1. 09:50 A棟溫度27.7°C，濕度73%，前半無風、悶，風扇未啟動。"
                        "B棟溫度28.2°C，濕度77%，檢疫舍溫度27.9°C，濕度83%。\n"
                        "2. 檢疫舍羊飼料槽高度已改善，有放置鹽磚，飲水槽乾淨，原蹄甲異常已修正。",
                        "1. 檢疫舍豬隻精神食慾尚可(有殘飼)，無其他明顯異常。\n"
                        "2. 如有鼻分泌液偏多，以長效 penicillin 治療。",
                        "羊隻部分維持觀察，如有鼻分泌過多，給予Penicillin及Meloxicam IM SID 7天",
                    )
                ],
            ),
        ],
        "photos": [],
        "photo_groups": groups or [],
    }


def render(base: str, body: dict) -> bytes:
    req = urllib.request.Request(
        base + ENDPOINT,
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as r:
        return r.read()


def pages(pdf: bytes) -> list[str]:
    return [p.extract_text() or "" for p in PdfReader(io.BytesIO(pdf)).pages]


SIGN_MARK = "巡場獸醫師"


def photo_pages(pdf: bytes) -> list[str]:
    """只取照片頁的文字。

    照片區塊在範本中排在簽名區之後、且每組都帶 `page-break-before: always`，
    所以「簽名頁之後的頁」就是照片頁。用來把圖說的計數與主表格的內文隔開
    （耳號字串在兩邊都會出現）。
    """
    txt = pages(pdf)
    sign_idx = next((i for i, t in enumerate(txt) if SIGN_MARK in t), -1)
    return txt[sign_idx + 1 :] if sign_idx >= 0 else []


BODY_MARK = "維持觀察"


def body_font_pt(pdf: bytes, marker: str = BODY_MARK) -> float:
    """含 `marker` 的那段正文的字級（pt，四捨五入到 0.1）；找不到回 0。

    為什麼需要這個：只斷言頁數抓不到「過度壓縮」。一頁化最主要的失效模式是
    `LIMIT_PX` 算太小或量測偏高，導致**每一份**都被無謂壓到 9pt——那時頁數
    仍然是 1，所有只看頁數的斷言都會全綠（審查於 2026-08-08 指出）。

    為什麼不用「出現最多的字級」：頁首頁尾（文件編號／頁碼／版權）是固定 9pt、
    每頁都出現約 127 個字元，在正文稀疏的頁上會蓋過正文成為眾數。實測第 1 頁
    只有兩行標題時，眾數就是頁尾的 9pt，量到的完全不是想量的東西。改為直接
    鎖定一個只出現在 tbody 的字串。

    單位換算：visitor 拿到的是 CSS px，pt = px × 72 / 96。
    """
    found: list[float] = []

    def visitor(text, cm, tm, font_dict, font_size):  # noqa: ANN001, ARG001
        if marker not in (text or ""):
            return
        scale = tm[3] if tm and len(tm) > 3 else 1
        found.append(abs(float(font_size) * float(scale)) * 72.0 / 96.0)

    for page in PdfReader(io.BytesIO(pdf)).pages:
        page.extract_text(visitor_text=visitor)
        if found:
            break
    return round(found[0], 1) if found else 0.0


# 只用 tbody 才有的字串判斷「這頁有表格內容」。
# 刻意**不含「觀察內容」**——那是 thead 文字，而 `table.rpt thead` 帶
# `display: table-header-group`，Chromium 會在每個 table fragment 重複表頭，
# 可能產出「只有重複表頭、沒有任何資料列」的頁。若簽名落在那種頁上，視覺上
# 就是「簽名幾乎獨佔一頁」（正要防的 bug），用 thead 字串卻會判成 PASS
# （審查於 2026-08-08 指出）。
TBODY_MARKS = ("豬隻狀況", "防疫及消毒計畫", "維持觀察", "試驗中")


def check(
    name: str,
    pdf: bytes,
    want_pages: int | None,
    want_sign_page_has_table: bool,
    want_font_pt: float | None = None,
) -> bool:
    txt = pages(pdf)
    n = len(txt)
    sign_idx = next((i for i, t in enumerate(txt) if SIGN_MARK in t), -1)
    ok = True
    detail = [f"pages={n}", f"sign_on_page={sign_idx + 1}"]

    if want_pages is not None and n != want_pages:
        ok = False
        detail.append(f"EXPECTED pages={want_pages}")
    if sign_idx < 0:
        ok = False
        detail.append("SIGN BLOCK MISSING")
    elif want_sign_page_has_table:
        page_txt = txt[sign_idx]
        has_table = any(k in page_txt for k in TBODY_MARKS)
        detail.append(f"sign_page_has_table={has_table}")
        if not has_table:
            ok = False
            detail.append("SIGN ALONE ON ITS OWN PAGE")

    if want_font_pt is not None:
        got = body_font_pt(pdf)
        detail.append(f"font={got}pt(want~{want_font_pt})")
        # 容差必須小於階梯相鄰兩級的 0.5pt 差距，否則區分不出「沒壓」與「壓了一級」。
        # 原本寫 0.6（大於 0.5）是個 bug：abs(11.5-12.0)=0.5 仍會通過，
        # 等於字級回歸檢查形同虛設（CodeRabbit 於 2026-08-09 指出）。
        if abs(got - want_font_pt) > 0.2:
            ok = False
            detail.append("FONT SIZE OFF")

    print(("PASS " if ok else "FAIL ") + name + " :: " + ", ".join(detail))
    return ok


def main() -> int:
    base = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:9211"
    results = []

    # A. 短報告（1 條目）——本來就塞得下，**不應被壓縮**（字級須維持 12pt），且必須 1 頁。
    #    字級斷言擋的是「LIMIT_PX 算太小 / 量測偏高 → 每份都被無謂壓到 9pt」這個失效模式；
    #    那時頁數仍是 1，只看頁數會全綠（審查於 2026-08-08 指出）。
    results.append(
        check(
            "A 短報告（不得壓縮）",
            render(base, payload([("#592 精神採食良好，雙眼無明顯異常。", "維持觀察。", "維持觀察")])),
            1,
            True,
            want_font_pt=12.0,
        )
    )

    # 依使用者 2026-07-20 那份截圖重建的內容。注意：**這組在修正前也是 1 頁**——
    # 截圖的第 1 頁上緣被裁掉，實際條目比看得到的多，所以它不是有效的迴歸案例，
    # 只當「不要把原本正常的報告弄壞」的守門用。真正的失效樣態見 B。
    real = [
        ("#215 無皮膚紅疹，精神尚可。", "維持觀察。", "維持觀察"),
        ("#282、#283、#284精神食慾正常，肩胛植入電刺激器之傷口對合良好，無明顯發炎。", "維持正常照護。", "維持照護"),
        ("#674、#691、#818 右頸注射部位無明顯異常，#691精神食慾尚可，輕微腹式呼吸。", "維持觀察", "維持觀察"),
        ("#592 精神採食良好，雙眼無明顯異常。", "維持觀察。", "維持觀察"),
    ]
    results.append(check("A2 截圖重建（修正前後皆應 1 頁）", render(base, payload(real)), 1, True))

    # B. 真正的失效樣態：5 條目時表格剛好把簽名區擠過頁緣，
    #    修正前＝2 頁且第 2 頁「只有簽名」，修正後＝1 頁。
    #    （2026-08-07 對照實測：n=4/5/16/17 都會讓對照組出現簽名獨佔一頁。）
    repro = [
        (
            f"#{700 + i} 精神食慾正常，體表無明顯異常，採食量穩定，飲水正常，糞便成形，活動力佳。",
            "維持正常照護。",
            "維持觀察",
        )
        for i in range(5)
    ]
    results.append(check("B 簽名獨佔一頁的重現案例", render(base, payload(repro)), 1, True))

    # C. 超長報告（24 條目）——壓縮到下限仍會跨頁，但簽名不得單獨成頁，
    #    且字級**必須還原成 12pt**：一頁化既然不可能達成，就不該犧牲字級。
    #
    #    這是修 F1 之前的行為缺陷：舊版跑完 7 級階梯就停在 9pt 然後宣告完成，
    #    結果既沒省到頁數、又把 GLP 歸檔文件壓到下限，兩頭空。2026-08-07 的掃描
    #    數據（n=12~15 印 2 頁、n=18~20 印 3 頁）其實早就顯示了，當時沒看出來。
    #    對照實測：對照組 n=12/16/24 皆為 12pt，修正版必須一致。
    long_entries = [
        (f"#{700 + i} 精神食慾正常，體表無明顯異常，採食量穩定，飲水正常，糞便成形。", "維持正常照護。", "維持觀察")
        for i in range(24)
    ]
    results.append(
        check("C 超長報告（塞不下須還原 12pt）", render(base, payload(long_entries)), None, True, want_font_pt=12.0)
    )

    # D. 圖說：一張有說明、一張沒填 —— 有填的要印出說明，沒填的只印耳號
    groups = [
        {
            "caption": "#674、#691、#818",
            "description": "右頸注射部位無明顯異常",
            "srcs": [DOT_URL, DOT_URL],
            "photos": [
                {"src": DOT_URL, "caption": "右頸注射部位，無紅腫熱痛"},
                {"src": DOT_URL, "caption": ""},
            ],
        }
    ]
    pdf = render(base, payload(real, groups))
    # 只在**照片頁**範圍內計數：耳號字串也出現在主表格的觀察內容裡，對整份 PDF
    # 數會多算一次。照片頁都帶 page-break-before 且排在簽名區之後，故取簽名頁之後。
    photo_txt = "\n".join(photo_pages(pdf))
    # 用 count 而非 `in`：本次修的 bug 正是「同組每張照片共用同一個圖說」，
    # 只檢查「有出現」的話，範本若把第一張的說明複製到第二張，測試照樣會過
    # （CodeRabbit #44 指出）。故要求說明**恰好一次**、耳號**每張各一次**。
    want_tags = len(groups[0]["photos"])
    desc_n = photo_txt.count("右頸注射部位，無紅腫熱痛")
    tag_n = photo_txt.count("#674、#691、#818")
    ok_d = desc_n == 1 and tag_n == want_tags
    print(
        ("PASS " if ok_d else "FAIL ")
        + f"D 圖說帶說明 :: desc={desc_n}(want 1), tags={tag_n}(want {want_tags})"
    )
    results.append(ok_d)

    # F. 4 格照片頁（g4）＋長圖說 —— 全檔最緊的版面，先前**零覆蓋**。
    #
    #    D 用 2 張走 g2、E 用 1 張走 g1，`pp=4` 這條路徑一次都沒跑到，而它正是
    #    圖說改成兩行後最容易溢頁的地方（審查於 2026-08-08 算出：原本 66mm 的照片格
    #    只留 21.6mm 給圖說列，說明折到第 3 行就溢 0.78mm，耳號自己折兩行時更早）。
    #    照片格已降到 60mm 換取餘裕；本案例鎖住「4 張照片仍收在同一頁」。
    #    圖說長度定在 130 字：那是「66mm 溢成 2 頁、60mm 收在 1 頁」的區間，
    #    才真的鎖得住本次改善。實測（同一份範本、只差照片格高度）：
    #        字數   66mm   60mm
    #         72     1      1
    #        100     2      1
    #        130     2      1     ← 本案例用這個
    #        150     2      2     ← 兩者都不夠，屬已知殘留限制
    #
    #    ⚠️ 不可拿 prod 舊映像當這一項的對照組：舊範本根本不渲染 per-photo 圖說
    #    （只印 g.caption 的耳號），沒有長圖說就永遠不會溢頁，比較無意義。
    long_cap = (
        "右頸注射部位無明顯紅腫熱痛，採食與飲水正常，精神狀態良好，活動力佳，"
        "體表無外傷與皮膚病灶，糞便成形無異味，建議維持現行照護並持續觀察三日後複查，"
        "若出現食慾下降或體溫升高應即時通報獸醫師"
    )[:130]
    g4_groups = [
        {
            "caption": "#801、#802、#803、#804、#805、#806",  # 夠長，會讓耳號行自己折行
            "description": "",
            "srcs": [DOT_URL] * 4,
            "photos": [{"src": DOT_URL, "caption": long_cap} for _ in range(4)],
        }
    ]
    pdf = render(base, payload(real, g4_groups))
    n_photo_pages = len(photo_pages(pdf))
    ok_f = n_photo_pages == 1
    print(
        ("PASS " if ok_f else "FAIL ")
        + f"F g4 四格照片頁不溢頁 :: photo_pages={n_photo_pages}(want 1)"
    )
    results.append(ok_f)

    # E. 舊 payload（只有 srcs、沒有 photos）仍須正常出圖說＝耳號
    old_groups = [{"caption": "#275", "description": "", "srcs": [DOT_URL]}]
    pdf = render(base, payload(real, old_groups))
    txt = "\n".join(pages(pdf))
    ok_e = "#275" in txt
    print(("PASS " if ok_e else "FAIL ") + "E 舊 payload 相容 :: tag_rendered=" + str(ok_e))
    results.append(ok_e)

    print("\n" + ("ALL PASS" if all(results) else "SOME FAILED"))
    return 0 if all(results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
