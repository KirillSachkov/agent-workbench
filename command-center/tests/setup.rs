mod support;

use std::fs;

use support::Env;

const OWNER_CONFIG: &str =
    "[keys]\nprefix = \"ctrl+a\"\n\n[ui]\nsidebar_width = 30\n\n[ui.toast]\ndelivery = \"herdr\"\n";

fn config(env: &Env) -> String {
    fs::read_to_string(env.path("herdr-config.toml")).unwrap_or_default()
}

#[test]
fn configure_writes_a_fenced_block_after_the_owner_agrees_and_unconfigure_removes_it() {
    let env = Env::new();
    fs::write(env.path("herdr-config.toml"), OWNER_CONFIG).unwrap();

    let declined = env.run_input(&["configure"], "n\n");
    assert!(declined.ok, "{}", declined.stderr);
    assert!(declined.stdout.contains("Nothing written."));
    assert_eq!(config(&env), OWNER_CONFIG);

    let agreed = env.run_input(&["configure"], "y\n");
    assert!(agreed.ok, "{}", agreed.stderr);
    let written = config(&env);
    let parsed: toml::Value = toml::from_str(&written).expect("still valid TOML");
    let commands = parsed["keys"]["command"].as_array().unwrap();
    assert_eq!(commands[0]["key"].as_str(), Some("prefix+i"));
    assert_eq!(commands[0]["type"].as_str(), Some("plugin_action"));
    assert_eq!(
        commands[0]["command"].as_str(),
        Some("agent-workbench.command-center.overview")
    );
    assert_eq!(
        commands[1]["command"].as_str(),
        Some("agent-workbench.command-center.card")
    );
    let tab = &parsed["ui"]["tab_bar_right"][0];
    assert_eq!(tab["type"].as_str(), Some("command"));
    assert!(
        tab["command"]
            .as_str()
            .unwrap()
            .ends_with("workbench-cc' tab-status")
    );
    assert_eq!(
        parsed["ui"]["sidebar_width"].as_integer(),
        Some(30),
        "the owner's settings stay"
    );
    let rows = parsed["ui"]["sidebar"]["agents"]["rows"]
        .as_array()
        .unwrap();
    assert!(
        rows.iter()
            .flat_map(|r| r.as_array().unwrap())
            .any(|c| c.as_str() == Some("$wb_tag"))
    );
    assert!(written.starts_with(OWNER_CONFIG.split("\n\n").next().unwrap()));
    assert_eq!(
        fs::read_to_string(env.path("herdr-config.toml.workbench-cc.bak")).unwrap(),
        OWNER_CONFIG
    );
    assert!(
        env.calls("herdr")
            .contains(&"server reload-config".to_string())
    );

    // Running configure again replaces our blocks instead of adding more.
    assert!(env.run(&["configure", "--yes"]).ok);
    assert_eq!(config(&env), written);

    assert!(env.run(&["unconfigure"]).ok);
    assert_eq!(config(&env), OWNER_CONFIG);
}

#[test]
fn configure_refuses_a_key_that_is_already_taken() {
    let env = Env::new();
    let taken = "[[keys.command]]\nkey = \"prefix+i\"\ntype = \"plugin_action\"\ncommand = \"other.plugin.open\"\n";
    fs::write(env.path("herdr-config.toml"), taken).unwrap();
    let run = env.run(&["configure", "--yes"]);
    assert!(!run.ok);
    assert!(
        run.stderr
            .contains("prefix+i is already bound to [[keys.command]] other.plugin.open"),
        "{}",
        run.stderr
    );
    assert_eq!(config(&env), taken);

    let default = env.run(&["configure", "--yes", "--overview-key", "prefix+e"]);
    assert!(!default.ok, "Herdr's default bindings count as taken");
    assert!(
        default.stderr.contains("edit_scrollback"),
        "{}",
        default.stderr
    );

    let free = env.run(&["configure", "--yes", "--overview-key", "prefix+m"]);
    assert!(free.ok, "{}", free.stderr);
    assert!(config(&env).contains("key = \"prefix+m\""));
}

#[test]
fn configure_leaves_the_owners_own_sidebar_rows_and_tab_bar_alone() {
    let env = Env::new();
    let own = "[ui]\ntab_bar_right = [{ type = \"hostname\" }]\n\n[ui.sidebar.agents]\nrows = [[\"agent\"]]\n";
    fs::write(env.path("herdr-config.toml"), own).unwrap();
    let run = env.run(&["configure", "--yes"]);
    assert!(run.ok, "{}", run.stderr);
    assert!(run.stdout.contains("tab_bar_right is yours"));
    assert!(run.stdout.contains("add \"$wb_tag\""));
    let parsed: toml::Value = toml::from_str(&config(&env)).unwrap();
    assert_eq!(parsed["ui"]["tab_bar_right"].as_array().unwrap().len(), 1);
    assert_eq!(
        parsed["ui"]["sidebar"]["agents"]["rows"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(env.run(&["unconfigure"]).ok);
    assert_eq!(config(&env), own);
}

#[test]
fn configure_works_on_an_empty_configuration() {
    let env = Env::new();
    assert!(env.run(&["configure", "--yes"]).ok);
    let parsed: toml::Value = toml::from_str(&config(&env)).unwrap();
    assert!(parsed["ui"]["tab_bar_right"].is_array());
    assert!(env.run(&["unconfigure"]).ok);
    assert_eq!(config(&env), "");
}

#[test]
fn doctor_reports_each_check() {
    let env = Env::new();
    env.rule("gh", "--version", "gh version 2.88.1 (2026-03-12)\n", 0);
    env.rule("gh", "auth status", "", 1);
    env.write_config("editor = [\"true\"]\ndiff = [\"true\"]\nbrowser = [\"open\", \"{url}\"]\n");
    let run = env.run(&["doctor"]);
    assert!(!run.ok, "a failed check fails doctor");
    assert!(run.stdout.contains("ok   herdr: 0.9.1"), "{}", run.stdout);
    assert!(
        run.stdout.contains("ok   git: git version"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("ok   gh: gh version 2.88.1"),
        "{}",
        run.stdout
    );
    assert!(run.stdout.contains("FAIL gh auth"), "{}", run.stdout);
    assert!(
        run.stdout.contains("ok   editor: /usr/bin/true"),
        "{}",
        run.stdout
    );

    env.rule("gh", "auth status", "", 0);
    env.rule("herdr", "--version", "herdr 0.8.0\n", 0);
    let old = env.run(&["doctor"]);
    assert!(
        old.stdout.contains("FAIL herdr: 0.8.0 is older than 0.9.1"),
        "{}",
        old.stdout
    );

    env.rule("herdr", "--version", "herdr 0.9.2\n", 0);
    let good = env.run(&["doctor"]);
    assert!(good.ok, "{}", good.stdout);
}

#[test]
fn configure_does_not_repeat_an_existing_sidebar_agents_table() {
    let env = Env::new();
    let own = "[keys]\nnew_tab = [\"prefix+i\", \"prefix+c\"]\n\n[ui.sidebar.agents]\n";
    fs::write(env.path("herdr-config.toml"), own).unwrap();
    let taken = env.run(&["configure", "--yes"]);
    assert!(!taken.ok, "a key bound in a list counts as taken");
    assert!(taken.stderr.contains("new_tab"), "{}", taken.stderr);
    let run = env.run(&["configure", "--yes", "--overview-key", "prefix+m"]);
    assert!(run.ok, "{}", run.stderr);
    assert!(run.stdout.contains("add \"$wb_tag\""), "{}", run.stdout);
    toml::from_str::<toml::Value>(&config(&env)).expect("valid TOML");
}
