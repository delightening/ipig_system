# Docker Image Digest Pinning（High 8）

生產與監控用第三方 image 以 `image: name:tag@sha256:digest` 釘選，降低 supply chain 風險。  
更新時：`docker pull name:tag` 後執行 `docker inspect --format '{{index .RepoDigests 0}}' name:tag` 取得新 digest，更新本表與對應 compose。

| Image | Tag | Digest (sha256:) |
|-------|-----|------------------|
| postgres | 16-alpine | 20edbde7749f822887a1a022ad526fde0a47d6b2be9a8364433605cf65099416 |
| prom/prometheus | v2.51.0 | 5ccad477d0057e62a7cd1981ffcc43785ac10c5a35522dc207466ff7e7ec845f |
| prom/alertmanager | v0.27.0 | e13b6ed5cb929eeaee733479dce55e10eb3bc2e9c4586c705a4e8da41e5eacf5 |
| grafana/grafana | 10.4.0 | f9811e4e687ffecf1a43adb9b64096c50bc0d7a782f8608530f478b6542de7d5 |
| containrrr/watchtower | latest | 6dd50763bbd632a83cb154d5451700530d1e44200b268a4e9488fefdfcf2b038 |

其他（docker-compose.logging.yml）可於部署前依同法取得 digest 後補上。
