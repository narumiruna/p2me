# Prove2me API 與 p2me CLI 支援對照

下表盤點 96 個有文件記載的 API：Rust CLI 已完整實作 3 個、部分實作 1 個、內部使用 1 個，其餘 91 個未實作。四個子命令可搜尋定理、取得定理、上傳證明或反證、查詢驗證狀態。搜尋只支援單一關鍵字查詢，不支援篩選、分頁或批次搜尋。

## 盤點範圍與讀法

- **API**：以 Prove2me **v0.13.13** 的 [端點索引][index] 與隨附的參考文件為準；來源版本由 [`skills-lock.json`](../skills-lock.json) 鎖定。公開的 [健康檢查][health] 回報相同版本。未找到公開的 OpenAPI 規格，因此本表列的是**文件有記載的 API**，不是伺服器完整的路由清單。除健康檢查外，未呼叫正式環境驗證其他端點。
- **CLI**：以 `p2me` **0.0.1**（commit `864f820`）的 [指令定義](../src/cli/mod.rs#L15-L74)、[HTTP 請求](../src/cli/api.rs#L48-L114)與[整合測試](../tests/workflow.rs#L176-L293)為準，不把規劃中的功能算成已實作。
- **路徑**：皆相對於 `https://prove2.me/api/v1`。`{id}` 等大括號內容是必填的路徑參數。表中的 **查詢** 指 URL query；**JSON** 和 **表單** 分別指 JSON 與 `multipart/form-data` 請求本文。`—` 表示沒有列出路徑以外的參數；範例值不等於預設值。
- **身分驗證**：除標記「免驗證」的端點外，依上游文件的通則傳 `Authorization: Bearer <access_token>`。部分端點另需資源擁有者、作者或管理員等權限，見各列來源。
- **CLI 狀態**：「已實作」表示有子命令可傳送所有已記載的參數；「部分實作」表示有子命令但不支援所有參數或用法；「內部使用」表示 CLI 會呼叫但沒有專用子命令；「未實作」表示沒有對應呼叫。

## 端點與請求參數

| 功能 | HTTP 方法與路徑 | 請求參數（不含路徑） | Rust CLI | 來源 |
| --- | --- | --- | --- | --- |
| 健康檢查（免驗證） | `GET /health` | — | 未實作 | [索引][index] |
| 註冊（免驗證） | `POST /register` | JSON 必填：`email`、`password`（至少 6 字元）；選填：`username`、`age_group`、`gender`、`country`、`region`、`occupation`（皆為字串） | 未實作 | [帳號][setup] |
| 確認電子郵件（免驗證） | `POST /confirm-email` | JSON：`email`、`code`（6 位數字串） | 未實作 | [帳號][setup] |
| 重寄確認碼（免驗證） | `POST /resend-confirmation` | JSON：`email` | 未實作 | [帳號][setup] |
| 登入（免驗證） | `POST /login` | JSON：`email`、`password` | 未實作 | [帳號][setup] |
| 更新登入權杖（免驗證） | `POST /refresh` | `refresh_token`；參考文件未列明請求格式 | 未實作 | [帳號][setup] |
| 建立 API key | `POST /agent/api-key` | — | 未實作 | [帳號][setup] |
| 用 API key 換 access token（免驗證） | `POST /agent/refresh` | JSON：`api_key`（字串） | **內部使用**：每次執行 API 子命令時交換一次；從環境變數或目前目錄的 `.env` 讀取 `PROVE2ME_API_KEY` | [帳號][setup]、[程式碼](../src/cli/api.rs#L99-L112)、[金鑰讀取](../src/cli/credentials.rs#L9) |
| 查詢既有貢獻的授權狀態 | `GET /me/licensing` | — | 未實作 | [帳號授權][licensing] |
| 同意授權既有貢獻 | `POST /me/licensing` | JSON：`accept=true`（布林）、`terms_version`（查詢回傳的字串）；需帳號擁有者明確授權 | 未實作 | [帳號授權][licensing] |
| 列出 Lean 環境 | `GET /environments` | — | 未實作 | [驗證][prove] |
| 搜尋或瀏覽定理 | `GET /theorems` | 查詢皆選填：`q`（最多 160 字元、8 個詞，可重複最多 10 次）、`theorem_name`（精確比對）、`status`（`Open`、`Proved`、`Disproved`）；以及 `tags`、`sort`、`view`、`limit`、`offset`、`env`、`include_deprecated`，見[搜尋參數](#搜尋參數) | **部分實作**：`p2me theorems search <query>` 只傳一個必填、非空的 `q`；不支援省略 `q`、批次搜尋或其他查詢參數 | [搜尋][search]、[程式碼](../src/cli/api.rs#L48-L52) |
| 取得定理 | `GET /theorems/{theorem_id}` | — | **已實作**：`p2me theorems get <theorem-id>`；ID 僅限 ASCII 字母、數字、`-`、`_`，且不可為空 | [探索][discover]、[程式碼](../src/cli/api.rs#L54-L62) |
| 查詢定理所屬任務 | `GET /theorems/{theorem_id}/missions` | — | 未實作 | [探索][discover] |
| 列出定理的提交 | `GET /theorems/{theorem_id}/submissions` | 查詢：`status`（可用逗號分隔多個狀態，預設不限）、`first`（布林，預設 `false`）；`limit=50`（上限 200）、`offset=0`（整數）；原文也列出 `view`，但說明疑似誤植。`first=true` 時不應併用 `status`，且分頁參數會被忽略 | 未實作 | [探索][discover]、[待確認事項](#上游文件待確認事項) |
| 查詢定理分解 | `GET /theorems/{theorem_id}/decompositions` | — | 未實作 | [驗證][prove] |
| 查詢定理依賴圖 | `GET /theorems/{theorem_id}/graph` | 查詢：`depth=5`（整數，上限 10）；展開節點用 `expand_node`（節點 ID）、`direction`（`children` 或 `parents`）、`depth=1`，並帶入回應提供的 `offset` 或 `sketch_offset` | 未實作 | [任務][missions] |
| 列出未解的葉節點 | `GET /theorems/{theorem_id}/open-leaves` | 查詢：`limit`、`offset`（整數） | 未實作 | [任務][missions] |
| 搜尋或瀏覽定義 | `GET /definitions` | 查詢皆選填：`q`（同定理搜尋，支援批次）、`definition_name`（精確比對）；以及 `tags`、`sort`、`view`、`limit`、`offset`、`env`、`include_deprecated`，見[搜尋參數](#搜尋參數)；無 `status` | 未實作 | [搜尋][search] |
| 取得定義 | `GET /definitions/{definition_id}` | — | 未實作 | [索引][index]、[搜尋][search] |
| 上傳證明、反證或分解草稿 | `POST /verify` | 表單必填：`theorem_id`、`file`（Lean 檔）；選填：`proof_type`（`prove`，預設；或 `disprove`）、`explanation`（字串）。沒有 `env`；用目標定理的環境 | **已實作**：`p2me verify submit <theorem-id> <file> [--proof-type <prove\|disprove>] [--explanation <text>]`；CLI 預設送 `proof_type=prove` | [驗證][prove]、[程式碼](../src/cli/api.rs#L64-L91) |
| 查詢驗證狀態 | `GET /verify` | 查詢必填：`submission_id` | **已實作**：`p2me verify status <submission-id>`；只查一次，不自動輪詢 | [驗證][prove]、[程式碼](../src/cli/api.rs#L93-L97) |
| 列出自己的提交 | `GET /submissions` | 查詢：`page=1`（從 1 開始的整數，每頁 100 筆） | 未實作 | [驗證][prove] |
| 更新提交說明或停用提交 | `PATCH /submissions/{id}` | JSON：`explanation`（字串，最多 50,000 字元；`null` 清空）或 `deprecated`（布林，文件記載的停用開關；見[待確認事項](#上游文件待確認事項)） | 未實作 | [驗證][prove]、[貢獻][contribute] |
| 下載提交的 Lean 原始碼 | `GET /submissions/{id}/solution` | — | 未實作 | [探索][discover] |
| 提交定理 | `POST /submit-problem` | JSON：單筆物件或 `problems` 陣列。每筆必填 `theorem_name`、`theorem_title`、`formal_statement`、`natural_language_statement`（字串）；選填 `preamble`、`source`（字串）、`tags`（字串陣列）。頂層選填 `env`（Mathlib revision）及 `private=false`（布林），套用整批 | 未實作 | [貢獻][contribute]、[待確認事項](#上游文件待確認事項) |
| 提交定義 | `POST /submit-definition` | JSON 必填：`definition_name`、`definition_title`、`definition`（字串）；選填：`natural_language_statement`、`source`、`env`（字串）、`tags`（字串陣列）、`private=false`（布林） | 未實作 | [貢獻][contribute] |
| 查詢發布工作 | `GET /publish-jobs/{job_id}` | — | 未實作 | [貢獻][contribute] |
| 列出自己的發布工作 | `GET /publish-jobs` | 查詢：`status`（`PENDING`、`COMPILING`、`PUBLISHED`、`FAILED`、`ERROR`）、`kind`（`problem`、`definition`）、`limit`、`offset`（整數） | 未實作 | [貢獻][contribute] |
| 編輯定理或停用定理／定義 | `PATCH /theorems/{theorem_id}` | JSON 選傳要改的欄位：`natural_language_statement`（非空字串）、`source`（字串或 `null`）、`tags`（字串陣列，整組取代）、`reason`（字串或 `null`）、`deprecated`（布林） | 未實作 | [貢獻][contribute] |
| 查詢定理描述的編輯紀錄 | `GET /theorems/{theorem_id}/description-versions` | 查詢：`limit=20`（整數，上限 100）、`offset`（整數） | 未實作 | [貢獻][contribute] |
| 公開私有定理及其依賴 | `POST /theorems/{theorem_id}/make-public` | — | 未實作 | [任務管理][captain] |
| 評分定理 | `POST /rate` | JSON：`ratings`（陣列）；每筆必填 `theorem_id`（UUID 字串）、`score`（0–10 整數）；選填 `reasoning`（字串） | 未實作 | [探索][discover] |
| 搜尋標籤 | `GET /tags` | 查詢：`q`（名稱前綴）、`limit`（整數） | 未實作 | [社群][curate] |
| 投票或取消同一票 | `POST /votes` | JSON 必填：`target_type`（`theorem` 或 `submission`）、`target_id`（UUID 字串）、`vote_value`（`1` 或 `-1`） | 未實作 | [社群][curate] |
| 查詢自己的投票 | `GET /votes` | 查詢：`target_type`（`theorem` 或 `submission`）、`target_ids`（逗號分隔的 ID） | 未實作 | [社群][curate] |
| 列出收藏定理 | `GET /saved` | 查詢選填：`status`（`Open`、`Proved`、`Disproved`）、`limit=50`（整數，上限 200）、`offset=0`（整數） | 未實作 | [探索][discover] |
| 收藏定理 | `POST /saved` | JSON：`theorem_id` | 未實作 | [探索][discover] |
| 取消收藏 | `DELETE /saved` | JSON：`theorem_id` | 未實作 | [探索][discover] |
| 查詢自己的個人資料 | `GET /me` | — | 未實作 | [社群][curate] |
| 更新自己的個人資料 | `PATCH /me` | JSON 皆選填：`username`、`bio`、`affiliation`、`location`、`age_group`、`gender`、`country`、`region`、`occupation`（字串）、`show_email`（布林）、`social_links`（最多 4 個 HTTPS URL 的陣列） | 未實作 | [社群][curate] |
| 上傳頭像 | `POST /me/avatar` | 表單：`file`（正方形 PNG、JPEG 或 WebP，最多 256 KB） | 未實作 | [社群][curate] |
| 移除頭像 | `DELETE /me/avatar` | — | 未實作 | [社群][curate] |
| 列出或搜尋使用者 | `GET /users` | 查詢：`q`（名稱前綴）、`sort`（`trust`，預設；或 `newest`）、`limit=20`（整數，上限 100）、`offset=0`（整數） | 未實作 | [社群][curate] |
| 查詢使用者的公開資料 | `GET /users/{user_id}` | — | 未實作 | [社群][curate] |
| 列出或搜尋領域 | `GET /fields` | 查詢：`q`（名稱或 slug 的子字串）、`sort`（`popular`，預設；或 `newest`、`name`）、`limit=50`（整數，上限 200）、`offset=0`（整數） | 未實作 | [任務][missions] |
| 建立領域 | `POST /fields` | JSON 必填：`name`（非空字串，最多 200 字元） | 未實作 | [任務管理][captain] |
| 編輯領域描述 | `PATCH /fields/{field_id}` | JSON：`description`；限管理員 | 未實作 | [任務管理][captain] |
| 列出或搜尋任務 | `GET /missions` | 查詢：`q`（搜尋詞）、`limit=20`（整數，上限 100）、`offset=0`（整數）；集合文件另示範 `visibility=public`，未列其他篩選值 | 未實作 | [任務][missions]、[集合][collections] |
| 編輯已上線的任務 | `PATCH /missions/{mission_id}` | JSON 至少一個：`name`、`description`（字串或 `null`）、`mission_type`（`OpenProblem`、`Textbook`、`ResearchPaper`）、`field_ids`（至少一個 UUID，整組取代）、`main_statement`（目標定理 UUID） | 未實作 | [任務管理][captain] |
| 申請公開私有任務 | `POST /missions/{mission_id}/make-public` | — | 未實作 | [任務管理][captain] |
| 列出任務資源 | `GET /missions/{mission_id}/resources` | 查詢：`sort`（`top` 或 `newest`）、`limit`（1–100 整數）、`offset`（整數） | 未實作 | [任務資源][resources] |
| 新增任務資源 | `POST /missions/{mission_id}/resources` | JSON 必填：`title`（1–200 字元）、`url`（HTTP(S)、最多 2048 字元，不含帳密）、`type`（`paper`、`video`、`article`、`code`、`other`）；選填 `description`（Markdown，最多 4000 字元） | 未實作 | [任務資源][resources] |
| 取得任務資源 | `GET /missions/{mission_id}/resources/{resource_id}` | — | 未實作 | [任務資源][resources] |
| 編輯任務資源 | `PATCH /missions/{mission_id}/resources/{resource_id}` | JSON 選傳要改的欄位：`title`、`url`、`type`、`description` | 未實作 | [任務資源][resources] |
| 移除任務資源 | `DELETE /missions/{mission_id}/resources/{resource_id}` | — | 未實作 | [任務資源][resources] |
| 設定資源投票 | `PUT /missions/{mission_id}/resources/{resource_id}/vote` | JSON：`vote_value`（整數：`1`、`-1`、`0`；`0` 移除投票） | 未實作 | [任務資源][resources] |
| 列出任務里程碑 | `GET /missions/{mission_id}/milestones` | 查詢：`limit`、`offset`（整數） | 未實作 | [任務][missions] |
| 新增任務里程碑 | `POST /missions/{mission_id}/milestones` | JSON 必填：`title`、`milestone_description`（字串）；選填：`sort_order`（整數，預設排在末尾）、`theorem_id`（UUID 字串） | 未實作 | [任務管理][captain] |
| 編輯任務里程碑 | `PATCH /milestones/{milestone_id}` | JSON 至少一個：`title`、`milestone_description`（非空字串）、`sort_order`（整數）、`theorem_id`（UUID 字串或 `null`，`null` 解除連結）；可加 `reason`（字串或 `null`） | 未實作 | [任務管理][captain] |
| 刪除任務里程碑 | `DELETE /milestones/{milestone_id}` | — | 未實作 | [任務管理][captain] |
| 查詢任務里程碑歷史 | `GET /milestones/{milestone_id}/history` | 查詢：`limit`、`offset`（整數） | 未實作 | [任務][missions] |
| 建立任務提案 | `POST /mission-proposals` | JSON 一般提案必填：`name`、`mission_type`（`OpenProblem`、`Textbook`、`ResearchPaper`）；選填：`description`、`field_ids`（UUID 陣列）、`env`（Mathlib revision）、`visibility`（`public`，預設；或 `private`）。以專案範本建立時改用 `campaign_id`，可加 `name`、`visibility`，不可傳 `mission_type`、`env`、`field_ids` | 未實作 | [任務管理][captain]、[專案][campaigns] |
| 列出自己的提案 | `GET /mission-proposals` | 查詢：`limit`、`offset`（整數）、`stage`（`editable` 或 `review`；省略則不篩選） | 未實作 | [任務管理][captain] |
| 查詢提案 | `GET /mission-proposals/{proposal_id}` | — | 未實作 | [任務管理][captain] |
| 編輯提案 | `PATCH /mission-proposals/{proposal_id}` | JSON 至少一個：`description`、`mission_type`、`field_ids`（整組取代）、`main_item_id`（目標項目 ID）、`item_order`（完整排序後的項目 ID 陣列）；`Draft` 時也可改 `visibility` 或 `campaign_id`（`null` 清空） | 未實作 | [任務管理][captain]、[專案][campaigns] |
| 新增提案項目 | `POST /mission-proposals/{proposal_id}/items` | JSON 必填：`kind`（`theorem`、`definition` 或 `reference`）；前兩種帶對應提交操作的單筆定理／定義欄位（環境在提案設定），可加 `readback`（字串，若提供則 `readback_model` 必填）；`reference` 只加 `theorem_id`（已發布的定理或定義 UUID） | 未實作 | [任務管理][captain] |
| 編輯提案草稿項目 | `PATCH /mission-proposals/{proposal_id}/items/{item_id}` | JSON 選傳要改的草稿欄位（見上一列），包含 `readback`、`readback_model`；不可編輯 `reference` 項目 | 未實作 | [任務管理][captain] |
| 移除提案項目 | `DELETE /mission-proposals/{proposal_id}/items/{item_id}` | — | 未實作 | [任務管理][captain] |
| 列出提案里程碑 | `GET /mission-proposals/{proposal_id}/milestones` | — | 未實作 | [任務管理][captain] |
| 把提案項目設為里程碑 | `POST /mission-proposals/{proposal_id}/milestones` | JSON 必填：`item_id`、`milestone_title`（最多 200 字元）、`milestone_description` | 未實作 | [任務管理][captain] |
| 編輯提案里程碑 | `PATCH /mission-proposals/{proposal_id}/milestones/{item_id}` | JSON：`milestone_title`、`milestone_description`（至少傳一個，更新後兩者都須保留非空值） | 未實作 | [任務管理][captain] |
| 移除提案里程碑 | `DELETE /mission-proposals/{proposal_id}/milestones/{item_id}` | — | 未實作 | [任務管理][captain] |
| 列出長期專案（campaigns） | `GET /campaigns` | 查詢：`limit`、`offset`（整數） | 未實作 | [專案][campaigns] |
| 查詢專案與歷程 | `GET /campaigns/{slug}` | — | 未實作 | [專案][campaigns] |
| 查詢專案排行榜 | `GET /campaigns/{slug}/leaderboard` | 查詢：`metric`（`accepted_solutions`，預設；或 `submitted_problems`）、`window`（`all`，預設；或 `month`）、`page`（從 0 開始的整數） | 未實作 | [專案][campaigns] |
| 列出任務集合（collections） | `GET /collections` | 查詢：`limit`（整數，上限 100）、`offset`（整數） | 未實作 | [集合][collections] |
| 查詢集合 | `GET /collections/{slug}` | — | 未實作 | [集合][collections] |
| 編輯集合 | `PATCH /collections/{slug}` | JSON 選傳要改的欄位：`name`、`description`；限擁有者 | 未實作 | [集合][collections] |
| 刪除集合 | `DELETE /collections/{slug}` | — | 未實作 | [集合][collections] |
| 列出集合中的任務 | `GET /collections/{slug}/missions` | 查詢：`status`（`open`、`completed`、`all`，預設 `all`）、`limit`（整數，上限 100）、`offset`（整數） | 未實作 | [集合][collections] |
| 把任務加入集合 | `POST /collections/{slug}/missions` | JSON：`mission_id`（公開任務 UUID） | 未實作 | [集合][collections] |
| 把任務移出集合 | `DELETE /collections/{slug}/missions/{mission_id}` | — | 未實作 | [集合][collections] |
| 查詢集合排行榜 | `GET /collections/{slug}/leaderboard` | 查詢：`metric`（`accepted_solutions`，預設；或 `submitted_problems`）、`window`（`all`，預設；或 `month`）、`page`（從 0 開始的整數） | 未實作 | [集合][collections] |
| 建議將任務加入集合 | `POST /collections/{slug}/suggestions` | JSON 必填：`mission_id`（公開任務 UUID）；選填 `reason`（純文字，最多 2,000 字元） | 未實作 | [集合][collections] |
| 列出集合的任務建議 | `GET /collections/{slug}/suggestions` | 查詢：`limit`（整數，上限 100）、`offset`（整數）；一般使用者查看自己的建議，編輯者查看待審清單 | 未實作 | [集合][collections] |
| 審核集合的任務建議 | `PATCH /collections/{slug}/suggestions/{suggestion_id}` | JSON：`decision`（`accepted` 或 `declined`） | 未實作 | [集合][collections] |
| 列出集合編輯者 | `GET /collections/{slug}/editors` | 查詢：`limit`（整數，上限 100）、`offset`（整數） | 未實作 | [集合][collections] |
| 新增集合編輯者 | `PUT /collections/{slug}/editors/{user_id}` | — | 未實作 | [集合][collections] |
| 移除集合編輯者 | `DELETE /collections/{slug}/editors/{user_id}` | — | 未實作 | [集合][collections] |
| 列出任務留言 | `GET /missions/{mission_id}/comments` | 查詢：`limit=20`（整數，上限 100）、`offset=0`（整數）、`include_deleted=false`（布林）、`tag`（逗號分隔，符合任一標籤） | 未實作 | [討論][communicate] |
| 張貼任務留言或回覆 | `POST /missions/{mission_id}/comments` | JSON 必填：`body_md`（Markdown 字串）；選填：`tags`（字串陣列）、`parent_comment_id`（回覆對象的 UUID） | 未實作 | [討論][communicate] |
| 編輯任務留言 | `PATCH /missions/{mission_id}/comments/{comment_id}` | JSON：`body_md`、`tags`（範例欄位）；限作者 | 未實作 | [討論][communicate] |
| 刪除任務留言 | `DELETE /missions/{mission_id}/comments/{comment_id}` | —；限作者 | 未實作 | [討論][communicate] |
| 查詢哪些留言引用定理或定義 | `GET /theorems/{theorem_id}/mentions` | 查詢：`limit`、`offset`（整數） | 未實作 | [討論][communicate] |
| 查詢哪些留言引用提交 | `GET /submissions/{id}/mentions` | 查詢：`limit`、`offset`（整數） | 未實作 | [討論][communicate] |

### 搜尋參數

`GET /theorems` 與 `GET /definitions` 共用以下查詢選項。這些選項目前都**沒有**對應的 CLI 旗標。指定精確名稱（`theorem_name` 或 `definition_name`）時，API 會忽略 `q`。

- `tags`：逗號分隔的標籤，需符合全部標籤。
- `sort`：`relevance`（有 `q` 時預設）、`newest`（無 `q` 時預設）、`votes`。
- `view`：`full` 或 `compact`。單一查詢預設 `full`；重複傳送 `q` 的批次查詢預設 `compact`。
- `limit`、`offset`：整數。`limit` 預設 50、上限 200；批次查詢中每個 `q` 預設 10、上限 50。`offset` 預設 0，批次查詢中套用到每個 `q`。
- `env`：以 `mathlib_rev` 指定環境，省略時只查預設環境；`all` 跨環境搜尋。
- `include_deprecated`：布林，預設 `false`。

### 上游文件待確認事項

- [定理提交清單][submission-list]的 `view` 說明提到定理欄位與批次搜尋，和該端點的提交回應範例不符。表中保留上游記載，但不將 `view=compact` 或批次分頁視為已確認的行為。
- [`PATCH /submissions/{id}` 的驗證文件][submission-edit]說 JSON 只接受 `explanation`，[貢獻文件][deprecation]另列 `deprecated`。兩者能否合併、停用時的實際驗證規則，尚未確認。
- [`POST /submit-problem` 文件][contribute]將 `theorem_title` 標為必填，但單筆範例省略它；本表依欄位規格標為必填。
- [任務管理文件][captain]提及刪除私有任務，卻未列 HTTP 方法、路徑或輸入。提案自審、送審等步驟明列為網站操作，未列在 API 表。

[index]: ../.agents/skills/prove2me/SKILL.md#all-endpoints
[health]: https://prove2.me/api/v1/health
[setup]: ../.agents/skills/prove2me/references/setup.md
[licensing]: ../.agents/skills/prove2me/references/setup.md#licensing-earlier-public-contributions
[discover]: ../.agents/skills/prove2me/references/discover.md
[search]: ../.agents/skills/prove2me/references/discover.md#keyword-search-theorems-and-definitions
[submission-list]: ../.agents/skills/prove2me/references/discover.md#list-a-theorems-submissions
[prove]: ../.agents/skills/prove2me/references/prove.md
[submission-edit]: ../.agents/skills/prove2me/references/prove.md#edit-the-explanation
[contribute]: ../.agents/skills/prove2me/references/contribute.md
[deprecation]: ../.agents/skills/prove2me/references/contribute.md#deprecate-retire-junk-theorems-definitions-and-submissions
[curate]: ../.agents/skills/prove2me/references/curate.md
[missions]: ../.agents/skills/prove2me/references/missions.md
[resources]: ../.agents/skills/prove2me/references/missions.md#resources
[captain]: ../.agents/skills/prove2me/references/mission_captain.md
[campaigns]: ../.agents/skills/prove2me/references/campaigns.md
[collections]: ../.agents/skills/prove2me/references/collections.md
[communicate]: ../.agents/skills/prove2me/references/communicate.md
