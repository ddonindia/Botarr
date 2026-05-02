# Botarr Lua Plugin System

Botarr supports an extensible plugin system using the Lua scripting language. Inspired by the Irssi plugin ecosystem, this system allows you to automate tasks, extend functionality, and interact directly with the IRC and XDCC components.

## Installation

To install a plugin, simply place your `.lua` script inside the `plugins/` directory. Botarr will automatically load all scripts in this directory on startup.

## Available APIs

Botarr exposes a global `botarr` object to all Lua scripts, which provides the following methods:

### botarr.signal_add(event_name, callback)
Registers a callback function to listen for a specific event triggered by Botarr.

Supported Events:
* `irc_message`: Triggered when an IRC PRIVMSG is received.
  Arguments: `(network, channel, nick, message)`
* `irc_notice`: Triggered when an IRC NOTICE is received.
  Arguments: `(nick, message)`
* `download_started`: Triggered when an XDCC file transfer begins.
  Arguments: `(filename)`
* `download_completed`: Triggered when an XDCC file transfer finishes successfully.
  Arguments: `(filename)`
* `download_failed`: Triggered when an XDCC file transfer fails.
  Arguments: `(error_message)`

### botarr.download(url)
Programmatically queues a new XDCC download.
Example: `botarr.download("irc://irc.rizon.net/#nibl/Ginpachi-Sensei/1337")`

### botarr.print(message)
Logs a string message to the Botarr console/logs.
Example: `botarr.print("Plugin loaded successfully!")`

### botarr.execute(command, arguments)
Spawns an external command or script. The `arguments` must be a table (array) of strings.
Example: `botarr.execute("echo", {"Hello", "World"})`

## Example: Autodl Plugin

Below is an example of a simple autodl script (`plugins/autodl.lua`) that automatically downloads releases matching a specific pattern:

```lua
local function on_irc_message(network, channel, nick, message)
    if channel == "#testannounce" and string.find(message, "TEST.RELEASE.1080p") then
        local pack_num = string.match(message, "Pack #(%d+)")
        if pack_num then
            botarr.print("Autodl match! Queuing pack #" .. pack_num)
            botarr.download("irc://" .. network .. "/" .. channel .. "/" .. nick .. "/" .. pack_num)
        end
    end
end

botarr.signal_add("irc_message", on_irc_message)
```
