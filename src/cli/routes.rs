use reqwest::Method;

#[derive(Clone, Copy)]
pub enum Auth {
    Bearer,
    Public,
    ApiKey,
    Session,
}

#[derive(Clone, Copy)]
pub enum Input {
    None,
    Json,
    Search,
    Submit,
    Status,
    Avatar,
}

pub struct Route {
    pub name: &'static str,
    pub about: &'static str,
    pub method: Method,
    pub path: &'static str,
    pub auth: Auth,
    pub input: Input,
    pub query: &'static [&'static str],
}

pub struct Group {
    pub name: &'static str,
    pub about: &'static str,
    pub routes: &'static [Route],
}

// The URL placeholder names are also used as the positional argument labels in help.
macro_rules! op {
    ($name:literal, $about:literal, $method:ident $path:literal $(, $input:ident)? $(; $($query:literal),+)?) => {
        Route {
            name: $name,
            about: $about,
            method: Method::$method,
            path: $path,
            auth: Auth::Bearer,
            input: op!(@input $($input)?),
            query: &[$($($query),+)?],
        }
    };
    (@input) => { Input::None };
    (@input $input:ident) => { Input::$input };
}

macro_rules! auth {
    ($auth:ident, $($op:tt)*) => {
        Route { auth: Auth::$auth, ..op!($($op)*) }
    };
}

macro_rules! group {
    ($name:literal, $about:literal, $($route:expr),+ $(,)?) => {
        Group { name: $name, about: $about, routes: &[$($route),+] }
    };
}

