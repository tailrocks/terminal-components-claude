use super::{Bin, Root, interact};

const BOOT: &str = "Junie Design system";

const GRID_EDITS: &[&str] = &[
    "tab",
    "tab",
    "right",
    "right",
    "right",
    "enter",
    "ctrl-l",
    "type:600",
    "enter",
    "down",
    "enter",
    "ctrl-l",
    "type:12",
    "enter",
    "wait:2 pending",
];

#[rustfmt::skip]
pub const SURFACES: &[Root] = &[

    Root::new("showcase/pages/overview", Bin::Showcase, &["--page", "overview"], BOOT),
    Root::new("showcase/pages/lists", Bin::Showcase, &["--page", "lists"], BOOT),
    Root::new("showcase/pages/trees", Bin::Showcase, &["--page", "trees"], BOOT),
    Root::new("showcase/pages/tables", Bin::Showcase, &["--page", "tables"], BOOT),
    Root::new("showcase/pages/editable", Bin::Showcase, &["--page", "editabletables"], BOOT),
    Root::new("showcase/pages/panels", Bin::Showcase, &["--page", "panels"], BOOT),
    Root::new("showcase/pages/sidebars", Bin::Showcase, &["--page", "sidebars"], BOOT),

    Root::new("showcase/sections/sidebars-options", Bin::Showcase, &["--page", "sidebars"], BOOT).sends(&["tab", "2", "wait:Rail when hidden"]).pending(),
    Root::new("showcase/sections/sidebars-states", Bin::Showcase, &["--page", "sidebars"], BOOT).sends(&["tab", "3", "wait:focus + hover"]).pending(),
    Root::new("showcase/flows/sidebars/collapsed", Bin::Showcase, &["--page", "sidebars", "--opt", "sidebars.collapsed=on"], BOOT).sends(&["wait:Collapse keeps rows"]).pending(),
    Root::new("showcase/flows/sidebars/hidden", Bin::Showcase, &["--page", "sidebars", "--opt", "sidebars.hidden=on"], BOOT).sends(&["wait:Collapse keeps rows"]).pending(),
    Root::new("showcase/pages/dialogs", Bin::Showcase, &["--page", "dialogs"], BOOT),
    Root::new("showcase/pages/codeeditor", Bin::Showcase, &["--page", "codeeditor"], BOOT),


    Root::new("showcase/sections/codeeditor-options", Bin::Showcase, &["--page", "codeeditor"], BOOT).sends(&["tab", "2", "wait:Command keys"]).pending(),
    Root::new("showcase/sections/codeeditor-states", Bin::Showcase, &["--page", "codeeditor"], BOOT).sends(&["tab", "3", "wait:expected an item"]).pending(),
    Root::new("showcase/flows/codeeditor/read-only", Bin::Showcase, &["--page", "codeeditor", "--opt", "codeeditor.read_only=on"], BOOT).sends(&["wait:read-only"]).pending(),
    Root::new("showcase/flows/codeeditor/plain", Bin::Showcase, &["--page", "codeeditor", "--opt", "codeeditor.highlighter=off", "--opt", "codeeditor.segmenter=off"], BOOT).sends(&["wait:0 blocks"]).pending(),
    Root::new("showcase/pages/datagrid", Bin::Showcase, &["--page", "datagrid"], BOOT),
    Root::new("showcase/pages/chips", Bin::Showcase, &["--page", "chipsselects"], BOOT),


    Root::new("showcase/sections/chips-options", Bin::Showcase, &["--page", "chipsselects"], BOOT).sends(&["tab", "2", "wait:Chip bar width"]).pending(),
    Root::new("showcase/sections/chips-states", Bin::Showcase, &["--page", "chipsselects"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/flows/chips/overflow", Bin::Showcase, &["--page", "chipsselects", "--opt", "chipsselects.bar_width=40"], BOOT).pending(),
    Root::new("showcase/flows/chips/select_above", Bin::Showcase, &["--page", "chipsselects", "--opt", "chipsselects.placement=above"], BOOT).sends(&["tab", "tab", "tab", "tab", "enter", "wait:customer"]).pending(),
    Root::new("showcase/pages/pickers", Bin::Showcase, &["--page", "pickers"], BOOT),
    Root::new("showcase/pages/chrome", Bin::Showcase, &["--page", "chrome", "--motion", "paused"], BOOT),
    Root::new("showcase/pages/settings", Bin::Showcase, &["--page", "settings"], BOOT),


    Root::new("showcase/sections/settings-options", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "2", "wait:Confirm on save"]).pending(),
    Root::new("showcase/sections/settings-states", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "3", "wait:Section tabs"]).pending(),
    Root::new("showcase/flows/settings/environment-single", Bin::Showcase, &["--page", "settings", "--opt", "settings.tab=environment", "--opt", "settings.env_select=single"], BOOT).sends(&["wait:DATABASE_URL"]).pending(),
    Root::new("showcase/pages/taskrunner", Bin::Showcase, &["--page", "taskrunner"], BOOT),

    Root::new("showcase/sections/taskrunner-options", Bin::Showcase, &["--page", "taskrunner"], BOOT).sends(&["tab", "2", "wait:Log wraps"]).pending(),
    Root::new("showcase/sections/taskrunner-states", Bin::Showcase, &["--page", "taskrunner"], BOOT).sends(&["tab", "3", "wait:Task bar"]).pending(),
    Root::new("showcase/flows/taskrunner/plain-tree", Bin::Showcase, &["--page", "taskrunner", "--opt", "taskrunner.env_glyphs=off", "--opt", "taskrunner.tree_meta=on"], BOOT).sends(&["wait:shared-libs"]).pending(),


    Root::new("showcase/pages/scrolling", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["wait:739.63s"]),


    Root::new("showcase/pages/terminal", Bin::Showcase, &["--page", "terminal"], BOOT).sends(&["wait:✓ Ready"]).timeout(60000),


    Root::new("showcase/sections/diff-options", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "2", "wait:Page width"]).pending(),
    Root::new("showcase/sections/diff-states", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "3", "wait:No file selected"]).pending(),
    Root::new("showcase/flows/diff/narrow", Bin::Showcase, &["--page", "diff", "--opt", "diff.width=narrow"], BOOT).sends(&["wait:Changes"]).pending(),
    Root::new("showcase/flows/diff/renamed", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "tab", "tab", "tab", "end", "wait:use super::exponential"]).pending(),
    Root::new("showcase/sections/terminal-options", Bin::Showcase, &["--page", "terminal", "--motion", "paused"], BOOT).sends(&["tab", "2", "wait:Find marks"]).pending(),
    Root::new("showcase/sections/terminal-states", Bin::Showcase, &["--page", "terminal", "--motion", "paused"], BOOT).sends(&["tab", "3", "wait:Step rail"]).pending(),
    Root::new("showcase/flows/terminal/marks", Bin::Showcase, &["--page", "terminal", "--motion", "paused", "--opt", "terminal.mark=on"], BOOT).sends(&["wait:jackin launch"]).pending(),
    Root::new("showcase/flows/terminal/retention", Bin::Showcase, &["--page", "terminal", "--opt", "terminal.max_lines=20"], BOOT).sends(&["wait:last 20 kept"]).timeout(60000).pending(),



    Root::new("showcase/pages/selectable", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno"]),








    Root::new("showcase/pages/progress", Bin::Showcase, &["--page", "progress", "--motion", "paused"], BOOT),


    Root::new("showcase/pages/celleditors", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["wait:checkout-api"]).pending(),


    Root::new("showcase/pages/columns", Bin::Showcase, &["--page", "columns"], BOOT).sends(&["wait:Width solver"]).pending(),



    Root::new("showcase/pages/queryworkbench", Bin::Showcase, &["--page", "queryworkbench"], BOOT).sends(&["wait:Northwind Traders"]).pending(),
    Root::new("showcase/sections/queryworkbench-options", Bin::Showcase, &["--page", "queryworkbench"], BOOT).sends(&["tab", "2", "wait:Environment"]).pending(),
    Root::new("showcase/sections/queryworkbench-states", Bin::Showcase, &["--page", "queryworkbench"], BOOT).sends(&["tab", "3", "wait:Query tabs"]).pending(),
    Root::new("showcase/flows/queryworkbench/error", Bin::Showcase, &["--page", "queryworkbench", "--opt", "queryworkbench.result=error"], BOOT).sends(&["wait:does not exist"]).pending(),
    Root::new("showcase/flows/queryworkbench/plan", Bin::Showcase, &["--page", "queryworkbench", "--opt", "queryworkbench.result=plan"], BOOT).sends(&["wait:Index Scan"]).pending(),



    Root::new("showcase/pages/connections", Bin::Showcase, &["--page", "connections"], BOOT).sends(&["wait:prod-db-1.acme.io"]).pending(),
    Root::new("showcase/sections/connections-options", Bin::Showcase, &["--page", "connections"], BOOT).sends(&["tab", "2", "wait:Connect state"]).pending(),
    Root::new("showcase/sections/connections-states", Bin::Showcase, &["--page", "connections"], BOOT).sends(&["tab", "3", "wait:Delete…"]).pending(),
    Root::new("showcase/flows/connections/failed", Bin::Showcase, &["--page", "connections", "--opt", "connections.state=failed"], BOOT).sends(&["wait:Reconnect"]).pending(),
    Root::new("showcase/flows/connections/empty", Bin::Showcase, &["--page", "connections", "--opt", "connections.list=empty"], BOOT).sends(&["wait:No connections"]).pending(),



    Root::new("showcase/pages/finder", Bin::Showcase, &["--page", "finder"], BOOT).sends(&["wait:git pull --ff-only"]).pending(),
    Root::new("showcase/sections/finder-options", Bin::Showcase, &["--page", "finder"], BOOT).sends(&["tab", "2", "wait:Risk column"]).pending(),
    Root::new("showcase/sections/finder-states", Bin::Showcase, &["--page", "finder"], BOOT).sends(&["tab", "3", "wait:No matches"]).pending(),
    Root::new("showcase/flows/finder/nomatch", Bin::Showcase, &["--page", "finder", "--opt", "finder.query=nomatch"], BOOT).sends(&["wait:No matches"]).pending(),
    Root::new("showcase/flows/finder/line", Bin::Showcase, &["--page", "finder", "--opt", "finder.preview=line"], BOOT).sends(&["wait:→ git pull"]).pending(),




    Root::new("showcase/pages/statushints", Bin::Showcase, &["--page", "statushints"], BOOT).sends(&["wait:HintBar::resolve"]).pending(),
    Root::new("showcase/pages/breadcrumbsrails", Bin::Showcase, &["--page", "breadcrumbsrails"], BOOT).sends(&["wait:RailAction::Expand"]).pending(),
    Root::new("showcase/pages/helpkeys", Bin::Showcase, &["--page", "helpkeys"], BOOT).sends(&["wait:HelpCmd::Close"]).pending(),
    Root::new("showcase/pages/quitleave", Bin::Showcase, &["--page", "quitleave"], BOOT).sends(&["wait:QuitKeys::classify"]).pending(),

    Root::new("showcase/pages/checksradios", Bin::Showcase, &["--page", "checksradios"], BOOT).sends(&["wait:Select all rows"]).pending(),
    Root::new("showcase/sections/checksradios-options", Bin::Showcase, &["--page", "checksradios"], BOOT).sends(&["tab", "2", "wait:Checkbox click target"]).pending(),
    Root::new("showcase/sections/checksradios-states", Bin::Showcase, &["--page", "checksradios"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/elements/checks/row-hover", Bin::Showcase, &["--page", "checksradios", "--opt", "checksradios.hit=row"], BOOT).hover("Include archived projects").pending(),

    Root::new("showcase/pages/filebrowser", Bin::Showcase, &["--page", "filebrowser"], BOOT).sends(&["wait:New mount"]).pending(),
    Root::new("showcase/sections/filebrowser-options", Bin::Showcase, &["--page", "filebrowser"], BOOT).sends(&["tab", "2", "wait:Option check box"]).pending(),
    Root::new("showcase/sections/filebrowser-states", Bin::Showcase, &["--page", "filebrowser"], BOOT).sends(&["tab", "3", "wait:Hidden toggle"]).pending(),

    Root::new("showcase/pages/emptyerror", Bin::Showcase, &["--page", "emptyerror"], BOOT).sends(&["wait:No connections yet"]).pending(),
    Root::new("showcase/sections/emptyerror-options", Bin::Showcase, &["--page", "emptyerror"], BOOT).sends(&["tab", "2", "wait:Read-only reason"]).pending(),
    Root::new("showcase/sections/emptyerror-states", Bin::Showcase, &["--page", "emptyerror"], BOOT).sends(&["tab", "3", "wait:With actions"]).pending(),
    Root::new("showcase/flows/emptyerror/error", Bin::Showcase, &["--page", "emptyerror", "--opt", "emptyerror.kind=error"], BOOT).sends(&["wait:Could not load connections"]).pending(),
    Root::new("showcase/elements/emptyerror/action-hover", Bin::Showcase, &["--page", "emptyerror"], BOOT).hover("New connection").pending(),

    Root::new("showcase/sections/progress-options", Bin::Showcase, &["--page", "progress", "--motion", "paused"], BOOT).sends(&["tab", "2", "wait:Label column"]).pending(),
    Root::new("showcase/sections/progress-states", Bin::Showcase, &["--page", "progress", "--motion", "paused"], BOOT).sends(&["tab", "3", "wait:Meter stale"]).pending(),
    Root::new("showcase/flows/progress/measured-labels", Bin::Showcase, &["--page", "progress", "--motion", "paused", "--opt", "progress.label_width=measured", "--opt", "progress.reserve_value=off"], BOOT).sends(&["wait:Warm caches"]).pending(),


    Root::new("showcase/pages/meters", Bin::Showcase, &["--page", "meters"], BOOT).sends(&["wait:Session (5 h)"]).pending(),
    Root::new("showcase/sections/meters-options", Bin::Showcase, &["--page", "meters"], BOOT).sends(&["tab", "2", "wait:Quota meter visual"]).pending(),
    Root::new("showcase/sections/meters-states", Bin::Showcase, &["--page", "meters"], BOOT).sends(&["tab", "3", "wait:Disk list"]).pending(),
    Root::new("showcase/flows/meters/offline", Bin::Showcase, &["--page", "meters", "--opt", "meters.offline=on"], BOOT).sends(&["wait:last-good values"]).pending(),
    Root::new("showcase/flows/meters/narrow-line", Bin::Showcase, &["--page", "meters", "--opt", "meters.label_width=8", "--opt", "meters.visual=line"], BOOT).sends(&["wait:Session…"]).pending(),


    Root::new("showcase/pages/notices", Bin::Showcase, &["--page", "notices"], BOOT).sends(&["wait:Authentication failed"]).pending(),
    Root::new("showcase/sections/notices-options", Bin::Showcase, &["--page", "notices"], BOOT).sends(&["tab", "2", "wait:Result notice kind"]).pending(),
    Root::new("showcase/sections/notices-states", Bin::Showcase, &["--page", "notices"], BOOT).sends(&["tab", "3", "wait:With hint"]).pending(),
    Root::new("showcase/flows/notices/narrow", Bin::Showcase, &["--page", "notices", "--opt", "notices.width=narrow"], BOOT).sends(&["wait:Authentication failed"]).pending(),
    Root::new("showcase/flows/notices/warning", Bin::Showcase, &["--page", "notices", "--opt", "notices.kind=warning"], BOOT).sends(&["wait:syntax error"]).pending(),


    Root::new("showcase/pages/splitsstacks", Bin::Showcase, &["--page", "splitsstacks"], BOOT).sends(&["wait:Three panes"]).pending(),
    Root::new("showcase/sections/splitsstacks-options", Bin::Showcase, &["--page", "splitsstacks"], BOOT).sends(&["tab", "2", "wait:Seam double-click"]).pending(),
    Root::new("showcase/sections/splitsstacks-states", Bin::Showcase, &["--page", "splitsstacks"], BOOT).sends(&["tab", "3", "wait:Horizontal seam"]).pending(),
    Root::new("showcase/flows/splitsstacks/zoomed", Bin::Showcase, &["--page", "splitsstacks", "--opt", "splitsstacks.maximized=second"], BOOT).sends(&["wait:output zoomed"]).pending(),
    Root::new("showcase/flows/splitsstacks/vertical", Bin::Showcase, &["--page", "splitsstacks", "--opt", "splitsstacks.dir=vertical"], BOOT).sends(&["wait:Three panes"]).pending(),
    Root::new("showcase/flows/splitsstacks/stack-scrolls", Bin::Showcase, &["--page", "splitsstacks", "--opt", "splitsstacks.min_child_rows=3"], BOOT).sends(&["wait:not in view"]).pending(),
    Root::new("showcase/pages/masterdetail", Bin::Showcase, &["--page", "masterdetail"], BOOT).sends(&["wait:Database browser"]).pending(),
    Root::new("showcase/sections/masterdetail-options", Bin::Showcase, &["--page", "masterdetail"], BOOT).sends(&["tab", "2", "wait:Collapses below"]).pending(),
    Root::new("showcase/sections/masterdetail-states", Bin::Showcase, &["--page", "masterdetail"], BOOT).sends(&["tab", "3", "wait:Master-detail"]).pending(),
    Root::new("showcase/flows/masterdetail/narrow", Bin::Showcase, &["--page", "masterdetail", "--opt", "masterdetail.width=72"], BOOT).sends(&["wait:Tab shows details"]).pending(),
    Root::new("showcase/flows/masterdetail/share", Bin::Showcase, &["--page", "masterdetail", "--opt", "masterdetail.size=share"], BOOT).sends(&["wait:Database browser"]).pending(),
    Root::new("showcase/pages/panetree", Bin::Showcase, &["--page", "panetree"], BOOT).sends(&["wait:Workspace panes"]).pending(),
    Root::new("showcase/sections/panetree-options", Bin::Showcase, &["--page", "panetree"], BOOT).sends(&["tab", "2", "wait:Starting layout"]).pending(),
    Root::new("showcase/sections/panetree-states", Bin::Showcase, &["--page", "panetree"], BOOT).sends(&["tab", "3", "wait:Pane tree"]).pending(),
    Root::new("showcase/flows/panetree/grid", Bin::Showcase, &["--page", "panetree", "--opt", "panetree.layout=grid"], BOOT).sends(&["wait:4 panes"]).pending(),
    Root::new("showcase/flows/panetree/unframed", Bin::Showcase, &["--page", "panetree", "--opt", "panetree.frames=never"], BOOT).sends(&["wait:Workspace panes"]).pending(),
    Root::new("showcase/pages/textutilities", Bin::Showcase, &["--page", "textutilities"], BOOT).sends(&["wait:TextBuffer"]).pending(),


    Root::new("showcase/pages/diskcleanup", Bin::Showcase, &["--page", "diskcleanup"], BOOT).sends(&["wait:Cleanup categories"]).pending(),
    Root::new("showcase/sections/diskcleanup-options", Bin::Showcase, &["--page", "diskcleanup"], BOOT).sends(&["tab", "2", "wait:Typed phrase"]).pending(),
    Root::new("showcase/sections/diskcleanup-states", Bin::Showcase, &["--page", "diskcleanup"], BOOT).sends(&["tab", "3", "wait:Usage tree"]).pending(),
    Root::new("showcase/flows/diskcleanup/scanning", Bin::Showcase, &["--page", "diskcleanup", "--motion", "paused", "--opt", "diskcleanup.scan=scanning"], BOOT).sends(&["wait:scanning ~/work"]).pending(),
    Root::new("showcase/flows/diskcleanup/permanent", Bin::Showcase, &["--page", "diskcleanup", "--opt", "diskcleanup.mode=permanent"], BOOT).sends(&["wait:permanent delete"]).pending(),
    Root::new("showcase/flows/diskcleanup/drawer", Bin::Showcase, &["--page", "diskcleanup", "--opt", "diskcleanup.collapse_below=108"], BOOT).sends(&["wait:Cleanup categories", "tab", "tab", "tab", "wait:Freshness"]).pending(),


    Root::new("showcase/pages/activity", Bin::Showcase, &["--page", "activity", "--motion", "paused"], BOOT).sends(&["wait:following"]).pending(),
    Root::new("showcase/sections/activity-options", Bin::Showcase, &["--page", "activity", "--motion", "paused"], BOOT).sends(&["tab", "2", "wait:Retained lines"]).pending(),
    Root::new("showcase/sections/activity-states", Bin::Showcase, &["--page", "activity", "--motion", "paused"], BOOT).sends(&["tab", "3", "wait:Streams"]).pending(),
    Root::new("showcase/flows/activity/failed-find", Bin::Showcase, &["--page", "activity", "--opt", "activity.state=failed", "--opt", "activity.find=error"], BOOT).sends(&["wait:Free the port"]).pending(),
    Root::new("showcase/flows/activity/stopped-hidden-stream", Bin::Showcase, &["--page", "activity", "--opt", "activity.state=stopped"], BOOT).sends(&["wait:stopped · exit 143", "tab", "tab", "3", "wait:3 of 4 streams"]).pending(),


    Root::new("showcase/pages/workspaces", Bin::Showcase, &["--page", "workspaces"], BOOT).sends(&["wait:daemon · 3 s ago"]).pending(),
    Root::new("showcase/sections/workspaces-options", Bin::Showcase, &["--page", "workspaces"], BOOT).sends(&["tab", "2", "wait:Instance roster"]).pending(),
    Root::new("showcase/sections/workspaces-states", Bin::Showcase, &["--page", "workspaces"], BOOT).sends(&["tab", "3", "wait:Workspace tree"]).pending(),
    Root::new("showcase/flows/workspaces/drawer", Bin::Showcase, &["--page", "workspaces", "--opt", "workspaces.collapse_below=108"], BOOT).sends(&["wait:Tab shows details", "tab", "tab", "tab", "wait:Esc back"]).pending(),


    Root::new("showcase/pages/accountsusage", Bin::Showcase, &["--page", "accountsusage"], BOOT).sends(&["wait:last 7 days"]).pending(),
    Root::new("showcase/sections/accountsusage-options", Bin::Showcase, &["--page", "accountsusage"], BOOT).sends(&["tab", "2", "wait:Account warnings"]).pending(),
    Root::new("showcase/sections/accountsusage-states", Bin::Showcase, &["--page", "accountsusage"], BOOT).sends(&["tab", "3", "wait:Account warning"]).pending(),
    Root::new("showcase/flows/accountsusage/narrow", Bin::Showcase, &["--page", "accountsusage", "--opt", "accountsusage.width=72"], BOOT).sends(&["wait:Tab shows details"]).pending(),


    Root::new("showcase/pages/launchcockpit", Bin::Showcase, &["--page", "launchcockpit"], BOOT).sends(&["wait:not started"]).pending(),
    Root::new("showcase/sections/launchcockpit-options", Bin::Showcase, &["--page", "launchcockpit"], BOOT).sends(&["tab", "2", "wait:Launch outcome"]).pending(),
    Root::new("showcase/sections/launchcockpit-states", Bin::Showcase, &["--page", "launchcockpit"], BOOT).sends(&["tab", "3", "wait:Failed launch"]).pending(),
    Root::new("showcase/flows/launchcockpit/numbered", Bin::Showcase, &["--page", "launchcockpit", "--opt", "launchcockpit.numbered=on"], BOOT).sends(&["wait:01 Identity"]).pending(),


    Root::new("showcase/pages/readonlyreport", Bin::Showcase, &["--page", "readonlyreport"], BOOT).sends(&["wait:Host snapshot"]).pending(),
    Root::new("showcase/sections/readonlyreport-options", Bin::Showcase, &["--page", "readonlyreport"], BOOT).sends(&["tab", "2", "wait:Attention notices"]).pending(),
    Root::new("showcase/sections/readonlyreport-states", Bin::Showcase, &["--page", "readonlyreport"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/flows/readonlyreport/offline", Bin::Showcase, &["--page", "readonlyreport", "--opt", "readonlyreport.offline=on"], BOOT).sends(&["wait:last-good 2 m ago"]).pending(),






    Root::new("showcase/sections/buttons-options", Bin::Showcase, &["--page", "buttons"], BOOT).sends(&["tab", "2", "wait:Focus ring"]).pending(),

    Root::new("showcase/flows/buttons/row-folded", Bin::Showcase, &["--page", "buttons", "--opt", "buttons.row_width=30"], BOOT).pending(),
    Root::new("showcase/flows/buttons/row-wrapped", Bin::Showcase, &["--page", "buttons", "--opt", "buttons.row_width=30", "--opt", "buttons.row_overflow=wrap"], BOOT).pending(),
    Root::new("showcase/elements/buttons/tool-hover", Bin::Showcase, &["--page", "buttons"], BOOT).hover("Format").pending(),
    Root::new("showcase/sections/buttons-states", Bin::Showcase, &["--page", "buttons", "--motion", "paused"], BOOT).sends(&["tab", "3", "wait:focus + hover"]).pending(),
    Root::new("showcase/sections/inputs-options", Bin::Showcase, &["--page", "inputs"], BOOT).sends(&["tab", "2", "wait:Edit entry"]).pending(),

    Root::new("showcase/sections/textareas-options", Bin::Showcase, &["--page", "textareas"], BOOT).sends(&["tab", "2", "wait:Esc reverts"]).pending(),
    Root::new("showcase/sections/textareas-states", Bin::Showcase, &["--page", "textareas"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/flows/inputs/custom-keymap", Bin::Showcase, &["--page", "inputs", "--opt", "inputs.text_keymap=custom"], BOOT).pending(),
    Root::new("showcase/flows/inputs/wrapped-labels", Bin::Showcase, &["--page", "inputs", "--opt", "inputs.wrap_label=on"], BOOT).pending(),
    Root::new("showcase/sections/inputs-states", Bin::Showcase, &["--page", "inputs"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),

    Root::new("showcase/sections/columns-options", Bin::Showcase, &["--page", "columns"], BOOT).sends(&["tab", "2", "wait:Header menu labels"]).pending(),
    Root::new("showcase/sections/columns-states", Bin::Showcase, &["--page", "columns"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/columns-narrow-row", Bin::Showcase, &["--page", "columns", "--opt", "columns.width=28"], BOOT).sends(&["wait:dropped"]).pending(),
    Root::new("showcase/sections/basictable-options", Bin::Showcase, &["--page", "basictable"], BOOT).sends(&["tab", "2", "wait:Selection"]).pending(),
    Root::new("showcase/sections/basictable-states", Bin::Showcase, &["--page", "basictable"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),

    Root::new("showcase/sections/datagrid-options", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "2", "wait:Duplicate row keys"]).pending(),
    Root::new("showcase/sections/datagrid-states", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/datagrid-read-only", Bin::Showcase, &["--page", "datagrid", "--opt", "datagrid.access=view"], BOOT).sends(&["tab", "tab", "enter", "wait:customer_summary is a view"]).pending(),
    Root::new("showcase/sections/datagrid-error-empty", Bin::Showcase, &["--page", "datagrid", "--opt", "datagrid.rows=error"], BOOT).sends(&["wait:Could not load customers"]).pending(),
    Root::new("showcase/flows/datagrid/filter-chip", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "tab", "right", "right", "f", "wait:plan = enterprise"]).pending(),


    Root::new("showcase/sections/editable-options", Bin::Showcase, &["--page", "editable"], BOOT).sends(&["tab", "2", "wait:Header menu labels"]).pending(),
    Root::new("showcase/sections/editable-states", Bin::Showcase, &["--page", "editable"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/selection-options", Bin::Showcase, &["--page", "selection"], BOOT).sends(&["tab", "2", "wait:Checkbox column"]).pending(),
    Root::new("showcase/sections/selection-states", Bin::Showcase, &["--page", "selection"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/celleditors-options", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["tab", "2", "wait:Popup placement"]).pending(),
    Root::new("showcase/sections/celleditors-states", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/celleditors-affordance-always", Bin::Showcase, &["--page", "celleditors", "--opt", "celleditors.affordance=always"], BOOT).sends(&["wait:checkout-api"]).pending(),
    Root::new("showcase/flows/celleditors/standalone-open", Bin::Showcase, &["--page", "celleditors", "--opt", "celleditors.placement=above"], BOOT).sends(&["tab", "tab", "tab", "enter", "wait:Search"]).pending(),
    Root::new("showcase/sections/basictable-multi", Bin::Showcase, &["--page", "basictable", "--opt", "basictable.selection=multi"], BOOT).sends(&["wait:[ ] #1040"]).pending(),


    Root::new("showcase/sections/chrome-options", Bin::Showcase, &["--page", "chrome"], BOOT).sends(&["tab", "2", "wait:Press-drag-release"]).pending(),
    Root::new("showcase/sections/chrome-states", Bin::Showcase, &["--page", "chrome", "--motion", "paused"], BOOT).sends(&["tab", "3", "wait:Clickable segment"]).pending(),
    Root::new("showcase/sections/chrome-narrow-chevron", Bin::Showcase, &["--page", "chrome", "--opt", "chrome.bar=narrow", "--opt", "chrome.overflow=chevron", "--motion", "paused"], BOOT).sends(&["wait:overflow Chevron"]).pending(),
    Root::new("showcase/sections/chrome-status-tooltip", Bin::Showcase, &["--page", "chrome", "--motion", "paused"], BOOT).hover("PR #482").pending(),
    Root::new("showcase/sections/statushints-options", Bin::Showcase, &["--page", "statushints"], BOOT).sends(&["tab", "2", "wait:F5 priority"]).pending(),
    Root::new("showcase/sections/statushints-states", Bin::Showcase, &["--page", "statushints"], BOOT).sends(&["tab", "3", "wait:Pinned F5"]).pending(),
    Root::new("showcase/sections/statushints-modal-narrow", Bin::Showcase, &["--page", "statushints", "--opt", "statushints.layer=modal", "--opt", "statushints.width=narrow"], BOOT).sends(&["wait:Enter Delete"]).pending(),


    Root::new("showcase/sections/statushints-shell", Bin::Showcase, &["--page", "statushints", "--motion", "paused"], BOOT).sends(&["tab", "tab", "wait:CommonArgs"]).pending(),
    Root::new("showcase/sections/statushints-too-small-custom", Bin::Showcase, &["--page", "statushints", "--motion", "paused", "--opt", "statushints.too_small_min=100x30", "--opt", "statushints.quit_hint=custom", "--opt", "statushints.brand=accent"], BOOT).sends(&["tab", "tab", "wait:resize to continue"]).pending(),


    Root::new("showcase/sections/textutilities-options", Bin::Showcase, &["--page", "textutilities"], BOOT).sends(&["tab", "2", "wait:Copy key"]).pending(),
    Root::new("showcase/sections/textutilities-states", Bin::Showcase, &["--page", "textutilities"], BOOT).sends(&["tab", "3", "wait:Narrow labels"]).pending(),
    Root::new("showcase/sections/textutilities-width-8", Bin::Showcase, &["--page", "textutilities", "--opt", "textutilities.width=8", "--opt", "textutilities.duration=compact"], BOOT).sends(&["wait:width 8"]).pending(),
    Root::new("showcase/sections/textutilities-list-focus", Bin::Showcase, &["--page", "textutilities"], BOOT).click("Port").pending(),
    Root::new("showcase/sections/breadcrumbsrails-options", Bin::Showcase, &["--page", "breadcrumbsrails"], BOOT).sends(&["tab", "2", "wait:Deep trail width"]).pending(),
    Root::new("showcase/sections/breadcrumbsrails-states", Bin::Showcase, &["--page", "breadcrumbsrails"], BOOT).sends(&["tab", "3", "wait:Plain text"]).pending(),
    Root::new("showcase/sections/breadcrumbsrails-collapsed", Bin::Showcase, &["--page", "breadcrumbsrails", "--opt", "breadcrumbsrails.collapsed=on"], BOOT).sends(&["wait:RailAction::Expand"]).pending(),
    Root::new("showcase/sections/breadcrumbsrails-narrow-trail", Bin::Showcase, &["--page", "breadcrumbsrails", "--opt", "breadcrumbsrails.width=20"], BOOT).sends(&["wait:width 20"]).pending(),
    Root::new("showcase/sections/helpkeys-options", Bin::Showcase, &["--page", "helpkeys"], BOOT).sends(&["tab", "2", "wait:Overlay size"]).pending(),
    Root::new("showcase/sections/helpkeys-states", Bin::Showcase, &["--page", "helpkeys"], BOOT).sends(&["tab", "3", "wait:Narrow keys"]).pending(),
    Root::new("showcase/sections/helpkeys-reference-narrow", Bin::Showcase, &["--page", "helpkeys", "--opt", "helpkeys.width=28", "--opt", "helpkeys.key_max_share=25"], BOOT).sends(&["wait:width 28"]).pending(),
    Root::new("showcase/sections/quitleave-options", Bin::Showcase, &["--page", "quitleave"], BOOT).sends(&["tab", "2", "wait:Quit letter"]).pending(),
    Root::new("showcase/sections/quitleave-states", Bin::Showcase, &["--page", "quitleave"], BOOT).sends(&["tab", "3", "wait:Discard"]).pending(),
    Root::new("showcase/sections/quitleave-running", Bin::Showcase, &["--page", "quitleave", "--opt", "quitleave.stakes=running"], BOOT).sends(&["wait:Stop export"]).pending(),
    Root::new("showcase/sections/lists-options", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["tab", "2", "wait:Skip disabled"]).pending(),
    Root::new("showcase/sections/lists-states", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/lists-empty-state", Bin::Showcase, &["--page", "lists", "--opt", "lists.empty=state"], BOOT).sends(&["wait:Try a shorter word"]).pending(),
    Root::new("showcase/sections/trees-options", Bin::Showcase, &["--page", "trees"], BOOT).sends(&["tab", "2", "wait:Selected marker"]).pending(),
    Root::new("showcase/sections/trees-states", Bin::Showcase, &["--page", "trees"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/trees-lazy-loading", Bin::Showcase, &["--page", "trees", "--motion", "paused"], BOOT).click("orders-db").pending(),
    Root::new("showcase/elements/trees/connection-hover", Bin::Showcase, &["--page", "trees"], BOOT).hover("replica-eu").pending(),
    Root::new("showcase/elements/breadcrumbs/crumb-hover", Bin::Showcase, &["--page", "breadcrumbsrails"], BOOT).hover("Users ›").pending(),
    Root::new("showcase/elements/rails/collapse-hover", Bin::Showcase, &["--page", "breadcrumbsrails"], BOOT).hover("‹").pending(),

    Root::new("showcase/sections/overview-options", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["tab", "2", "wait:State modifiers"]).pending(),
    Root::new("showcase/sections/overview-states", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["tab", "3", "wait:Cursor fill"]).pending(),
    Root::new("showcase/sections/overview-mono-modifiers", Bin::Showcase, &["--page", "overview", "--opt", "overview.mods=mono", "--opt", "overview.kind=primary"], BOOT).sends(&["wait:modifiers MONO"]).pending(),


    Root::new("showcase/pages/tabs", Bin::Showcase, &["--page", "tabs", "--motion", "paused"], BOOT).sends(&["wait:Editor strip"]).pending(),
    Root::new("showcase/sections/tabs-options", Bin::Showcase, &["--page", "tabs"], BOOT).sends(&["tab", "2", "wait:Active rule"]).pending(),
    Root::new("showcase/sections/tabs-states", Bin::Showcase, &["--page", "tabs", "--motion", "paused"], BOOT).sends(&["tab", "3", "wait:focus + hover"]).pending(),


    Root::new("showcase/pages/menus", Bin::Showcase, &["--page", "menus"], BOOT).sends(&["wait:Actions menu"]).pending(),
    Root::new("showcase/sections/menus-options", Bin::Showcase, &["--page", "menus"], BOOT).sends(&["tab", "2", "wait:Actions placement"]).pending(),
    Root::new("showcase/sections/menus-states", Bin::Showcase, &["--page", "menus"], BOOT).sends(&["tab", "3", "wait:Commands"]).pending(),

    Root::new("showcase/sections/dialogs-options", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["tab", "2", "wait:Placement"]).pending(),
    Root::new("showcase/sections/dialogs-states", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["tab", "3", "wait:Run task now?"]).pending(),


    Root::new("showcase/sections/pickers-options", Bin::Showcase, &["--page", "pickers"], BOOT).sends(&["tab", "2", "wait:Max rows"]).pending(),
    Root::new("showcase/sections/pickers-states", Bin::Showcase, &["--page", "pickers"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),


    Root::new("showcase/pages/popupstooltips", Bin::Showcase, &["--page", "popupstooltips"], BOOT).sends(&["wait:Open popover"]).pending(),
    Root::new("showcase/sections/popupstooltips-options", Bin::Showcase, &["--page", "popupstooltips"], BOOT).sends(&["tab", "2", "wait:Popover placement"]).pending(),
    Root::new("showcase/sections/popupstooltips-states", Bin::Showcase, &["--page", "popupstooltips"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).pending(),
    Root::new("showcase/sections/popupstooltips-tooltip", Bin::Showcase, &["--page", "popupstooltips"], BOOT).hover("Save draft").pending(),
    Root::new("showcase/sections/popupstooltips-status-tooltip", Bin::Showcase, &["--page", "popupstooltips"], BOOT).hover("eu-west-1").pending(),
    Root::new("showcase/sections/datagrid-header-tooltip", Bin::Showcase, &["--page", "datagrid"], BOOT).hover("plan").pending(),

    Root::new("showcase/pages/dialogkinds", Bin::Showcase, &["--page", "dialogkinds"], BOOT).sends(&["wait:Keyboard help"]).pending(),
    Root::new("showcase/sections/dialogkinds-options", Bin::Showcase, &["--page", "dialogkinds"], BOOT).sends(&["tab", "2", "wait:Width"]).pending(),
    Root::new("showcase/sections/dialogkinds-states", Bin::Showcase, &["--page", "dialogkinds"], BOOT).sends(&["tab", "3", "wait:Split"]).pending(),


    Root::new("showcase/pages/focusnavigation", Bin::Showcase, &["--page", "focusnavigation"], BOOT).sends(&["wait:Nav map"]).pending(),
    Root::new("showcase/sections/focusnavigation-options", Bin::Showcase, &["--page", "focusnavigation"], BOOT).sends(&["tab", "2", "wait:List nav keys"]).pending(),
    Root::new("showcase/sections/focusnavigation-states", Bin::Showcase, &["--page", "focusnavigation"], BOOT).sends(&["tab", "3", "wait:Tab strip"]).pending(),
    Root::new("showcase/elements/focusnavigation/vim-map", Bin::Showcase, &["--page", "focusnavigation", "--opt", "focusnavigation.nav=vim"], BOOT).sends(&["wait:Nav map"]).pending(),


    Root::new("showcase/pages/toolbars", Bin::Showcase, &["--page", "toolbars"], BOOT).sends(&["wait:Query toolbar"]).pending(),
    Root::new("showcase/sections/toolbars-options", Bin::Showcase, &["--page", "toolbars"], BOOT).sends(&["tab", "2", "wait:Skip disabled"]).pending(),
    Root::new("showcase/sections/toolbars-states", Bin::Showcase, &["--page", "toolbars"], BOOT).sends(&["tab", "3", "wait:Toggles"]).pending(),



    Root::new("showcase/sections/panels-options", Bin::Showcase, &["--page", "panels"], BOOT).sends(&["tab", "2", "wait:Follow key"]).pending(),
    Root::new("showcase/sections/panels-states", Bin::Showcase, &["--page", "panels"], BOOT).sends(&["tab", "3", "wait:Card · bar mark"]).pending(),
    Root::new("showcase/flows/panels/focus-bar", Bin::Showcase, &["--page", "panels", "--opt", "panels.focus_mark=bar", "--opt", "panels.tail=on"], BOOT).sends(&["tab", "tab", "tab", "tab", "wait:Follow tail"]).pending(),
    Root::new("showcase/sections/scrolling-options", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["wait:739.63s", "tab", "2", "wait:Skip disabled"]).pending(),
    Root::new("showcase/sections/scrolling-states", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["wait:739.63s", "tab", "3", "wait:Log tail"]).pending(),
    Root::new("showcase/flows/scrolling/vim-keys", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600", "--opt", "scrolling.profile=vim"], BOOT).sends(&["wait:739.63s", "tab", "tab", "tab", "G", "wait:Home End g G"]).pending(),
    Root::new("showcase/flows/scrolling/flag", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["wait:739.63s", "tab", "tab", "tab", "down", "f", "wait:flagged"]).pending(),
    Root::new("showcase/elements/toolbars/focused", Bin::Showcase, &["--page", "toolbars"], BOOT).sends(&["tab", "tab", "right", "right"]).pending(),
    Root::new("showcase/elements/toolbars/item-hover", Bin::Showcase, &["--page", "toolbars"], BOOT).hover("Format").pending(),

    Root::new("showcase/audit/buttons", Bin::Showcase, &["--page", "buttons"], BOOT),
    Root::new("showcase/audit/diff", Bin::Showcase, &["--page", "diff"], BOOT),
    Root::new("showcase/audit/forms", Bin::Showcase, &["--page", "forms"], BOOT),

    Root::new("showcase/sections/forms-options", Bin::Showcase, &["--page", "forms"], BOOT).sends(&["tab", "2", "wait:Managed toggle"]).pending(),
    Root::new("showcase/sections/forms-states", Bin::Showcase, &["--page", "forms"], BOOT).sends(&["tab", "3", "wait:Required field"]).pending(),
    Root::new("showcase/flows/forms/plain", Bin::Showcase, &["--page", "forms", "--opt", "forms.help=off", "--opt", "forms.placeholders=off", "--opt", "forms.managed=off"], BOOT).sends(&["wait:Task name"]).pending(),
    Root::new("showcase/audit/inputs", Bin::Showcase, &["--page", "inputs"], BOOT),
    Root::new("showcase/audit/textareas", Bin::Showcase, &["--page", "textareas"], BOOT),
];

#[rustfmt::skip]
pub const ELEMENTS: &[Root] = &[

    Root::new("showcase/hover/buttons", Bin::Showcase, &["--page", "buttons"], BOOT).live(interact::hover_buttons),
    Root::new("showcase/hover/lists", Bin::Showcase, &["--page", "lists"], BOOT).live(interact::hover_lists),
    Root::new("showcase/hover/tables", Bin::Showcase, &["--page", "tables"], BOOT).live(interact::hover_tables),

    Root::new("showcase/elements/buttons/run-hover", Bin::Showcase, &["--page", "buttons"], BOOT).hover("Run task").pending(),
    Root::new("showcase/elements/buttons/run-pressed", Bin::Showcase, &["--page", "buttons", "--motion", "paused"], BOOT).press("Run task").pending(),
    Root::new("showcase/elements/buttons/disabled-hover", Bin::Showcase, &["--page", "buttons"], BOOT).hover("Disabled primary").pending(),
    Root::new("showcase/elements/buttons/disabled-pressed", Bin::Showcase, &["--page", "buttons", "--motion", "paused"], BOOT).press("Disabled primary").pending(),
    Root::new("showcase/elements/toggles/verbose-hover", Bin::Showcase, &["--page", "buttons"], BOOT).hover("Verbose").pending(),



    Root::new("showcase/elements/nav/item-hover", Bin::Showcase, &["--page", "buttons"], BOOT).hover("Text areas").pending(),
    Root::new("showcase/elements/nav/item-pressed", Bin::Showcase, &["--page", "buttons", "--motion", "paused"], BOOT).press("Text areas").pending(),



    Root::new("showcase/elements/nav/group-hover", Bin::Showcase, &["--page", "overview"], BOOT).hover("Tables").pending(),
    Root::new("showcase/elements/nav/group-collapsed", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["left", "left"]).pending(),
    Root::new("showcase/elements/nav/all-groups", Bin::Showcase, &["--page", "overview", "--nav", "all"], BOOT).pending(),
    Root::new("showcase/elements/nav/type-ahead", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["t"]).pending(),
    Root::new("showcase/elements/nav/current-in-group", Bin::Showcase, &["--page", "datagrid"], BOOT).pending(),



    Root::new("showcase/elements/celleditors/select-open", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["tab", "tab", "enter", "wait:Disaster recovery"]).pending(),
    Root::new("showcase/elements/celleditors/combo-search", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["tab", "tab", "right", "right", "right", "enter", "type:ros", "wait:Sofia Rossi"]).pending(),
    Root::new("showcase/elements/celleditors/segmented-open", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["tab", "tab", "right", "right", "enter", "wait:50 %"]).pending(),
    Root::new("showcase/elements/celleditors/tags-open", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["tab", "tab", "right", "right", "right", "right", "right", "enter", "wait:Retired label"]).pending(),
    Root::new("showcase/elements/celleditors/radio-open", Bin::Showcase, &["--page", "celleditors"], BOOT).sends(&["tab", "tab", "right", "right", "right", "right", "right", "right", "enter", "wait:Page on-call now"]).pending(),
    Root::new("showcase/elements/celleditors/cell-hover", Bin::Showcase, &["--page", "celleditors"], BOOT).hover("Staging").pending(),
    Root::new("showcase/elements/nav/filter", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["/", "type:grid"]).pending(),
    Root::new("showcase/elements/selectable/selected-hover", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno", "tab", "tab", "down", "space", "wait:1 selected"]).hover("Ari Moreno").pending(),
    Root::new("showcase/elements/lists/selected-hover", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["tab", "tab", "space"]).hover("Python").pending(),


    Root::new("showcase/elements/states/compare-levels", Bin::Showcase, &["--page", "buttons", "--motion", "paused"], BOOT).sends(&["tab", "3", "tab", "enter", "wait:256 colors"]).pending(),
    Root::new("showcase/elements/states/compare-hover", Bin::Showcase, &["--page", "buttons"], BOOT).sends(&["tab", "3", "wait:Compare levels"]).hover("Compare levels").pending(),


    Root::new("showcase/elements/tabs/tab-hover", Bin::Showcase, &["--page", "tabs", "--motion", "paused"], BOOT).hover("orders.sql").pending(),
    Root::new("showcase/elements/tabs/strip-focused", Bin::Showcase, &["--page", "tabs", "--motion", "paused"], BOOT).sends(&["tab", "tab", "right"]).pending(),
    Root::new("showcase/elements/tabs/rule-neutral", Bin::Showcase, &["--page", "tabs", "--motion", "paused", "--opt", "tabs.rule=neutral"], BOOT).pending(),
    Root::new("showcase/elements/tabs/close-on-hover", Bin::Showcase, &["--page", "tabs", "--motion", "paused", "--opt", "tabs.close_reveal=hover"], BOOT).pending(),
    Root::new("showcase/elements/lists/row-pressed", Bin::Showcase, &["--page", "lists", "--motion", "paused"], BOOT).press("Python").pending(),
];

#[rustfmt::skip]
pub const OVERLAYS: &[Root] = &[


    Root::new("showcase/flows/help/overlay", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["?"]).dismiss_pending(),
    Root::new("showcase/flows/inspector/open", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["i"]).dismiss_pending(),


    Root::new("showcase/flows/inspector/options", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["i", "wait:Options"]).click("Options").pending(),

    Root::new("showcase/flows/inspector/opt-glyphs", Bin::Showcase, &["--page", "buttons", "--opt", "shell.glyphs=ascii"], BOOT).pending(),

    Root::new("showcase/flows/dialogs/open", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["tab", "tab", "enter"]).dismiss_pending(),

    Root::new("showcase/flows/dialogs/prompt", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["tab", "tab", "tab", "enter", "wait:Shown in the task list"]).dismiss_pending(),

    Root::new("showcase/flows/dialogs/destructive", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["d", "wait:Delete branch?"]).dismiss_pending(),
    Root::new("showcase/flows/pickers/open", Bin::Showcase, &["--page", "pickers"], BOOT).sends(&["tab", "tab", "enter"]).dismiss_pending(),

    Root::new("showcase/flows/chrome/menu", Bin::Showcase, &["--page", "chrome"], BOOT).sends(&["tab", "tab", "enter", "wait:New tab"]).dismiss_pending(),

    Root::new("showcase/flows/chrome/context", Bin::Showcase, &["--page", "chrome"], BOOT).live(interact::chrome_context).dismiss_pending(),


    Root::new("showcase/flows/editor/completion", Bin::Showcase, &["--page", "codeeditor"], BOOT).sends(&["tab", "tab", "type:cl", "wait:Client"]).dismiss_pending(),




    Root::new("showcase/flows/datagrid/save-review", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "tab", "right", "right", "right", "enter", "ctrl-l", "type:12", "enter", "ctrl-s", "wait:Save changes"]).pending().dismiss_pending(),
    Root::new("showcase/flows/datagrid/preview", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "p", "wait:UPDATE customers SET seats = 600"]).dismiss_pending(),
    Root::new("showcase/flows/selectable/delete_confirm", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno", "tab", "tab", "down", "space", "wait:1 selected", "shift-down", "wait:2 selected", "d", "wait:Delete 2 members?"]).dismiss_pending(),

    Root::new("showcase/overlays/chrome_context_row", Bin::Showcase, &["--page", "chrome"], BOOT).right_click("Codex (Primary)").pending().dismiss_pending(),

    Root::new("showcase/overlays/chrome_env_menu", Bin::Showcase, &["--page", "chrome", "--motion", "paused"], BOOT).click("production").pending().dismiss_pending(),
    Root::new("showcase/overlays/lists_context", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["tab", "tab", "shift-f10"]).pending().dismiss_pending(),


    Root::new("showcase/overlays/tabs_overflow_menu", Bin::Showcase, &["--page", "tabs", "--motion", "paused"], BOOT).click("+").pending().dismiss_pending(),
    Root::new("showcase/overlays/tabs_close_dirty", Bin::Showcase, &["--page", "tabs", "--motion", "paused"], BOOT).sends(&["tab", "tab", "right", "x", "wait:Discard"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/helpkeys_overlay", Bin::Showcase, &["--page", "helpkeys"], BOOT).click("Open help overlay").pending().dismiss_pending(),
    Root::new("showcase/overlays/helpkeys_overlay_small", Bin::Showcase, &["--page", "helpkeys", "--opt", "helpkeys.max_percent=60", "--opt", "helpkeys.title=custom"], BOOT).click("Open help overlay").pending().dismiss_pending(),
    Root::new("showcase/overlays/quitleave_unsaved", Bin::Showcase, &["--page", "quitleave", "--opt", "quitleave.stakes=unsaved"], BOOT).click("Leave for Overview").pending().dismiss_pending(),
    Root::new("showcase/overlays/quitleave_running", Bin::Showcase, &["--page", "quitleave", "--opt", "quitleave.stakes=running"], BOOT).click("Quit    Stop export").pending().dismiss_pending(),



    Root::new("showcase/overlays/menus_row_keyboard", Bin::Showcase, &["--page", "menus"], BOOT).sends(&["tab", "tab", "shift-f10", "wait:Duplicate"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/menus_row_pointer", Bin::Showcase, &["--page", "menus"], BOOT).right_click("README.md").pending().dismiss_pending(),
    Root::new("showcase/overlays/menus_actions", Bin::Showcase, &["--page", "menus"], BOOT).click("Actions ▾").pending().dismiss_pending(),



    Root::new("showcase/overlays/dialogs_viewer", Bin::Showcase, &["--page", "dialogs"], BOOT).click("View config").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogs_gate", Bin::Showcase, &["--page", "dialogs"], BOOT).click("Drop table…").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogs_push", Bin::Showcase, &["--page", "dialogs"], BOOT).click("Push failed…").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogs_notes", Bin::Showcase, &["--page", "dialogs"], BOOT).click("Release notes").pending().dismiss_pending(),




    Root::new("showcase/overlays/filebrowser", Bin::Showcase, &["--page", "filebrowser"], BOOT).click("Browse…").pending().dismiss_pending(),
    Root::new("showcase/overlays/filebrowser_url", Bin::Showcase, &["--page", "filebrowser", "--opt", "filebrowser.url_entry=on", "--opt", "filebrowser.stepper=on"], BOOT).click("Browse…").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogkinds_form", Bin::Showcase, &["--page", "dialogkinds"], BOOT).click("Add connection…").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogkinds_info", Bin::Showcase, &["--page", "dialogkinds"], BOOT).click("Server info").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogkinds_choice", Bin::Showcase, &["--page", "dialogkinds"], BOOT).click("Release channel…").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogkinds_help", Bin::Showcase, &["--page", "dialogkinds"], BOOT).click("Keyboard help").pending().dismiss_pending(),

    Root::new("showcase/overlays/popupstooltips_popover", Bin::Showcase, &["--page", "popupstooltips"], BOOT).click("Open popover").pending().dismiss_pending(),


    Root::new("showcase/overlays/pickers_commands", Bin::Showcase, &["--page", "pickers"], BOOT).click("Run command").pending().dismiss_pending(),
    Root::new("showcase/overlays/pickers_remote", Bin::Showcase, &["--page", "pickers", "--motion", "paused"], BOOT).click("Remote branch").pending().dismiss_pending(),
    Root::new("showcase/overlays/dialogs_anchored", Bin::Showcase, &["--page", "dialogs", "--opt", "dialogs.placement=below"], BOOT).click("Confirm run").pending().dismiss_pending(),
    Root::new("showcase/overlays/emptyerror_row_actions", Bin::Showcase, &["--page", "emptyerror"], BOOT).click("Row actions ▾").pending().dismiss_pending(),

    Root::new("showcase/overlays/quit_confirm", Bin::Showcase, &["--page", "overview"], BOOT).click("Junie").pending().dismiss_pending(),

    Root::new("showcase/overlays/settings_quit_guard", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "tab", "space", "wait:General · unsaved", "q", "wait:Unsaved changes"]).pending().dismiss_pending(),


    Root::new("showcase/overlays/dialogs_gate2", Bin::Showcase, &["--page", "dialogs"], BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "tab", "tab", "enter", "wait:1 of 2", "right", "enter", "wait:2 of 2"]).pending().dismiss_pending(),






    Root::new("showcase/overlays/launchcockpit_credentials", Bin::Showcase, &["--page", "launchcockpit", "--opt", "launchcockpit.outcome=credentials"], BOOT).sends(&["wait:not started", "tab", "tab", "tab", "tab", "enter", "wait:Credential source unavailable"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/launchcockpit_build_failed", Bin::Showcase, &["--page", "launchcockpit", "--opt", "launchcockpit.outcome=build-failure"], BOOT).sends(&["wait:not started", "tab", "tab", "tab", "tab", "enter", "wait:Docker build failed"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/launchcockpit_cancel", Bin::Showcase, &["--page", "launchcockpit"], BOOT).sends(&["wait:not started", "tab", "tab", "tab", "tab", "enter", "tab", "enter", "wait:Cancel launch?"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/diskcleanup_gate1", Bin::Showcase, &["--page", "diskcleanup"], BOOT).sends(&["wait:Cleanup categories", "tab", "tab", "tab", "tab", "d", "wait:gate 1 of 2"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/diskcleanup_gate2", Bin::Showcase, &["--page", "diskcleanup"], BOOT).sends(&["wait:Cleanup categories", "tab", "tab", "tab", "tab", "d", "wait:gate 1 of 2", "right", "enter", "wait:gate 2 of 2"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/workspaces_purge", Bin::Showcase, &["--page", "workspaces"], BOOT).sends(&["wait:daemon · 3 s ago", "tab", "tab", "tab", "tab", "tab", "tab", "tab", "tab", "enter", "wait:Delete payments-platform?"]).pending().dismiss_pending(),


    Root::new("showcase/overlays/dialogkinds_key_picker", Bin::Showcase, &["--page", "dialogkinds"], BOOT).sends(&["tab", "tab", "enter", "wait:Add connection", "tab", "tab", "tab", "tab", "tab", "space", "tab", "enter", "wait:SSH key"]).pending().dismiss_pending(),

    Root::new("showcase/overlays/selectable_context", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno", "tab", "tab", "shift-f10"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/tables_context", Bin::Showcase, &["--page", "tables"], BOOT).sends(&["tab", "tab", "shift-f10"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/trees_context", Bin::Showcase, &["--page", "trees"], BOOT).sends(&["tab", "tab", "shift-f10"]).pending().dismiss_pending(),

    Root::new("showcase/overlays/selectable_discard", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno", "tab", "tab", "down", "space", "wait:1 selected", "enter", "enter", "end", "backspace", "backspace", "backspace", "backspace", "backspace", "backspace", "backspace", "backspace", "backspace", "backspace", "enter", "wait:Unsaved changes"]).click("Cleo Barnes").pending().dismiss_pending(),


    Root::new("showcase/overlays/settings_save_keyboard", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "tab", "space", "wait:General · unsaved", "tab", "tab", "enter", "wait:Save settings?"]).pending().dismiss_pending(),
    Root::new("showcase/overlays/settings_save_pointer", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "tab", "space", "wait:General · unsaved"]).click("Save changes").pending().dismiss_pending(),
    Root::new("showcase/overlays/settings_remove_member", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "right", "wait:Remove…"]).click("Remove…").pending().dismiss_pending(),
    Root::new("showcase/overlays/settings_add_variable", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "right", "right", "wait:Add variable…"]).click("Add variable…").pending().dismiss_pending(),

    Root::new("showcase/overlays/taskrunner_cancel", Bin::Showcase, &["--page", "taskrunner", "--motion", "paused"], BOOT).sends(&["r", "wait:Cancel"]).click("Cancel").pending().dismiss_pending(),


    Root::new("showcase/overlays/connections_form", Bin::Showcase, &["--page", "connections"], BOOT).sends(&["wait:New connection…"]).click("New connection…").pending().dismiss_pending(),
    Root::new("showcase/overlays/finder_alternatives", Bin::Showcase, &["--page", "finder"], BOOT).sends(&["wait:Alternatives…"]).click("Alternatives…").pending().dismiss_pending(),
    Root::new("showcase/overlays/finder_why", Bin::Showcase, &["--page", "finder"], BOOT).sends(&["wait:Why?"]).click("Why?").pending().dismiss_pending(),
    Root::new("showcase/overlays/connections_delete", Bin::Showcase, &["--page", "connections"], BOOT).sends(&["wait:Delete…"]).click("Delete…").pending().dismiss_pending(),
];

#[rustfmt::skip]
pub const JOURNEYS: &[Root] = &[

    Root::new("showcase/flows/inputs/editing", Bin::Showcase, &["--page", "inputs"], BOOT).sends(&["tab", "enter", "wait:EDIT"]),
    Root::new("showcase/flows/inputs/selected", Bin::Showcase, &["--page", "inputs"], BOOT).sends(&["tab", "enter", "ctrl-l", "wait:EDIT"]),
    Root::new("showcase/flows/forms/invalid", Bin::Showcase, &["--page", "forms"], BOOT).sends(&["tab", "ctrl-s", "wait:Required"]),
    Root::new("showcase/flows/diff/review", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "tab", "enter", "wait:● Review"]),
    Root::new("showcase/flows/diff/empty", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "tab", "enter", "tab", "enter", "wait:No file selected"]),
    Root::new("showcase/flows/buttons/focus", Bin::Showcase, &["--page", "buttons"], BOOT).sends(&["tab"]),
    Root::new("showcase/flows/lists/moved", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["tab", "tab", "down", "down"]),
    Root::new("showcase/flows/trees/expanded", Bin::Showcase, &["--page", "trees"], BOOT).sends(&["tab", "tab", "right"]),

    Root::new("showcase/flows/tables/selected", Bin::Showcase, &["--page", "tables"], BOOT).sends(&["tab", "tab", "down", "down"]),
    Root::new("showcase/flows/editable/editing", Bin::Showcase, &["--page", "editabletables"], BOOT).sends(&["tab", "tab", "enter"]),
    Root::new("showcase/flows/datagrid/selected", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "tab", "down", "right"]),
    Root::new("showcase/flows/chips/toggled", Bin::Showcase, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "space"]),

    Root::new("showcase/flows/scrolling/scrolled", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["wait:739.63s", "tab", "tab", "down", "down", "down"]),
    Root::new("showcase/flows/settings/toggled", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "space"]),




    Root::new("showcase/flows/selectable/bulk_mixed", Bin::Showcase, &["--page", "selectable"], BOOT).sends(&["wait:Ari Moreno", "tab", "tab", "down", "space", "wait:1 selected", "shift-down", "shift-down", "wait:3 selected"]),






    Root::new("showcase/flows/forms/valid", Bin::Showcase, &["--page", "forms"], BOOT).sends(&["tab", "tab", "enter", "type:Fix the login redirect loop", "enter", "ctrl-s", "wait:Task created ✓"]),


    Root::new("showcase/flows/nav/cycled", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["]", "wait:App shell › Chrome"]),


    Root::new("showcase/flows/nav/end", Bin::Showcase, &["--page", "overview"], BOOT).sends(&["0", "G"]),


    Root::new("showcase/flows/pickers/level", Bin::Showcase, &["--page", "pickers"], BOOT).sends(&["tab", "tab", "tab", "tab", "enter", "down", "enter", "wait:Chose Safe Mode (Full)"]),

    Root::new("showcase/flows/editable/committed", Bin::Showcase, &["--page", "editabletables"], BOOT).sends(&["tab", "tab", "enter", "type:x", "enter", "wait:1 edits"]),


    Root::new("showcase/flows/settings/toggled_off", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "tab", "tab", "tab", "tab", "space", "space", "wait:General · unsaved"]),





    Root::new("showcase/flows/editor/diag", Bin::Showcase, &["--page", "codeeditor"], BOOT).sends(&["tab", "tab", "}", "ctrl-r", "wait:Block ran in 77 ms"]),



    Root::new("showcase/flows/editor/running", Bin::Showcase, &["--page", "codeeditor", "--motion", "paused"], BOOT).sends(&["tab", "tab", "}", "ctrl-r"]),
    Root::new("showcase/flows/datagrid/pending", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(GRID_EDITS),



    Root::new("showcase/flows/datagrid/failed", Bin::Showcase, &["--page", "datagrid"], BOOT).sends(&["tab", "tab", "right", "right", "right", "enter", "ctrl-l", "type:600", "enter", "down", "enter", "ctrl-l", "type:12", "enter", "ctrl-s", "type:customers", "enter", "wait:Save failed"]),


    Root::new("showcase/flows/pickers/tabs", Bin::Showcase, &["--page", "pickers", "--motion", "paused"], BOOT).sends(&["tab", "tab", "tab", "enter", "wait:Filter tabs…"]),




    Root::new("showcase/flows/chips/select_open", Bin::Showcase, &["--page", "chipsselects"], BOOT).sends(&["tab", "tab", "tab", "tab", "enter", "wait:customer"]),

    Root::new("showcase/flows/settings/members", Bin::Showcase, &["--page", "settings"], BOOT).sends(&["tab", "tab", "right", "tab", "down", "wait:cursor: Jonas Weber"]),


    Root::new("showcase/flows/chrome/menu_view", Bin::Showcase, &["--page", "chrome"], BOOT).sends(&["tab", "tab", "enter", "right", "right", "wait:Zoom pane"]),


    Root::new("showcase/flows/chrome/focus", Bin::Showcase, &["--page", "chrome"], BOOT).sends(&["tab", "tab", "wait:←→ Menu"]),

    Root::new("showcase/flows/progress/mid", Bin::Showcase, &["--page", "progress", "--motion", "paused", "--frame", "80"], BOOT),


    Root::new("showcase/flows/progress/done", Bin::Showcase, &["--page", "progress", "--motion", "paused", "--frame", "200"], BOOT),



    Root::new("showcase/flows/taskrunner/running", Bin::Showcase, &["--page", "taskrunner", "--motion", "paused"], BOOT).sends(&["r"]),


    Root::new("showcase/flows/scrolling/mid", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "800"], BOOT),


    Root::new("showcase/flows/terminal/mid", Bin::Showcase, &["--page", "terminal", "--motion", "paused", "--frame", "60"], BOOT),

    Root::new("showcase/flows/diff/drag-selected", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "tab", "enter", "wait:● Review"]).live(interact::diff_drag_select),



    Root::new("showcase/flows/inspector/scrolled", Bin::Showcase, &["--page", "lists"], BOOT).sends(&["i"]).live(interact::wheel_python),

    Root::new("showcase/fade/lists/wheel-fade", Bin::Showcase, &["--page", "lists"], BOOT).live(interact::wheel_python),
    Root::new("showcase/fade/trees/wheel-fade", Bin::Showcase, &["--page", "trees"], BOOT).live(interact::trees_wheel),
    Root::new("showcase/fade/datagrid/wheel", Bin::Showcase, &["--page", "datagrid"], BOOT).live(interact::datagrid_wheel),

    Root::new("showcase/fade/editor/wheel", Bin::Showcase, &["--page", "codeeditor"], BOOT).live(interact::editor_wheel),

    Root::new("showcase/fade/textarea/wheel", Bin::Showcase, &["--page", "textareas"], BOOT).live(interact::textarea_wheel),

    Root::new("showcase/fade/diff/wheel", Bin::Showcase, &["--page", "diff"], BOOT).sends(&["tab", "tab", "enter", "wait:● Review"]).live(interact::diff_wheel),


    Root::new("showcase/fade/scrolling/wheel-fade", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).live(interact::scrolling_wheel_up),

    Root::new("showcase/fade/scroll/page", Bin::Showcase, &["--page", "scrolling", "--motion", "paused", "--frame", "1600"], BOOT).sends(&["tab", "tab", "pagedown"]).live(interact::none),

    Root::new("showcase/fade/sidebars/wheel", Bin::Showcase, &["--page", "sidebars"], BOOT).live(interact::sidebars_wheel),

    Root::new("showcase/fade/terminal/scrollback", Bin::Showcase, &["--page", "terminal", "--motion", "paused", "--frame", "60"], BOOT).live(interact::terminal_scrollback),
];

#[rustfmt::skip]
pub const RESIZES: &[Root] = &[
    Root::new("showcase/resize/overview_shrunk", Bin::Showcase, &["--page", "overview"], BOOT).timeout(15_000).resize((120, 40), interact::showcase_overview_resize),
    Root::new("showcase/resize/overview_grown", Bin::Showcase, &["--page", "overview"], BOOT).timeout(15_000).resize((80, 24), interact::showcase_overview_resize),

    Root::new("showcase/resize/lists_from160x50", Bin::Showcase, &["--page", "lists"], BOOT).timeout(15_000).resize((160, 50), interact::resize_case).pending(),
    Root::new("showcase/resize/lists_from72x20", Bin::Showcase, &["--page", "lists"], BOOT).timeout(15_000).resize((72, 20), interact::resize_case).pending(),
];
