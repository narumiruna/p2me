# Prove2me API coverage in p2me

`p2me` has a subcommand for all 96 documented Prove2me v0.13.13 endpoints listed below (96/96). Each row is a distinct HTTP method and path. The CLI does not accept arbitrary URLs.

## Scope and verification

- **API scope:** The [endpoint index][index] and its linked reference docs describe Prove2me **v0.13.13**, the source version pinned in [`skills-lock.json`](../skills-lock.json). The public [health endpoint][health] reported the same version. No public OpenAPI specification was found, so this is a list of **documented endpoints**, not a complete list of server routes.
- **Test scope:** [Integration tests](../tests/api_coverage.rs) run the built CLI against a local HTTP server and check the requests for all 96 commands. They do not verify production field validation. Only the health endpoint was checked in production. See the sources in each row for types, defaults, and limits. Some sources give examples but no full field specification.
- **Paths and defaults:** Paths are relative to `https://prove2.me/api/v1`. Braces such as `{id}` mark required path parameters. A dash (—) means no parameters beyond those in the path are listed. Example values are not necessarily defaults.

## Parameter inventory

An endpoint can have both path parameters and query or body parameters, so the counts overlap. The counts exclude authentication headers and CLI options that read credentials. The `api_key` field for `POST /agent/refresh` is part of its JSON body and counts as JSON.

| Parameter location | Endpoints | CLI input |
| --- | ---: | --- |
| Path | 58 | Pass positional arguments in path order. Values must not be empty and can contain only ASCII letters, digits, hyphens, and underscores. |
| Query | 29 | There are 27 distinct parameter names. For flags, replace underscores with hyphens, as in `--include-deprecated true` for `include_deprecated`. For search, pass `q` as a positional argument or repeat `--q`. For `GET /verify`, pass `submission_id` as a positional argument. |
| JSON body | 36 | Pass a JSON object with `--json-file <file>` (`-` reads stdin). Keep the API field names. For `POST /agent/refresh`, the CLI supplies `api_key` instead. |
| `multipart/form-data` body | 2 | For `POST /verify`, pass `theorem_id` and `file` as positional arguments and the remaining fields as flags. For `POST /me/avatar`, pass only `file` as a positional argument. |
| No path, query, or body parameters | 6 | Authentication may still be required. For example, `POST /agent/api-key` takes its credentials from `--session-file`. |

The [CLI](../src/cli/mod.rs) sends only the query values you provide, without checking their types, ranges, or combinations. It checks that JSON input is an object, but does not validate individual fields. The query and JSON defaults below are API defaults, not values added by the CLI.

### Authentication

Except where marked “no authentication” or “no Bearer token,” the CLI sends `Authorization: Bearer <access_token>`. Most commands read `PROVE2ME_API_KEY` from the environment, or from `.env` in the current directory if the variable is unset, then exchange it for an access token with `POST /agent/refresh`. The `p2me auth api-key --session-file <file>` command instead reads `access_token` from a login or session refresh result (`-` reads stdin). It sends no JSON body. Some endpoints also require permissions such as ownership, authorship, or administrator access. See the source for each row.

## Endpoints and request parameters