pub static GROUPS: &[Group] = &[
    group!(
        "health",
        "Service status",
        auth!(Public, "check", "Check API health", GET "health"),
    ),
    group!(
        "auth",
        "Account and access tokens",
        auth!(Public, "register", "Register with an email address", POST "register", Json),
        auth!(Public, "confirm-email", "Confirm an email address", POST "confirm-email", Json),
        auth!(Public, "resend-confirmation", "Send a new confirmation code", POST "resend-confirmation", Json),
        auth!(Public, "login", "Start an email and password session", POST "login", Json),
        auth!(Public, "refresh-session", "Exchange a login refresh token", POST "refresh", Json),
        auth!(Session, "api-key", "Mint an API key using a login session", POST "agent/api-key"),
        auth!(ApiKey, "refresh", "Exchange the configured API key for an access token", POST "agent/refresh"),
    ),
    group!(
        "me",
        "Your Prove2me account",
        op!("get", "Get your profile", GET "me"),
        op!("update", "Update your profile", PATCH "me", Json),
        op!("upload-avatar", "Upload a PNG, JPEG, or WebP avatar", POST "me/avatar", Avatar),
        op!("remove-avatar", "Remove your avatar", DELETE "me/avatar"),
        op!("licensing", "Get the licensing status for earlier contributions", GET "me/licensing"),
        op!("accept-license", "Accept licensing for earlier contributions", POST "me/licensing", Json),
    ),
    group!(
        "environments",
        "Lean environments",
        op!("list", "List Lean and Mathlib environments", GET "environments"),
    ),
    group!(
        "theorems",
        "Find or manage theorems",
        op!("search", "Search or list theorems", GET "theorems", Search;
            "q", "theorem-name", "status", "tags", "sort", "view", "limit", "offset", "env", "include-deprecated"),
        op!("get", "Get the formal statement and details", GET "theorems/:THEOREM_ID"),
        op!("missions", "List a theorem's missions", GET "theorems/:THEOREM_ID/missions"),
        op!("submissions", "List submissions for a theorem", GET "theorems/:THEOREM_ID/submissions";
            "status", "first", "view", "limit", "offset"),
        op!("decompositions", "List a theorem's decompositions", GET "theorems/:THEOREM_ID/decompositions"),
        op!("graph", "Get a theorem's dependency graph", GET "theorems/:THEOREM_ID/graph";
            "depth", "expand-node", "direction", "offset", "sketch-offset"),
        op!("open-leaves", "List the open leaves of a theorem", GET "theorems/:THEOREM_ID/open-leaves";
            "limit", "offset"),
        op!("submit", "Submit one or more new theorems", POST "submit-problem", Json),
        op!("update", "Update or deprecate a theorem or definition", PATCH "theorems/:THEOREM_ID", Json),
        op!("description-versions", "List a theorem's description edits", GET "theorems/:THEOREM_ID/description-versions";
            "limit", "offset"),
        op!("make-public", "Permanently publish a theorem and its dependencies", POST "theorems/:THEOREM_ID/make-public"),
        op!("rate", "Rate theorems", POST "rate", Json),
        op!("mentions", "Find discussions that cite a theorem or definition", GET "theorems/:THEOREM_ID/mentions";
            "limit", "offset"),
    ),
    group!(
        "definitions",
        "Find or submit definitions",
        op!("search", "Search or list definitions", GET "definitions", Search;
            "q", "definition-name", "tags", "sort", "view", "limit", "offset", "env", "include-deprecated"),
        op!("get", "Get a definition", GET "definitions/:DEFINITION_ID"),
        op!("submit", "Submit a new definition", POST "submit-definition", Json),
    ),
    group!(
        "verify",
        "Submit a Lean file or check a verification result",
        op!("submit", "Upload a Lean file without waiting for verification", POST "verify", Submit),
        op!("status", "Fetch the current verification status once", GET "verify", Status),
    ),
    group!(
        "submissions",
        "Inspect or update proof submissions",
        op!("list", "List your submissions", GET "submissions"; "page"),
        op!("update", "Edit a submission's explanation or deprecate the submission", PATCH "submissions/:SUBMISSION_ID", Json),
        op!("solution", "Fetch a submission's Lean source", GET "submissions/:SUBMISSION_ID/solution"),
        op!("mentions", "Find discussions that cite a submission", GET "submissions/:SUBMISSION_ID/mentions";
            "limit", "offset"),
    ),
    group!(
        "publish-jobs",
        "Track publication of theorems and definitions",
        op!("get", "Get a publish job", GET "publish-jobs/:JOB_ID"),
        op!("list", "List your publish jobs", GET "publish-jobs"; "status", "kind", "limit", "offset"),
    ),
    group!(
        "tags",
        "Find theorem and definition tags",
        op!("search", "Search tags by prefix", GET "tags"; "q", "limit"),
    ),
    group!(
        "votes",
        "Vote on theorems and submissions",
        op!("cast", "Cast or toggle a vote", POST "votes", Json),
        op!("list", "List your votes for selected targets", GET "votes"; "target-type", "target-ids"),
    ),
    group!(
        "saved",
        "Bookmark theorems",
        op!("list", "List saved theorems", GET "saved"; "status", "limit", "offset"),
        op!("add", "Save a theorem", POST "saved", Json),
        op!("remove", "Remove a saved theorem", DELETE "saved", Json),
    ),
    group!(
        "users",
        "Find users and read public profiles",
        op!("list", "List or search users", GET "users"; "q", "sort", "limit", "offset"),
        op!("get", "Get a user's public profile", GET "users/:USER_ID"),
    ),
    group!(
        "fields",
        "Browse or manage mathematical fields",
        op!("list", "List or search fields", GET "fields"; "q", "sort", "limit", "offset"),
        op!("create", "Create a field", POST "fields", Json),
        op!("update", "Update a field description", PATCH "fields/:FIELD_ID", Json),
    ),
    group!(
        "missions",
        "Browse or manage missions",
        op!("list", "List or search missions", GET "missions"; "q", "limit", "offset", "visibility"),
        op!("update", "Update an existing mission", PATCH "missions/:MISSION_ID", Json),
        op!("make-public", "Publish a mission's content and request a public listing", POST "missions/:MISSION_ID/make-public"),
    ),
    group!(
        "resources",
        "Browse or manage mission resources",
        op!("list", "List mission resources", GET "missions/:MISSION_ID/resources"; "sort", "limit", "offset"),
        op!("add", "Add a resource to a mission", POST "missions/:MISSION_ID/resources", Json),
        op!("get", "Get a mission resource", GET "missions/:MISSION_ID/resources/:RESOURCE_ID"),
        op!("update", "Update a mission resource", PATCH "missions/:MISSION_ID/resources/:RESOURCE_ID", Json),
        op!("remove", "Remove a mission resource", DELETE "missions/:MISSION_ID/resources/:RESOURCE_ID"),
        op!("vote", "Set or remove your vote on a resource", PUT "missions/:MISSION_ID/resources/:RESOURCE_ID/vote", Json),
    ),
    group!(
        "milestones",
        "Browse or manage mission milestones",
        op!("list", "List mission milestones", GET "missions/:MISSION_ID/milestones"; "limit", "offset"),
        op!("create", "Create a mission milestone", POST "missions/:MISSION_ID/milestones", Json),
        op!("update", "Update a mission milestone", PATCH "milestones/:MILESTONE_ID", Json),
        op!("delete", "Delete a mission milestone", DELETE "milestones/:MILESTONE_ID"),
        op!("history", "List milestone edits", GET "milestones/:MILESTONE_ID/history"; "limit", "offset"),
    ),
    group!(
        "mission-proposals",
        "Create and manage mission proposal drafts",
        op!("create", "Create a mission or campaign proposal", POST "mission-proposals", Json),
        op!("list", "List your proposals", GET "mission-proposals"; "limit", "offset", "stage"),
        op!("get", "Get a proposal", GET "mission-proposals/:PROPOSAL_ID"),
        op!("update", "Update a proposal", PATCH "mission-proposals/:PROPOSAL_ID", Json),
        op!("add-item", "Add a draft or reference item", POST "mission-proposals/:PROPOSAL_ID/items", Json),
        op!("update-item", "Update a draft item", PATCH "mission-proposals/:PROPOSAL_ID/items/:ITEM_ID", Json),
        op!("remove-item", "Remove a proposal item", DELETE "mission-proposals/:PROPOSAL_ID/items/:ITEM_ID"),
        op!("milestones", "List proposal milestones", GET "mission-proposals/:PROPOSAL_ID/milestones"),
        op!("add-milestone", "Mark a proposal item as a milestone", POST "mission-proposals/:PROPOSAL_ID/milestones", Json),
        op!("update-milestone", "Update a proposal milestone", PATCH "mission-proposals/:PROPOSAL_ID/milestones/:ITEM_ID", Json),
        op!("remove-milestone", "Remove a proposal milestone", DELETE "mission-proposals/:PROPOSAL_ID/milestones/:ITEM_ID"),
    ),
    group!(
        "campaigns",
        "Browse campaigns and rankings",
        op!("list", "List campaigns", GET "campaigns"; "limit", "offset"),
        op!("get", "Get a campaign and its entry timeline", GET "campaigns/:SLUG"),
        op!("leaderboard", "Get the campaign leaderboard", GET "campaigns/:SLUG/leaderboard";
            "metric", "window", "page"),
    ),
    group!(
        "collections",
        "Browse or manage mission collections",
        op!("list", "List collections", GET "collections"; "limit", "offset"),
        op!("get", "Get a collection", GET "collections/:SLUG"),
        op!("update", "Update collection details", PATCH "collections/:SLUG", Json),
        op!("delete", "Delete a collection", DELETE "collections/:SLUG"),
        op!("missions", "List missions in a collection", GET "collections/:SLUG/missions"; "status", "limit", "offset"),
        op!("add-mission", "Add a mission to a collection", POST "collections/:SLUG/missions", Json),
        op!("remove-mission", "Remove a mission from a collection", DELETE "collections/:SLUG/missions/:MISSION_ID"),
        op!("leaderboard", "Get the collection leaderboard", GET "collections/:SLUG/leaderboard";
            "metric", "window", "page"),
        op!("suggest", "Suggest adding a mission to a collection", POST "collections/:SLUG/suggestions", Json),
        op!("suggestions", "List collection suggestions", GET "collections/:SLUG/suggestions"; "limit", "offset"),
        op!("review-suggestion", "Accept or decline a suggestion", PATCH "collections/:SLUG/suggestions/:SUGGESTION_ID", Json),
        op!("editors", "List collection editors", GET "collections/:SLUG/editors"; "limit", "offset"),
        op!("add-editor", "Grant collection editor access", PUT "collections/:SLUG/editors/:USER_ID"),
        op!("remove-editor", "Revoke collection editor access", DELETE "collections/:SLUG/editors/:USER_ID"),
    ),
    group!(
        "comments",
        "Read or manage mission discussions",
        op!("list", "List mission comments", GET "missions/:MISSION_ID/comments";
            "limit", "offset", "include-deleted", "tag"),
        op!("post", "Post a comment or reply", POST "missions/:MISSION_ID/comments", Json),
        op!("update", "Edit a comment", PATCH "missions/:MISSION_ID/comments/:COMMENT_ID", Json),
        op!("delete", "Delete a comment", DELETE "missions/:MISSION_ID/comments/:COMMENT_ID"),
    ),
];
