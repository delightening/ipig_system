# 檔案儲存設定說明 (Storage Setup Guide)

## Docker 環境下的檔案儲存

### 基本設定

系統使用環境變數 `UPLOAD_DIR` 來指定上傳檔案的儲存目錄，預設為 `/app/uploads`（容器內路徑）。

### 本地開發環境

在 `docker-compose.yml` 中，已經設定了 volume mount：

```yaml
volumes:
  - ${UPLOAD_VOLUME:-./uploads}:/app/uploads
```

這會將主機的 `./uploads` 目錄掛載到容器的 `/app/uploads`。

**使用方式：**
1. 在專案根目錄建立 `uploads` 資料夾（如果不存在）
2. 確保該資料夾有適當的寫入權限
3. 啟動 Docker 容器後，檔案會儲存在主機的 `./uploads` 目錄中

### NAS 儲存設定

如果要在 NAS 上運行並使用 NAS 的儲存空間，有兩種方式：

#### 方式一：直接掛載 NAS 目錄（推薦）

1. **在主機上掛載 NAS：**
   ```bash
   # 例如使用 NFS
   sudo mount -t nfs nas-ip:/path/to/nas/share /mnt/nas
   
   # 或使用 CIFS/SMB
   sudo mount -t cifs //nas-ip/share /mnt/nas -o username=user,password=pass
   ```

2. **設定環境變數：**
   在 `.env` 檔案中設定：
   ```env
   UPLOAD_VOLUME=/mnt/nas/ipig/uploads
   ```

3. **更新 docker-compose.yml：**
   ```yaml
   volumes:
     - ${UPLOAD_VOLUME}:/app/uploads
   ```

4. **確保目錄存在且有權限：**
   ```bash
   sudo mkdir -p /mnt/nas/ipig/uploads
   sudo chown -R $(id -u):$(id -g) /mnt/nas/ipig/uploads
   ```

#### 方式二：使用 Docker volume driver

如果 NAS 支援 Docker volume plugin（如 NFS、CIFS），可以直接使用：

```yaml
volumes:
  uploads:
    driver: local
    driver_opts:
      type: nfs
      o: addr=nas-ip,nolock,soft
      device: ":/path/to/nas/share"
```

然後在 service 中使用：
```yaml
volumes:
  - uploads:/app/uploads
```

### 檔案結構

上傳的檔案會按照以下結構組織：

```
uploads/
├── pigs/
│   └── {pig_id}/
│       └── {date}_{uuid}.{ext}
├── protocols/
│   └── {protocol_id}/
│       └── {date}_{uuid}.{ext}
├── pathology/
│   └── {pig_id}/
│       └── {date}_{uuid}.{ext}
└── vet-recommendations/
    └── {record_type}_{record_id}/
        └── {date}_{uuid}.{ext}
```

### 效能考量

#### 使用 NAS 時的建議：

1. **網路延遲：**
   - 確保 NAS 與 Docker 主機之間的網路連線穩定
   - 考慮使用 Gigabit 或更快的網路連線
   - 如果可能，使用本地 SSD 快取常用檔案

2. **並發處理：**
   - 系統已經設計為非同步處理，可以同時處理多個上傳請求
   - 對於大量檔案上傳，考慮批次處理

3. **備份策略：**
   - NAS 通常有內建的備份功能，建議啟用
   - 定期備份 `uploads` 目錄
   - 考慮使用版本控制或快照功能

4. **儲存空間：**
   - 監控 NAS 的儲存空間使用情況
   - 設定適當的檔案保留政策
   - 考慮壓縮舊檔案或歸檔

### 安全性考量

1. **權限設定：**
   - 確保只有應用程式使用者可以寫入上傳目錄
   - 設定適當的檔案權限（建議 755 目錄，644 檔案）

2. **網路安全：**
   - 如果 NAS 在網路上，使用加密連線（NFSv4 with Kerberos 或 CIFS with encryption）
   - 限制 NAS 的網路存取

3. **檔案驗證：**
   - 系統已經實作了檔案類型驗證
   - 檔案大小限制已設定（照片 10MB，附件 20-50MB）

### 故障排除

#### 問題：無法寫入檔案

**解決方案：**
1. 檢查目錄權限：`ls -la /mnt/nas/ipig/uploads`
2. 檢查 Docker 容器的使用者 ID：`docker exec ipig-api id`
3. 調整權限：`sudo chown -R 1000:1000 /mnt/nas/ipig/uploads`（假設容器使用者 ID 是 1000）

#### 問題：NAS 掛載點失效

**解決方案：**
1. 檢查掛載狀態：`mount | grep nas`
2. 重新掛載：`sudo mount -a`
3. 考慮使用 `/etc/fstab` 設定自動掛載

#### 問題：上傳速度慢

**解決方案：**
1. 檢查網路頻寬：`iperf3 -c nas-ip`
2. 檢查 NAS 效能：查看 NAS 的資源使用情況
3. 考慮使用本地快取或 CDN

### 範例設定檔

#### `.env` 範例（本地開發）
```env
UPLOAD_VOLUME=./uploads
UPLOAD_DIR=/app/uploads
```

#### `.env` 範例（NAS 儲存）
```env
UPLOAD_VOLUME=/mnt/nas/ipig/uploads
UPLOAD_DIR=/app/uploads
```

#### `docker-compose.yml` 相關設定
```yaml
api:
  environment:
    UPLOAD_DIR: ${UPLOAD_DIR:-/app/uploads}
  volumes:
    - ${UPLOAD_VOLUME:-./uploads}:/app/uploads
```

### 注意事項

1. **資料持久性：**
   - 如果使用 Docker volume，確保 volume 有正確備份
   - 如果使用 bind mount，確保主機目錄有備份

2. **容器重建：**
   - 檔案儲存在 volume 中，重建容器不會影響已上傳的檔案
   - 但要注意 volume 的命名和生命週期

3. **多主機部署：**
   - 如果有多個 API 容器，必須使用共享儲存（如 NAS）
   - 確保所有容器都掛載同一個儲存位置

4. **效能監控：**
   - 監控上傳目錄的磁碟使用量
   - 設定告警當儲存空間不足時通知