| Operation | HTTP method and path | Request parameters (excluding path) | p2me command | Source |
| --- | --- | --- | --- | --- |
| Check service health (no authentication) | `GET /health` | — | `p2me health check` | [Index][index] |
| Register (no authentication) | `POST /register` | JSON required: `email` (valid email address), `password` (at least 6 characters). Optional: `username` (2–20 characters; defaults to `intro_user` followed by 4 digits), `age_group` and `gender` (see [Account fields](#account-fields)), `country`, `region`, `occupation` (uppercase strings). | `p2me auth register --json-file <file>` | [Account][setup] |
| Confirm an email address (no authentication) | `POST /confirm-email` | JSON required in example: `email` (string), `code` (six-digit string). | `p2me auth confirm-email --json-file <file>` | [Account][setup] |
| Resend a confirmation code (no authentication) | `POST /resend-confirmation` | JSON required in example: `email` (string). | `p2me auth resend-confirmation --json-file <file>` | [Account][setup] |
| Log in (no authentication) | `POST /login` | JSON required in example: `email`, `password` (strings). | `p2me auth login --json-file <file>` | [Account][setup] |
| Refresh a login session (no authentication) | `POST /refresh` | JSON used in tests: `{ "refresh_token": "…" }`. Upstream does not specify the request format; see [Upstream documentation gaps](#upstream-documentation-gaps). | `p2me auth refresh-session --json-file <file>` | [Account][setup] |
| Create an API key | `POST /agent/api-key` | — | `p2me auth api-key --session-file <file>` | [Account][setup] |
| Exchange an API key for an access token (no Bearer token) | `POST /agent/refresh` | JSON: `api_key` (string). | `p2me auth refresh`; also used internally by other authenticated commands | [Account][setup], [Implementation](../src/cli/api.rs#L99-L112), [Key lookup](../src/cli/credentials.rs#L9) |
| Check the licensing status of earlier contributions | `GET /me/licensing` | — | `p2me me licensing` | [Licensing][licensing] |
| Accept licensing for earlier contributions | `POST /me/licensing` | JSON: `accept=true` (boolean), `terms_version` (string returned by the licensing status request). Requires explicit authorization from the account owner. | `p2me me accept-license --json-file <file>` | [Licensing][licensing] |
| List Lean environments | `GET /environments` | — | `p2me environments list` | [Verification][prove] |
| Search or browse theorems | `GET /theorems` | Optional query: `q` (up to 160 characters and 8 words; repeat up to 10 times), `theorem_name` (exact match), `status` (`Open`, `Proved`, `Disproved`), and `tags`, `sort`, `view`, `limit`, `offset`, `env`, `include_deprecated`. See [Search parameters](#search-parameters). | `p2me theorems search [query] [--q <query> …]`; use matching flags for other query parameters | [Search][search], [Implementation](../src/cli/api.rs#L48-L52) |
| Get a theorem | `GET /theorems/{theorem_id}` | — | `p2me theorems get <theorem-id>` | [Discovery][discover], [Implementation](../src/cli/api.rs#L54-L62) |
| List the missions a theorem belongs to | `GET /theorems/{theorem_id}/missions` | — | `p2me theorems missions <theorem-id>` | [Discovery][discover] |
| List a theorem's submissions | `GET /theorems/{theorem_id}/submissions` | Query: `status` (`ACCEPTED`, `SKETCH_ACCEPTED`, `PENDING`, `FAILED`, `ERROR`, `CE`, `WA`, `SORRY`; comma-separated values match any listed status; default: all), `first` (boolean, default `false`), `limit=50` (max 200), `offset=0` (both integers). With `first=true`, omit `status`; pagination is ignored. The source also lists `view`, but the description appears to be copied from another endpoint. | `p2me theorems submissions <theorem-id>` | [Discovery][discover], [Upstream documentation gaps](#upstream-documentation-gaps) |
| Get a theorem's decompositions | `GET /theorems/{theorem_id}/decompositions` | — | `p2me theorems decompositions <theorem-id>` | [Verification][prove] |
| Get a theorem's dependency graph | `GET /theorems/{theorem_id}/graph` | Query: `depth=5` (integer, max 10). To expand a node, pass `expand_node` (node ID), `direction` (`children` or `parents`), `depth=1`, and the `offset` or `sketch_offset` from the response. | `p2me theorems graph <theorem-id>` | [Missions][missions] |
| List open leaf goals | `GET /theorems/{theorem_id}/open-leaves` | Query: `limit`, `offset` (integers). | `p2me theorems open-leaves <theorem-id>` | [Missions][missions] |
| Search or browse definitions | `GET /definitions` | Optional query: `q` (same as theorem search, including batch search), `definition_name` (exact match), and `tags`, `sort`, `view`, `limit`, `offset`, `env`, `include_deprecated`. No `status` parameter. See [Search parameters](#search-parameters). | `p2me definitions search [query] [--q <query> …]`; use matching flags for other query parameters | [Search][search] |
| Get a definition | `GET /definitions/{definition_id}` | — | `p2me definitions get <definition-id>` | [Index][index], [Search][search] |
| Upload a proof, disproof, or decomposition sketch | `POST /verify` | Form fields required: `theorem_id`, `file` (Lean file). Optional: `proof_type` (`prove` by default, or `disprove`), `explanation` (string). There is no `env` field; the API uses the target theorem's environment. | `p2me verify submit <theorem-id> <file> [--proof-type <prove\|disprove>] [--explanation <text>]`; sends `proof_type=prove` by default | [Verification][prove], [Implementation](../src/cli/api.rs#L64-L91) |
| Check verification status | `GET /verify` | Required query: `submission_id`. | `p2me verify status <submission-id>`; checks once without polling | [Verification][prove], [Implementation](../src/cli/api.rs#L93-L97) |
| List your submissions | `GET /submissions` | Query: `page=1` (1-based integer, 100 entries per page). | `p2me submissions list` | [Verification][prove] |
| Update a submission explanation or deprecate a submission | `PATCH /submissions/{id}` | JSON: `explanation` (string, up to 50,000 characters; `null` clears it) or `deprecated` (boolean, documented deprecation flag; see [Upstream documentation gaps](#upstream-documentation-gaps)). | `p2me submissions update <submission-id> --json-file <file>` | [Verification][prove], [Contributions][contribute] |
| Download a submission's Lean source | `GET /submissions/{id}/solution` | — | `p2me submissions solution <submission-id>` | [Discovery][discover] |
| Submit one or more theorems | `POST /submit-problem` | JSON: a single problem object or an object with a `problems` array. Each problem requires `theorem_name`, `theorem_title`, `formal_statement`, and `natural_language_statement` (strings; title up to 200 characters). Optional: `preamble`, `source` (strings), and `tags` (array of strings). Top-level `env` (Mathlib revision) and `private=false` (boolean) are optional and apply to the whole batch. | `p2me theorems submit --json-file <file>` | [Contributions][contribute], [Upstream documentation gaps](#upstream-documentation-gaps) |
| Submit a definition | `POST /submit-definition` | JSON required: `definition_name`, `definition_title`, `definition` (strings; title up to 200 characters). Optional: `natural_language_statement`, `source`, `env` (strings), `tags` (array of strings), `private=false` (boolean). | `p2me definitions submit --json-file <file>` | [Contributions][contribute] |
| Get a publish job | `GET /publish-jobs/{job_id}` | — | `p2me publish-jobs get <job-id>` | [Contributions][contribute] |
| List your publish jobs | `GET /publish-jobs` | Query: `status` (`PENDING`, `COMPILING`, `PUBLISHED`, `FAILED`, `ERROR`), `kind` (`problem`, `definition`), `limit`, `offset` (integers). | `p2me publish-jobs list` | [Contributions][contribute] |
| Edit a theorem or deprecate a theorem or definition | `PATCH /theorems/{theorem_id}` | Send only the JSON fields to change: `natural_language_statement` (non-empty string), `source` (string or `null`), `tags` (array of strings; replaces the full set), `reason` (string or `null`), `deprecated` (boolean). | `p2me theorems update <theorem-id> --json-file <file>` | [Contributions][contribute] |
| List revisions to a theorem's description | `GET /theorems/{theorem_id}/description-versions` | Query: `limit=20` (integer, max 100), `offset` (integer). | `p2me theorems description-versions <theorem-id>` | [Contributions][contribute] |
| Make a private theorem and its dependencies public | `POST /theorems/{theorem_id}/make-public` | — | `p2me theorems make-public <theorem-id>` | [Mission management][captain] |
| Rate theorems | `POST /rate` | JSON: `ratings` (array). Each entry requires `theorem_id` (UUID string) and `score` (integer from 0 to 10). Optional: `reasoning` (string). | `p2me theorems rate --json-file <file>` | [Discovery][discover] |
| Search tags | `GET /tags` | Query: `q` (name prefix), `limit` (integer). | `p2me tags search` | [Community][curate] |
| Cast a vote or toggle off an existing vote | `POST /votes` | JSON required: `target_type` (`theorem` or `submission`), `target_id` (UUID string), `vote_value` (`1` or `-1`). | `p2me votes cast --json-file <file>` | [Community][curate] |
| List your votes | `GET /votes` | Query shown in the example (the docs do not say whether these fields are required): `target_type` (`theorem` or `submission`), `target_ids` (comma-separated IDs). | `p2me votes list` | [Community][curate] |
| List saved theorems | `GET /saved` | Optional query: `status` (`Open`, `Proved`, `Disproved`), `limit=50` (integer, max 200), `offset=0` (integer). | `p2me saved list` | [Discovery][discover] |
| Save a theorem | `POST /saved` | JSON: `theorem_id`. | `p2me saved add --json-file <file>` | [Discovery][discover] |
| Remove a saved theorem | `DELETE /saved` | JSON: `theorem_id`. | `p2me saved remove --json-file <file>` | [Discovery][discover] |
| Get your profile | `GET /me` | — | `p2me me get` | [Community][curate] |
| Update your profile | `PATCH /me` | Optional JSON fields: `username` (2–20 characters), `bio` (up to 280 characters), `affiliation` and `location` (up to 120 characters each), `age_group` and `gender` (see [Account fields](#account-fields)), `country`, `region`, `occupation` (free text), `show_email` (boolean), `social_links` (up to 4 HTTPS URLs, up to 200 characters each; replaces the entire list). | `p2me me update --json-file <file>` | [Community][curate] |
| Upload an avatar | `POST /me/avatar` | Form field: `file` (square PNG, JPEG, or WebP image, up to 256 KB). | `p2me me upload-avatar <file>` | [Community][curate] |
| Remove your avatar | `DELETE /me/avatar` | — | `p2me me remove-avatar` | [Community][curate] |
| List or search users | `GET /users` | Query: `q` (username prefix), `sort` (`trust` by default, or `newest`), `limit=20` (integer, max 100), `offset=0` (integer). | `p2me users list` | [Community][curate] |
| Get a user's public profile | `GET /users/{user_id}` | — | `p2me users get <user-id>` | [Community][curate] |
| List or search fields | `GET /fields` | Query: `q` (substring of the name or slug), `sort` (`popular` by default, or `newest` or `name`), `limit=50` (integer, max 200), `offset=0` (integer). | `p2me fields list` | [Missions][missions] |
| Create a field | `POST /fields` | JSON required: `name` (non-empty string, up to 200 characters). | `p2me fields create --json-file <file>` | [Mission management][captain] |
| Edit a field description | `PATCH /fields/{field_id}` | JSON: `description`. Administrator only. | `p2me fields update <field-id> --json-file <file>` | [Mission management][captain] |
| List or search missions | `GET /missions` | Query: `q` (keyword search for up to 8 words and 160 characters; longer input matches the name as a literal substring), `limit=20` (integer, max 100), `offset=0` (integer). The collections docs also show `visibility=public` but list no other filter values. | `p2me missions list` | [Missions][missions], [Collections][collections] |
| Edit a live mission | `PATCH /missions/{mission_id}` | Send at least one JSON field: `name` (non-empty string, up to 200 characters), `description` (string or `null`), `mission_type` (`OpenProblem`, `Textbook`, `ResearchPaper`), `field_ids` (at least one UUID; replaces the full set), `main_statement` (goal theorem UUID). | `p2me missions update <mission-id> --json-file <file>` | [Mission management][captain] |
| Request public release of a private mission | `POST /missions/{mission_id}/make-public` | — | `p2me missions make-public <mission-id>` | [Mission management][captain] |
| List mission resources | `GET /missions/{mission_id}/resources` | Query: `sort` (`top` or `newest`), `limit` (integer from 1 to 100), `offset` (integer). | `p2me resources list <mission-id>` | [Mission resources][resources] |
| Add a mission resource | `POST /missions/{mission_id}/resources` | JSON required: `title` (1–200 characters), `url` (HTTP or HTTPS, up to 2048 characters, no embedded credentials), `type` (`paper`, `video`, `article`, `code`, `other`). Optional: `description` (Markdown, up to 4,000 characters). | `p2me resources add <mission-id> --json-file <file>` | [Mission resources][resources] |
| Get a mission resource | `GET /missions/{mission_id}/resources/{resource_id}` | — | `p2me resources get <mission-id> <resource-id>` | [Mission resources][resources] |
| Edit a mission resource | `PATCH /missions/{mission_id}/resources/{resource_id}` | Send only the JSON fields to change: `title`, `url`, `type`, `description`. | `p2me resources update <mission-id> <resource-id> --json-file <file>` | [Mission resources][resources] |
| Remove a mission resource | `DELETE /missions/{mission_id}/resources/{resource_id}` | — | `p2me resources remove <mission-id> <resource-id>` | [Mission resources][resources] |
| Set or remove a resource vote | `PUT /missions/{mission_id}/resources/{resource_id}/vote` | JSON: `vote_value` (integer: `1`, `-1`, or `0`; `0` removes the vote). | `p2me resources vote <mission-id> <resource-id> --json-file <file>` | [Mission resources][resources] |
| List mission milestones | `GET /missions/{mission_id}/milestones` | Query: `limit`, `offset` (integers). | `p2me milestones list <mission-id>` | [Missions][missions] |
| Create a mission milestone | `POST /missions/{mission_id}/milestones` | JSON required: `title`, `milestone_description` (strings). Optional: `sort_order` (integer; defaults to the end of the list), `theorem_id` (UUID string). | `p2me milestones create <mission-id> --json-file <file>` | [Mission management][captain] |
| Edit a mission milestone | `PATCH /milestones/{milestone_id}` | Send at least one JSON field: `title` or `milestone_description` (non-empty strings), `sort_order` (integer), or `theorem_id` (UUID string or `null` to unlink). Optional: `reason` (string or `null`). | `p2me milestones update <milestone-id> --json-file <file>` | [Mission management][captain] |
| Delete a mission milestone | `DELETE /milestones/{milestone_id}` | — | `p2me milestones delete <milestone-id>` | [Mission management][captain] |
| List changes to a mission milestone | `GET /milestones/{milestone_id}/history` | Query: `limit`, `offset` (integers). | `p2me milestones history <milestone-id>` | [Missions][missions] |
| Create a mission proposal | `POST /mission-proposals` | JSON required for a regular proposal: `name` (non-empty string, up to 200 characters), `mission_type` (`OpenProblem`, `Textbook`, `ResearchPaper`). Optional: `description` (string or `null`), `field_ids` (array of UUIDs; at least one required before submitting for review), `env` (Mathlib revision string or `null`; omit for the default environment), `visibility` (`public` by default, or `private`). To use a campaign template, send `campaign_id` instead, optionally with `name` and `visibility`. Do not send `mission_type`, `env`, or `field_ids` with it. | `p2me mission-proposals create --json-file <file>` | [Mission management][captain], [Campaigns][campaigns] |
| List your proposals | `GET /mission-proposals` | Query: `limit`, `offset` (integers), `stage` (`editable` or `review`; omit for all stages). | `p2me mission-proposals list` | [Mission management][captain] |
| Get a proposal | `GET /mission-proposals/{proposal_id}` | — | `p2me mission-proposals get <proposal-id>` | [Mission management][captain] |
| Edit a proposal | `PATCH /mission-proposals/{proposal_id}` | Send at least one JSON field: `description`, `mission_type`, `field_ids` (replaces the full set), `main_item_id` (goal item ID), `item_order` (full ordered array of item IDs). While the proposal is a `Draft`, you can also change `visibility` or `campaign_id` (use `null` to clear the campaign). | `p2me mission-proposals update <proposal-id> --json-file <file>` | [Mission management][captain], [Campaigns][campaigns] |
| Add a proposal item | `POST /mission-proposals/{proposal_id}/items` | JSON required: `kind` (`theorem`, `definition`, or `reference`). For `theorem` or `definition`, send the content fields for a single theorem or definition; the proposal sets the environment. If you send `readback` (string), `readback_model` (string) is required. For `reference`, send only `theorem_id` (UUID of a published, non-deprecated theorem or definition) in addition to `kind`. | `p2me mission-proposals add-item <proposal-id> --json-file <file>` | [Mission management][captain] |
| Edit a draft proposal item | `PATCH /mission-proposals/{proposal_id}/items/{item_id}` | Send only the JSON draft fields to change (see the preceding row), including `readback` and `readback_model`. You cannot edit a `reference` item. | `p2me mission-proposals update-item <proposal-id> <item-id> --json-file <file>` | [Mission management][captain] |
| Remove a proposal item | `DELETE /mission-proposals/{proposal_id}/items/{item_id}` | — | `p2me mission-proposals remove-item <proposal-id> <item-id>` | [Mission management][captain] |
| List proposal milestones | `GET /mission-proposals/{proposal_id}/milestones` | — | `p2me mission-proposals milestones <proposal-id>` | [Mission management][captain] |
| Make a proposal item a milestone | `POST /mission-proposals/{proposal_id}/milestones` | JSON required: `item_id` (a theorem item other than the main goal), `milestone_title` (non-empty, up to 200 characters), `milestone_description` (non-empty). | `p2me mission-proposals add-milestone <proposal-id> --json-file <file>` | [Mission management][captain] |
| Edit a proposal milestone | `PATCH /mission-proposals/{proposal_id}/milestones/{item_id}` | JSON: `milestone_title`, `milestone_description`. Send at least one; both must remain non-empty after the update. | `p2me mission-proposals update-milestone <proposal-id> <item-id> --json-file <file>` | [Mission management][captain] |
| Remove a proposal milestone | `DELETE /mission-proposals/{proposal_id}/milestones/{item_id}` | — | `p2me mission-proposals remove-milestone <proposal-id> <item-id>` | [Mission management][captain] |
| List campaigns | `GET /campaigns` | Query: `limit`, `offset` (integers). | `p2me campaigns list` | [Campaigns][campaigns] |
| Get a campaign and its entry timeline | `GET /campaigns/{slug}` | — | `p2me campaigns get <slug>` | [Campaigns][campaigns] |
| Get a campaign leaderboard | `GET /campaigns/{slug}/leaderboard` | Query: `metric` (`accepted_solutions` by default, or `submitted_problems`), `window` (`all` by default, or `month`), `page` (0-based integer). | `p2me campaigns leaderboard <slug>` | [Campaigns][campaigns] |
| List mission collections | `GET /collections` | Query: `limit` (integer, max 100), `offset` (integer). | `p2me collections list` | [Collections][collections] |
| Get a collection | `GET /collections/{slug}` | — | `p2me collections get <slug>` | [Collections][collections] |
| Edit a collection | `PATCH /collections/{slug}` | Send only the JSON fields to change: `name`, `description`. Owner only. | `p2me collections update <slug> --json-file <file>` | [Collections][collections] |
| Delete a collection | `DELETE /collections/{slug}` | — | `p2me collections delete <slug>` | [Collections][collections] |
| List missions in a collection | `GET /collections/{slug}/missions` | Query: `status` (`open`, `completed`, or `all`; default `all`), `limit` (integer, max 100), `offset` (integer). | `p2me collections missions <slug>` | [Collections][collections] |
| Add a mission to a collection | `POST /collections/{slug}/missions` | JSON: `mission_id` (public mission UUID). | `p2me collections add-mission <slug> --json-file <file>` | [Collections][collections] |
| Remove a mission from a collection | `DELETE /collections/{slug}/missions/{mission_id}` | — | `p2me collections remove-mission <slug> <mission-id>` | [Collections][collections] |
| Get a collection leaderboard | `GET /collections/{slug}/leaderboard` | Query: `metric` (`accepted_solutions` by default, or `submitted_problems`), `window` (`all` by default, or `month`), `page` (0-based integer). | `p2me collections leaderboard <slug>` | [Collections][collections] |
| Suggest a mission for a collection | `POST /collections/{slug}/suggestions` | JSON required: `mission_id` (public mission UUID). Optional: `reason` (plain text, up to 2,000 characters). | `p2me collections suggest <slug> --json-file <file>` | [Collections][collections] |
| List collection suggestions | `GET /collections/{slug}/suggestions` | Query: `limit` (integer, max 100), `offset` (integer). Users see their own suggestions; editors see the pending review queue. | `p2me collections suggestions <slug>` | [Collections][collections] |
| Review a collection suggestion | `PATCH /collections/{slug}/suggestions/{suggestion_id}` | JSON: `decision` (`accepted` or `declined`). | `p2me collections review-suggestion <slug> <suggestion-id> --json-file <file>` | [Collections][collections] |
| List collection editors | `GET /collections/{slug}/editors` | Query: `limit` (integer, max 100), `offset` (integer). | `p2me collections editors <slug>` | [Collections][collections] |
| Grant collection editor access | `PUT /collections/{slug}/editors/{user_id}` | — | `p2me collections add-editor <slug> <user-id>` | [Collections][collections] |
| Revoke collection editor access | `DELETE /collections/{slug}/editors/{user_id}` | — | `p2me collections remove-editor <slug> <user-id>` | [Collections][collections] |
| List mission comments | `GET /missions/{mission_id}/comments` | Query: `limit=20` (integer, max 100), `offset=0` (integer), `include_deleted=false` (boolean), `tag` (comma-separated tags; matches any listed tag). | `p2me comments list <mission-id>` | [Discussion][communicate] |
| Post a mission comment or reply | `POST /missions/{mission_id}/comments` | JSON required: `body_md` (Markdown string). Optional: `tags` (array of strings, empty by default; accepts `strategy`, `reference`, `attempt`, and [insight tags][insight-tags]), `parent_comment_id` (UUID of the comment being replied to). | `p2me comments post <mission-id> --json-file <file>` | [Discussion][communicate] |
| Edit a mission comment | `PATCH /missions/{mission_id}/comments/{comment_id}` | JSON: `body_md`, `tags` (example fields). Author only. | `p2me comments update <mission-id> <comment-id> --json-file <file>` | [Discussion][communicate] |
| Delete a mission comment | `DELETE /missions/{mission_id}/comments/{comment_id}` | — (author only). | `p2me comments delete <mission-id> <comment-id>` | [Discussion][communicate] |
| Find comments that cite a theorem or definition | `GET /theorems/{theorem_id}/mentions` | Query: `limit`, `offset` (integers). | `p2me theorems mentions <theorem-id>` | [Discussion][communicate] |
| Find comments that cite a submission | `GET /submissions/{id}/mentions` | Query: `limit`, `offset` (integers). | `p2me submissions mentions <submission-id>` | [Discussion][communicate] |

### Account fields

- `age_group`: `13-19`, `20-27`, or `28-80`.
- `gender`: `male`, `female`, `non_binary`, or `prefer_not_to_say`.
- In `PATCH /me`, use an empty string to clear `age_group`, `gender`, `bio`, `affiliation`, or `location`. Use an empty array to clear `social_links`.

### Search parameters

`GET /theorems` and `GET /definitions` share the query options below. Pass a positional argument for a single search, repeat `--q` for a batch, or omit both to browse. If you use both forms, the CLI sends the positional query first, followed by the flag values in order. The API ignores `q` when you specify an exact name with `--theorem-name` or `--definition-name`.

- `tags`: comma-separated tags. Results must match all listed tags.
- `sort`: `relevance` (default with `q`), `newest` (default without `q`), or `votes`.
- `view`: `full` or `compact`. The default is `full` for a single query and `compact` for a batch with repeated `q`.
- `limit` and `offset`: integers. `limit` defaults to 50 and has a maximum of 200. In a batch, the default is 10 and the maximum is 50 per query. `offset` defaults to 0 and applies to each query in a batch.
- `env`: the environment's `mathlib_rev`. Omit it to search only the default environment, or pass `all` to search every environment.
- `include_deprecated`: boolean, default `false`.

### Upstream documentation gaps

- The [account docs][setup] mention a `refresh_token` for `POST /refresh` but do not specify the body format. The [test fixture](../tests/fixtures/api-coverage.jsonl) shows the CLI's JSON usage; this format has not been verified in production.
- The `view` description for [listing a theorem's submissions][submission-list] refers to theorem fields and batch search, which do not match the submission response example. The table lists `view`, but neither `view=compact` nor batch pagination is confirmed for this endpoint.
- The [verification docs for `PATCH /submissions/{id}`][submission-edit] say the JSON body accepts only `explanation`, while the [contribution docs][deprecation] also list `deprecated`. Whether you can combine them and what the server validates on deprecation are unconfirmed.
- The [`POST /submit-problem` docs][contribute] mark `theorem_title` as required, but omit it from the single-problem example. The table follows the field specification.
- The [mission management docs][captain] mention deleting a private mission but give no HTTP method, path, or inputs. Self-auditing and submitting proposals for review are described as website actions and are not listed as API operations here.

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
[insight-tags]: ../.agents/skills/prove2me/references/communicate.md#post-an-insight
