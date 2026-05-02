use mlua::{Lua, Result};
fn main() -> Result<()> {
    let lua = Lua::new();
    let globals = lua.globals();
    let print = lua.create_function(|lua, msg: String| {
        let dbg = lua.inspect_stack(1);
        if let Some(info) = dbg {
            println!("Caller source: {:?}", String::from_utf8_lossy(info.source().source.unwrap_or(&[])));
        } else {
            println!("No caller info");
        }
        println!("Msg: {}", msg);
        Ok(())
    })?;
    globals.set("print_plugin", print)?;
    lua.load("print_plugin('hello from script')").set_name("my_plugin.lua").exec()?;
    Ok(())
}
