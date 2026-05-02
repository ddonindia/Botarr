use mlua::{Function, Lua, Table};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum PluginAction {
    Download(String),
    Queue(String),
    MonitorChannel(String, String, String), // plugin_name, network, channel
}

#[derive(Debug, Clone)]
pub enum EventData {
    String(String),
    Tuple2(String, String),
    Tuple4(String, String, String, String),
}

use std::collections::HashMap;

pub struct PluginManager {
    lua: Mutex<Lua>,
    pub loaded_scripts: Arc<RwLock<Vec<String>>>,
    pub recent_logs: Arc<RwLock<HashMap<String, VecDeque<String>>>>,
}

impl PluginManager {
    pub fn new() -> Result<(Self, mpsc::UnboundedReceiver<PluginAction>), mlua::Error> {
        let lua = Lua::new();
        let (tx, rx) = mpsc::unbounded_channel();
        let recent_logs = Arc::new(RwLock::new(HashMap::new()));
        let loaded_scripts = Arc::new(RwLock::new(Vec::new()));

        {
            // Expose botarr table
            let globals = lua.globals();
            let botarr_table = lua.create_table()?;

            // Registry for signals
            let signals = lua.create_table()?;
            botarr_table.set("_signals", signals)?;

            // signal_add(event_name, callback)
            let signal_add = lua.create_function(|lua, (event, func): (String, Function)| {
                let globals = lua.globals();
                let botarr: Table = globals.get("botarr")?;
                let signals: Table = botarr.get("_signals")?;

                // Allow multiple callbacks per signal
                let callbacks: mlua::Value = signals.get(event.clone())?;
                let list = match callbacks {
                    mlua::Value::Table(t) => t,
                    _ => lua.create_table()?,
                };

                let len = list.len()?;
                list.set(len + 1, func)?;
                signals.set(event, list)?;

                Ok(())
            })?;
            botarr_table.set("signal_add", signal_add)?;

            // print(plugin_name, msg)
            let logs_clone = recent_logs.clone();
            let print = lua.create_function(move |_, (plugin, msg): (String, String)| {
                tracing::info!("[{}] Plugin: {}", plugin, msg);
                if let Ok(mut logs_map) = logs_clone.write() {
                    let logs = logs_map
                        .entry(plugin.clone())
                        .or_insert_with(|| VecDeque::with_capacity(100));
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    logs.push_back(format!("[{}] {}", timestamp, msg));
                    if logs.len() > 100 {
                        logs.pop_front();
                    }
                }
                Ok(())
            })?;
            botarr_table.set("print", print)?;

            // execute(cmd, args)
            let execute = lua.create_function(|_, (cmd, args): (String, Vec<String>)| {
                std::thread::spawn(move || {
                    tracing::info!("Plugin executing: {} {:?}", cmd, args);
                    let _ = std::process::Command::new(&cmd)
                        .args(&args)
                        .spawn()
                        .map(|mut child| child.wait());
                });
                Ok(())
            })?;
            botarr_table.set("execute", execute)?;

            // download(url)
            let tx_clone = tx.clone();
            let download = lua.create_function(move |_, url: String| {
                tracing::info!("Plugin requested download: {}", url);
                let _ = tx_clone.send(PluginAction::Download(url));
                Ok(())
            })?;
            botarr_table.set("download", download)?;

            // queue(url)
            let tx_queue = tx.clone();
            let queue = lua.create_function(move |_, url: String| {
                tracing::info!("Plugin requested queue: {}", url);
                let _ = tx_queue.send(PluginAction::Queue(url));
                Ok(())
            })?;
            botarr_table.set("queue", queue)?;

            // monitor_channel(plugin_name, network, channel)
            let tx_clone2 = tx.clone();
            let monitor_channel = lua.create_function(
                move |_, (plugin, network, channel): (String, String, String)| {
                    tracing::info!(
                        "[{}] Plugin requested monitoring channel: {} on {}",
                        plugin,
                        channel,
                        network
                    );
                    let _ = tx_clone2.send(PluginAction::MonitorChannel(plugin, network, channel));
                    Ok(())
                },
            )?;
            botarr_table.set("monitor_channel", monitor_channel)?;

            globals.set("botarr", botarr_table)?;
        }

        Ok((
            Self {
                lua: Mutex::new(lua),
                loaded_scripts,
                recent_logs,
            },
            rx,
        ))
    }

    pub fn load_scripts(&self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                    if let Ok(code) = std::fs::read_to_string(path) {
                        tracing::info!("Loading plugin script: {:?}", path);
                        if let Ok(lua) = self.lua.lock() {
                            if let Err(e) = lua.load(&code).exec() {
                                tracing::error!("Failed to load plugin {:?}: {}", path, e);
                            } else {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    if let Ok(mut scripts) = self.loaded_scripts.write() {
                                        scripts.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Emit an event with arguments to Lua
    pub fn emit_signal(&self, event: &str, args: EventData) {
        if let Ok(lua) = self.lua.lock() {
            if let Ok(globals) = lua.globals().get::<_, Table>("botarr") {
                if let Ok(signals) = globals.get::<_, Table>("_signals") {
                    if let Ok(callbacks) = signals.get::<_, Table>(event) {
                        for pair in callbacks.pairs::<i32, Function>() {
                            if let Ok((_, func)) = pair {
                                let res = match &args {
                                    EventData::String(s) => func.call::<_, ()>(s.clone()),
                                    EventData::Tuple2(a, b) => {
                                        func.call::<_, ()>((a.clone(), b.clone()))
                                    }
                                    EventData::Tuple4(a, b, c, d) => func.call::<_, ()>((
                                        a.clone(),
                                        b.clone(),
                                        c.clone(),
                                        d.clone(),
                                    )),
                                };
                                if let Err(e) = res {
                                    tracing::error!(
                                        "Plugin callback error on event {}: {}",
                                        event,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
